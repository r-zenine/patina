use roaring::RoaringTreemap;
use smallvec::SmallVec;
use wyrand::WyRand;

use crate::memory::AllocationStrategy;
use crate::node::{KeySerialization, NodeBuilder};
use crate::prefetch::prefetch_region;
use crate::tape::{GraphTape, VectorTape};
use crate::tape_mutations::{
    MutationError, SequentialMutator, TapeMutation, TapeMutations, TapeMutator,
};

use rand_core::RngCore;

const _RAND_SEED: u64 = 42;
const _SMALL_VEC_SIZE_L: usize = 256;
const _SMALL_VEC_SIZE_S: usize = 64;

use crate::{
    DistanceFn, NodeId,
    prefetch::{CacheLevel, PrefetchKind},
};

// HNSW parameters
#[derive(Debug, Clone)]
pub struct HnswParams {
    pub max_connections: usize,        // M in the paper
    pub max_connections_level0: usize, // Ml in the paper (usually 2*M)
    pub ef_construction: usize,        // Size of dynamic candidate list during construction
    pub ef_search: usize,              // Size of dynamic candidate list during search
    pub max_level: u8,                 // Maximum number of levels
    pub inverse_log_connectivity: f64,
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            max_connections: 16,
            max_connections_level0: 32,
            inverse_log_connectivity: 1.0 / 16.0_f64.ln(),
            ef_construction: 200,
            ef_search: 50,
            max_level: 16,
        }
    }
}

// Candidate for search/construction
#[derive(Debug, Clone, PartialEq)]
struct Candidate<'a> {
    node_id: &'a NodeId,
    distance: f32,
}

impl<'a> Eq for Candidate<'a> {}

impl<'a> PartialOrd for Candidate<'a> {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // For max heap behavior, we want larger distances at the top
        self.distance.partial_cmp(&other.distance)
    }
}

impl<'a> Ord for Candidate<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

struct CandidateList<'a> {
    items: Vec<Candidate<'a>>,
    start_cursor: usize,
    max_len: usize,
}

impl<'a> CandidateList<'a> {
    fn new(max_len: usize) -> Self {
        CandidateList {
            items: Vec::with_capacity(max_len),
            start_cursor: 0,
            max_len,
        }
    }

    fn push(&mut self, candidate: Candidate<'a>) {
        if self.max_len == 0 {
            return; // Zero capacity - don't add anything
        }

        // Add to the end and sort only the valid range
        self.items.push(candidate);

        // If we exceeded capacity, remove the worst element
        if self.len() > self.max_len {
            // Sort the valid range to find the worst element
            self.items[self.start_cursor..]
                .sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

            // Remove the worst (last) element by just reducing the end
            self.items.pop();
        } else {
            // Sort the valid range to maintain order
            self.items[self.start_cursor..]
                .sort_unstable_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        }
    }

    fn pop(&mut self) -> Option<Candidate<'a>> {
        if self.len() > 0 {
            // Pop from the end (furthest element)
            self.items.pop()
        } else {
            None
        }
    }

    fn furthest_distance(&self) -> Option<f32> {
        if self.len() > 0 {
            self.items.last().map(|c| c.distance)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        if self.items.len() >= self.start_cursor {
            self.items.len() - self.start_cursor
        } else {
            0
        }
    }

    fn to_vec(&self) -> Vec<Candidate<'a>> {
        self.items[self.start_cursor..].to_vec()
    }
}

// Main HNSW index structure - Non-covering for data, covering for vectors
// K represents coordinates/references to external data (file paths, IDs, etc.)
// Vectors are stored in a separate tape for optimal memory layout
pub struct HnswIndex<
    K,
    const DIM: usize,
    const GRAPH_CHUNK_SIZE: usize,
    const VECTOR_CHUNK_SIZE: usize,
> where
    K: Copy + KeySerialization,
{
    graph_tape: GraphTape<K, GRAPH_CHUNK_SIZE>,
    vector_tape: VectorTape<DIM, VECTOR_CHUNK_SIZE>,
    entry_point: Option<NodeId>,
    params: HnswParams,
    node_count: usize,
    distance_fn: DistanceFn,
    wy_rand: WyRand,
}

impl<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>
    HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>
where
    K: Copy + KeySerialization,
{
    pub fn new(params: HnswParams, distance_fn: DistanceFn) -> Self {
        Self {
            graph_tape: GraphTape::new(
                params.max_connections_level0,
                params.max_connections,
                params.max_level as usize,
            ),
            vector_tape: VectorTape::new(),
            entry_point: None,
            params,
            node_count: 0,
            distance_fn,
            wy_rand: WyRand::new(_RAND_SEED),
        }
    }

    pub fn with_allocation_strategy(
        params: HnswParams,
        distance_fn: DistanceFn,
        _strategy: AllocationStrategy,
    ) -> Self {
        let graph_tape = GraphTape::new(
            params.max_connections_level0,
            params.max_connections,
            params.max_level as usize,
        );
        let vector_tape = VectorTape::new();
        // Note: We'd need to add a method to ChunkedTape to change strategy
        // vector_tape.change_allocation_strategy(AllocationStrategy::AppendOnly);

        Self {
            graph_tape,
            vector_tape,
            entry_point: None,
            params,
            node_count: 0,
            distance_fn,
            wy_rand: WyRand::new(_RAND_SEED),
        }
    }

    /// Create HNSW index with SimSIMD-optimized Euclidean distance
    pub fn new_euclidean(params: HnswParams) -> Self {
        Self::new(params, crate::select_best_euclidean())
    }

    /// Create HNSW index with SimSIMD-optimized Cosine distance
    pub fn new_cosine(params: HnswParams) -> Self {
        Self::new(params, crate::select_best_cosine())
    }

    /// Create HNSW index with SimSIMD-optimized Dot Product distance
    pub fn new_dot_product(params: HnswParams) -> Self {
        Self::new(params, crate::select_best_dot_product())
    }

    /// Create HNSW index with SimSIMD-optimized Manhattan distance
    pub fn new_manhattan(params: HnswParams) -> Self {
        Self::new(params, crate::select_best_manhattan())
    }

    pub fn len(&self) -> usize {
        self.node_count
    }

    pub fn is_empty(&self) -> bool {
        self.node_count == 0
    }

    pub fn entry_point(&self) -> Option<NodeId> {
        self.entry_point
    }

    // Get vector data for a node from vector tape
    pub fn get_node_vector(&self, node_id: NodeId) -> Option<&[f32; DIM]> {
        self.vector_tape.get_vector(&node_id)
    }

    #[cfg(debug_assertions)]
    pub fn graph_tape_mut(&mut self) -> &mut GraphTape<K, GRAPH_CHUNK_SIZE> {
        &mut self.graph_tape
    }

    pub fn neighbors_at_level(&self, node_id: &NodeId, level: u8) -> Option<&[NodeId]> {
        self.graph_tape.neighbors_at_level(node_id, level)
    }

    // Generate a random level for a new node
    fn random_level(&mut self) -> u8 {
        Self::sample_level(
            &mut self.wy_rand,
            self.params.inverse_log_connectivity,
            self.params.max_level,
        )
    }

    fn sample_level(rng: &mut WyRand, inverse_log_connectivity: f64, max_level: u8) -> u8 {
        // Map the 64-bit integer into the open interval (0, 1]
        let u = ((rng.next_u64() >> 11) as f64 + 1.0) / ((1_u64 << 53) as f64);
        let level = (-u.ln() * inverse_log_connectivity).floor() as u8;
        level.min(max_level)
    }

    // Insert a new node with key (coordinates to external data) and vector
    pub fn insert(&mut self, key: K, vector: [f32; DIM]) -> Result<NodeId, MutationError> {
        let level = self.random_level();
        let node_id = self.graph_tape.next_node_id();

        // Create new node with HNSW-style empty neighbor slots
        let builder = NodeBuilder::with_hnsw_levels(
            key,
            level,
            self.params.max_connections_level0,
            self.params.max_connections,
        )
        .map_err(|_| MutationError::NoEmptySlots { node_id, level: 0 })?;

        // Store vector in vector tape
        self.store_vector_data(node_id, vector)?;

        // If there are existing nodes, perform search and connection
        let mut mutations = Vec::with_capacity(256);
        if let Some(entry_point) = self.entry_point {
            let entry_level = self.get_node_level(&entry_point);

            // Phase 1: Greedy search from top levels down to our level
            let mut current_closest =
                self.greedy_search_to_level(vector.as_slice(), &entry_point, entry_level, level);

            // Phase 2: Connect at each level from our level down to 0
            for insert_level in (0..=level).rev() {
                let (next_closest, mutations_at_level) = self.connect_at_level(
                    &node_id,
                    vector.as_slice(),
                    current_closest,
                    insert_level,
                )?;

                mutations.extend(mutations_at_level);
                current_closest = next_closest;
            }
        }

        // Finalize insertion: build node, apply mutations, update entry point
        self.finalize_insertion(builder, node_id, mutations, level)
    }

    fn search_layer<'a>(
        &'a self,
        query: &[f32],
        entry_point: &'a NodeId,
        ef: usize,
        level: u8,
    ) -> Vec<Candidate<'a>> {
        self.prefetch_node_vector(entry_point, PrefetchKind::Temporal(CacheLevel::L1));

        let mut visited = RoaringTreemap::new();
        let mut candidates = CandidateList::new(ef); // For final results
        let mut dynamic_candidates = CandidateList::new(ef); // For search queue

        let entry_distance = self.calculate_distance(entry_point, query);
        let entry_candidate = Candidate {
            node_id: entry_point,
            distance: entry_distance,
        };

        candidates.push(entry_candidate.clone());
        dynamic_candidates.push(entry_candidate);
        visited.insert(entry_point.to_usize() as u64);

        while let Some(current) = dynamic_candidates.pop() {
            // If current is further than the furthest candidate in our ef list, stop
            if candidates.len() >= ef {
                if let Some(furthest_distance) = candidates.furthest_distance() {
                    if current.distance > furthest_distance {
                        break;
                    }
                }
            }

            // Examine neighbors of current node at the specified level
            if let Some(neighbors) = self.graph_tape.neighbors_at_level(current.node_id, level) {
                // Prefetch neighbors array for efficient traversal (following USearch pattern)
                self.graph_tape
                    .prefetch_neighbors_at_level(current.node_id, level);

                for neighbor in neighbors {
                    if neighbor.is_empty() || visited.contains(neighbor.to_usize() as u64) {
                        continue;
                    } else {
                        self.prefetch_node_vector(
                            neighbor,
                            PrefetchKind::NonTemporal(CacheLevel::L1),
                        );
                    }

                    visited.insert(neighbor.to_usize() as u64);
                    let neighbor_distance = self.calculate_distance(neighbor, query);
                    let neighbor_candidate = Candidate {
                        node_id: neighbor,
                        distance: neighbor_distance,
                    };

                    // If we have room or this neighbor is closer than our furthest candidate
                    if candidates.len() < ef {
                        candidates.push(neighbor_candidate.clone());
                        dynamic_candidates.push(neighbor_candidate);
                    } else if let Some(furthest_distance) = candidates.furthest_distance() {
                        if neighbor_distance < furthest_distance {
                            candidates.push(neighbor_candidate.clone()); // This will auto-remove furthest
                            dynamic_candidates.push(neighbor_candidate);
                        }
                    }
                }
            }
        }

        // CandidateList items are already sorted (closest first), just return them
        candidates.to_vec()
    }

    fn calculate_distance(&self, node_id: &NodeId, query: &[f32]) -> f32 {
        if let Some(node_vector) = self.vector_tape.get_vector(node_id) {
            (self.distance_fn)(node_vector.as_slice(), query)
        } else {
            f32::MAX // Return maximum distance if node not found
        }
    }

    #[inline(always)]
    fn prefetch_node_vector(&self, node_id: &NodeId, kind: PrefetchKind) {
        if let Some(vector) = self.vector_tape.get_vector_ptr(node_id) {
            prefetch_region(vector, DIM, &kind);
        }
    }

    fn get_node_level<'a>(&'a self, node_id: &'a NodeId) -> u8 {
        // Parse the node to determine its level count
        if let Some(offset) = self.graph_tape.get_offset(node_id) {
            if let Some(parser) = self.graph_tape.node_at_offset(offset) {
                let (_, parser, _) = parser.read_key();
                // Count levels by parsing through the node
                let mut level = 0u8;
                let mut current_parser = Some(parser);

                while let Some(parser) = current_parser {
                    let (level_data, next_parser, _) = parser.read_level();
                    if level_data.is_some() {
                        level += 1;
                        current_parser = next_parser;
                    } else {
                        break;
                    }
                }

                return level.saturating_sub(1); // Convert count to max level index
            }
        }
        0
    }

    /// Perform greedy search from top levels down to target level
    fn greedy_search_to_level<'a>(
        &'a self,
        query: &[f32],
        entry_point: &'a NodeId,
        entry_level: u8,
        target_level: u8,
    ) -> &'a NodeId {
        let mut current_closest = entry_point;

        for search_level in ((target_level + 1)..=entry_level).rev() {
            current_closest = self
                .search_layer(query, current_closest, 1, search_level)
                .into_iter()
                .next()
                .unwrap()
                .node_id;
        }
        current_closest
    }

    /// Process connections for a single level during insertion
    /// TODO be mindfull of allocations in all of this code path
    fn connect_at_level<'a>(
        &'a self,
        node_id: &'a NodeId,
        query: &[f32],
        current_closest: &'a NodeId,
        level: u8,
    ) -> Result<(&'a NodeId, SmallVec<[TapeMutation; _SMALL_VEC_SIZE_L]>), MutationError> {
        let ef = self.params.ef_construction;

        let mut candidates = self.search_layer(query, current_closest, ef, level);

        let max_connections = self.capacity_for_level(level);
        let survivors = self.prune_candidates(&mut candidates, max_connections);

        let mut mutations: SmallVec<[TapeMutation; _SMALL_VEC_SIZE_L]> = SmallVec::new();

        self.graph_tape.prefetch_neighbors_at_level(node_id, level);

        self.prefetch_node_vector(node_id, PrefetchKind::Temporal(CacheLevel::L1));
        for &survivor in &survivors {
            self.prefetch_node_vector(survivor, PrefetchKind::Temporal(CacheLevel::L2));
        }

        let old_neighbors = self
            .graph_tape
            .neighbors_at_level(node_id, level)
            .unwrap_or_default();
        mutations.extend(self.diff_neighbors(node_id, level, old_neighbors, &survivors));

        for &neighbor in &survivors {
            if level > self.get_node_level(neighbor) {
                continue;
            }

            self.prefetch_node_vector(neighbor, PrefetchKind::Temporal(CacheLevel::L2));
            self.graph_tape.prefetch_neighbors_at_level(neighbor, level);

            let neighbor_old = self
                .graph_tape
                .neighbors_at_level(neighbor, level)
                .unwrap_or_default();

            for survivor in old_neighbors {
                self.prefetch_node_vector(survivor, PrefetchKind::NonTemporal(CacheLevel::L2));
            }

            let neighbor_keep = self.prune_nodes(
                neighbor,
                neighbor_old,
                node_id,
                self.capacity_for_level(level),
            );

            mutations.extend(self.diff_neighbors(neighbor, level, neighbor_old, &neighbor_keep));
        }

        let next = survivors.first().copied().unwrap_or(current_closest);
        Ok((next, mutations))
    }

    fn capacity_for_level(&self, level: u8) -> usize {
        if level == 0 {
            self.params.max_connections_level0
        } else {
            self.params.max_connections
        }
    }

    fn prune_candidates<'nodes>(
        &self,
        candidates: &mut [Candidate<'nodes>],
        max: usize,
    ) -> SmallVec<[&'nodes NodeId; _SMALL_VEC_SIZE_S]> {
        candidates.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        let mut chosen: SmallVec<[&NodeId; _SMALL_VEC_SIZE_S]> = SmallVec::new();

        for candidate in candidates.iter() {
            self.prefetch_node_vector(candidate.node_id, PrefetchKind::Temporal(CacheLevel::L2));
        }

        for candidate in candidates.iter() {
            if chosen.len() == max {
                break;
            }
            if self.is_diverse(candidate, &chosen) {
                chosen.push(candidate.node_id);
            }
        }
        chosen
    }

    fn is_diverse<'nodes>(&self, candidate: &Candidate<'nodes>, chosen: &[&'nodes NodeId]) -> bool {
        for &other in chosen {
            if let Some(dist) = self.distance_between_nodes(candidate.node_id, other) {
                if dist < candidate.distance {
                    return false;
                }
            }
        }

        true
    }

    fn prune_nodes<'a>(
        &'a self,
        center: &'a NodeId,
        old_neighboors: &'a [NodeId],
        candidate: &'a NodeId,
        max: usize,
    ) -> SmallVec<[&'a NodeId; _SMALL_VEC_SIZE_S]> {
        self.prefetch_node_vector(center, PrefetchKind::Temporal(CacheLevel::L1));
        self.prefetch_node_vector(candidate, PrefetchKind::NonTemporal(CacheLevel::L1));

        // Single pass deduplication
        for node in old_neighboors {
            self.prefetch_node_vector(node, PrefetchKind::NonTemporal(CacheLevel::L1));
        }

        let mut candidates: SmallVec<[Candidate<'a>; _SMALL_VEC_SIZE_L]> =
            SmallVec::<[Candidate<'a>; _SMALL_VEC_SIZE_L]>::new();

        if let Some(distance) = self.distance_between_nodes(candidate, center) {
            candidates.push(Candidate {
                node_id: candidate,
                distance,
            });
        }

        for node_id in old_neighboors {
            if let Some(distance) = self.distance_between_nodes(candidate, center) {
                candidates.push(Candidate { node_id, distance });
            }
        }

        self.prune_candidates(&mut candidates, max)
    }

    fn diff_neighbors<'a>(
        &'a self,
        node_id: &'a NodeId,
        level: u8,
        old: &[NodeId],
        new: &[&NodeId],
    ) -> Vec<TapeMutation> {
        use smallvec::SmallVec;

        let mut old_sorted: SmallVec<[NodeId; _SMALL_VEC_SIZE_L]> =
            SmallVec::with_capacity(old.len());
        old_sorted.extend_from_slice(old);
        old_sorted.sort_unstable();

        let mut new_sorted: SmallVec<[NodeId; _SMALL_VEC_SIZE_L]> =
            SmallVec::with_capacity(new.len());
        new_sorted.extend(new.iter().map(|n| **n));
        new_sorted.sort_unstable();

        let mut removals: SmallVec<[NodeId; _SMALL_VEC_SIZE_S]> = SmallVec::new();
        let mut additions: SmallVec<[NodeId; _SMALL_VEC_SIZE_S]> = SmallVec::new();

        let mut i = 0;
        let mut j = 0;

        while i < old_sorted.len() && j < new_sorted.len() {
            match old_sorted[i].cmp(&new_sorted[j]) {
                std::cmp::Ordering::Less => {
                    removals.push(old_sorted[i]);
                    i += 1;
                }
                std::cmp::Ordering::Greater => {
                    additions.push(new_sorted[j]);
                    j += 1;
                }
                std::cmp::Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
            }
        }

        removals.extend_from_slice(&old_sorted[i..]);
        additions.extend_from_slice(&new_sorted[j..]);

        let swap_count = removals.len().min(additions.len());
        let mut mutations = Vec::with_capacity(removals.len() + additions.len());

        for idx in 0..swap_count {
            mutations.push(TapeMutation::Swap {
                node: *node_id,
                level,
                old: removals[idx],
                new: additions[idx],
            });
        }

        for removed in removals.iter().skip(swap_count) {
            mutations.push(TapeMutation::Drop {
                node: *node_id,
                level,
                neighbor: *removed,
            });
        }

        for added in additions.iter().skip(swap_count) {
            mutations.push(TapeMutation::Add {
                node: *node_id,
                level,
                neighbor: *added,
            });
        }

        mutations
    }

    fn distance_between_nodes<'a>(&'a self, a: &'a NodeId, b: &'a NodeId) -> Option<f32> {
        let vec_a = self.vector_tape.get_vector(a)?;
        let vec_b = self.vector_tape.get_vector(b)?;
        Some((self.distance_fn)(vec_a, vec_b))
    }

    /// Store vector data in the vector tape
    fn store_vector_data(
        &mut self,
        node_id: NodeId,
        vector: [f32; DIM],
    ) -> Result<(), MutationError> {
        self.vector_tape.append_vector(node_id, &vector)
    }

    /// Finalize node insertion by building and appending the node, then applying mutations
    fn finalize_insertion(
        &mut self,
        builder: NodeBuilder<K>,
        node_id: NodeId,
        mutations: Vec<TapeMutation>,
        level: u8,
    ) -> Result<NodeId, MutationError> {
        // Build and append the new node first
        let node_bytes = builder
            .to_bytes()
            .map_err(|_| MutationError::NoEmptySlots { node_id, level: 0 })?;
        self.graph_tape.append_node(node_id, &node_bytes)?;

        // Apply all mutations after the node exists
        if !mutations.is_empty() {
            SequentialMutator::apply_mutations(
                &mut self.graph_tape,
                TapeMutations::Unordered(mutations),
            )?;
        }

        // Only update entry point after everything succeeds
        let entry_level = self
            .entry_point
            .map(|ep| self.get_node_level(&ep))
            .unwrap_or(0);
        if level > entry_level || self.entry_point.is_none() {
            self.entry_point = Some(node_id);
        }

        self.node_count += 1;
        Ok(node_id)
    }

    /// Get number of graph chunks for serialization
    pub fn graph_chunks_count(&self) -> usize {
        self.graph_tape.chunks_count()
    }

    /// Get number of vector chunks for serialization
    pub fn vector_chunks_count(&self) -> usize {
        self.vector_tape.chunks_count()
    }

    /// Get reference to a graph chunk by index
    pub fn get_graph_chunk(
        &self,
        idx: usize,
    ) -> Result<&crate::memory::AlignedChunk<u8, GRAPH_CHUNK_SIZE>, String> {
        self.graph_tape.get_chunk(idx)
    }

    /// Get reference to a vector chunk by index
    pub fn get_vector_chunk(
        &self,
        idx: usize,
    ) -> Result<&crate::memory::AlignedChunk<f32, VECTOR_CHUNK_SIZE>, String> {
        self.vector_tape.get_chunk(idx)
    }

    /// Get vector offsets for serialization
    pub fn get_vector_offsets(&self) -> Result<Vec<usize>, String> {
        self.vector_tape.get_offsets()
    }

    /// Get graph offsets for serialization
    pub fn get_graph_offsets(&self) -> Result<Vec<usize>, String> {
        self.graph_tape.get_offsets()
    }

    /// Get metadata for serialization
    pub fn get_metadata(&self) -> Result<crate::persistence::format::IndexMetadata, String> {
        Ok(crate::persistence::format::IndexMetadata {
            entry_point: self.entry_point,
            node_count: self.node_count,
            max_connections: self.params.max_connections,
            max_connections_level0: self.params.max_connections_level0,
            max_level: self.params.max_level,
        })
    }
}

// Search implementation for all distance functions
impl<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>
    HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>
where
    K: Copy + KeySerialization,
{
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(NodeId, f32)> {
        self.search_with_ef(query, k, self.params.ef_search)
    }

    pub fn search_with_ef(&self, query: &[f32], k: usize, ef: usize) -> Vec<(NodeId, f32)> {
        if self.entry_point.is_none() {
            return Vec::new();
        }

        let entry_point = self.entry_point.unwrap();

        // Phase 1: Search from top level down to level 1
        let mut current_closest = &entry_point;

        // Get the level of the entry point
        let entry_level = self.get_node_level(&entry_point);

        for level in (1..=entry_level).rev() {
            current_closest = self
                .search_layer(query, current_closest, 1, level)
                .pop()
                .unwrap()
                .node_id;
        }

        // Phase 2: Search level 0 with ef candidates
        let candidates = self.search_layer(query, current_closest, ef, 0);

        // Return top k candidates
        candidates
            .into_iter()
            .take(k)
            .map(|c| (*c.node_id, c.distance))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use super::*;
    use crate::simd_distances;

    // Simple coordinate type for testing - could be file path, database ID, etc.
    // In this test case, we use u64 as a simple document ID
    type DocumentId = u64;

    // Test configuration with small dimensions and reasonable chunk sizes
    const TEST_DIM: usize = 3;
    const TEST_GRAPH_CHUNK_SIZE: usize = 1024;
    const TEST_VECTOR_CHUNK_SIZE: usize = 4096;

    type TestIndex = HnswIndex<DocumentId, TEST_DIM, TEST_GRAPH_CHUNK_SIZE, TEST_VECTOR_CHUNK_SIZE>;

    #[test]
    fn test_distance_functions() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        // Test SimSIMD Euclidean distance
        let euclidean = simd_distances::euclidean_f32(&a, &b);
        assert!((euclidean - 5.196).abs() < 0.01); // sqrt(27) ≈ 5.196

        // Test SimSIMD dot product distance
        let dot_product = simd_distances::dot_product_f32(&a, &b);
        assert_eq!(dot_product, -32.0); // -(1*4 + 2*5 + 3*6)

        // Test SimSIMD Manhattan distance
        let manhattan = simd_distances::manhattan_f32(&a, &b);
        assert_eq!(manhattan, 9.0); // |1-4| + |2-5| + |3-6|
    }

    #[test]
    fn test_hnsw_creation() {
        let params = HnswParams::default();
        let index = TestIndex::new_euclidean(params);

        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert_eq!(index.entry_point(), None);
    }

    #[test]
    fn test_candidate_ordering() {
        let mut candidates = BinaryHeap::new();

        candidates.push(Candidate {
            node_id: &NodeId([1, 0, 0, 0, 0]),
            distance: 3.0,
        });
        candidates.push(Candidate {
            node_id: &NodeId([2, 0, 0, 0, 0]),
            distance: 1.0,
        });
        candidates.push(Candidate {
            node_id: &NodeId([3, 0, 0, 0, 0]),
            distance: 2.0,
        });

        // Should pop in order of largest distance first (max heap)
        assert_eq!(candidates.pop().unwrap().distance, 3.0);
        assert_eq!(candidates.pop().unwrap().distance, 2.0);
        assert_eq!(candidates.pop().unwrap().distance, 1.0);
    }

    #[test]
    fn test_hnsw_insert_single_node() {
        let params = HnswParams::default();
        let mut index = TestIndex::new_euclidean(params);

        // Insert first node - should become entry point
        let vector = [1.0, 2.0, 3.0]; // Fixed-size array
        let document_id = 42u64; // This represents coordinates to external data
        let result = index.insert(document_id, vector);

        assert!(result.is_ok());
        let node_id = result.unwrap();

        // Verify index state
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
        assert_eq!(index.entry_point(), Some(node_id));

        // Verify vector can be retrieved
        let retrieved_vector = index.get_node_vector(node_id);
        assert!(retrieved_vector.is_some());
        assert_eq!(retrieved_vector.unwrap(), &vector);
    }

    #[test]
    fn test_hnsw_insert_two_nodes() {
        let params = HnswParams::default();
        let mut index = TestIndex::new_euclidean(params);

        // Insert first node
        let vector1 = [1.0, 0.0, 0.0];
        let result1 = index.insert(0u64, vector1);
        assert!(result1.is_ok());
        let node_id1 = result1.unwrap();

        // Insert second node
        let vector2 = [0.0, 1.0, 0.0];
        let result2 = index.insert(1u64, vector2);
        if let Err(e) = &result2 {
            panic!("Insert failed for second vector: {e:?}");
        }
        assert!(result2.is_ok());
        let node_id2 = result2.unwrap();

        // Verify index state
        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
        assert!(index.entry_point().is_some());

        // Verify vectors can be retrieved
        assert_eq!(index.get_node_vector(node_id1).unwrap(), &vector1);
        assert_eq!(index.get_node_vector(node_id2).unwrap(), &vector2);
    }

    #[test]
    fn test_hnsw_insert_multiple_nodes() {
        let params = HnswParams::default();
        let mut index = TestIndex::new_euclidean(params);

        // Insert multiple nodes
        let vectors = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
        ];

        let mut node_ids = Vec::new();
        for (i, vector) in vectors.iter().enumerate() {
            let document_id = i as u64; // Document ID representing external data coordinates
            let result = index.insert(document_id, *vector);
            if let Err(e) = &result {
                panic!("Insert failed for vector {i}: {e:?}");
            }
            assert!(result.is_ok());
            node_ids.push(result.unwrap());
        }

        // Verify index state
        assert_eq!(index.len(), 4);
        assert!(!index.is_empty());
        assert!(index.entry_point().is_some());

        // Verify all vectors can be retrieved
        for (i, &node_id) in node_ids.iter().enumerate() {
            let retrieved_vector = index.get_node_vector(node_id);
            assert!(retrieved_vector.is_some());
            assert_eq!(retrieved_vector.unwrap(), &vectors[i]);
        }
    }

    fn id(byte: u8) -> NodeId {
        NodeId([byte, 0, 0, 0, 0])
    }

    #[test]
    fn diff_neighbors_prefers_swaps_when_possible() {
        let params = HnswParams::default();
        let index = TestIndex::new_euclidean(params);

        let old = vec![id(1), id(2), id(3)];
        let new = [id(1), id(4), id(3)];

        let _old_refs: Vec<&NodeId> = old.iter().collect();
        let new_refs: Vec<&NodeId> = new.iter().collect();
        let mutations = index.diff_neighbors(&id(9), 0, &old, &new_refs);
        assert_eq!(mutations.len(), 1);

        match &mutations[0] {
            TapeMutation::Swap { old, new, .. } => {
                assert_eq!(*old, id(2));
                assert_eq!(*new, id(4));
            }
            other => panic!("expected swap mutation, got {other:?}"),
        }
    }

    #[test]
    fn diff_neighbors_emits_add_for_extra_neighbors() {
        let params = HnswParams::default();
        let index = TestIndex::new_euclidean(params);

        let old = vec![id(1), id(2)];
        let new = [id(2), id(3), id(4)];

        let _old_refs: Vec<&NodeId> = old.iter().collect();
        let new_refs: Vec<&NodeId> = new.iter().collect();
        let mutations = index.diff_neighbors(&id(9), 0, &old, &new_refs);

        assert_eq!(mutations.len(), 2);

        assert!(matches!(
            mutations[0],
            TapeMutation::Swap {
                old,
                new,
                ..
            } if old == id(1) && new == id(3)
        ));

        assert!(matches!(
            mutations[1],
            TapeMutation::Add {
                neighbor,
                ..
            } if neighbor == id(4)
        ));
    }

    #[test]
    fn test_hnsw_search_functionality() {
        let params = HnswParams::default();
        let mut index = TestIndex::new_euclidean(params);

        // Insert some test vectors
        let vectors = [
            [1.0, 0.0, 0.0], // Document ID 0
            [0.0, 1.0, 0.0], // Document ID 1
            [0.0, 0.0, 1.0], // Document ID 2
            [1.0, 1.0, 0.0], // Document ID 3
        ];

        for (i, vector) in vectors.iter().enumerate() {
            let document_id = i as u64; // Document ID representing external data coordinates
            let result = index.insert(document_id, *vector);
            assert!(result.is_ok());
        }

        // Test search - search for something close to first vector
        let query = [0.9, 0.1, 0.0];
        let results = index.search(&query, 2);

        // Should return some results
        assert!(!results.is_empty());
        assert!(results.len() <= 2);

        // Results should be sorted by distance (closest first)
        for i in 1..results.len() {
            assert!(results[i - 1].1 <= results[i].1);
        }
    }

    // Helper function to create test candidates for CandidateList tests
    fn create_test_candidate(id: u8, distance: f32) -> Candidate<'static> {
        static NODE_IDS: [NodeId; 10] = [
            NodeId([0, 0, 0, 0, 0]),
            NodeId([1, 0, 0, 0, 0]),
            NodeId([2, 0, 0, 0, 0]),
            NodeId([3, 0, 0, 0, 0]),
            NodeId([4, 0, 0, 0, 0]),
            NodeId([5, 0, 0, 0, 0]),
            NodeId([6, 0, 0, 0, 0]),
            NodeId([7, 0, 0, 0, 0]),
            NodeId([8, 0, 0, 0, 0]),
            NodeId([9, 0, 0, 0, 0]),
        ];

        Candidate {
            node_id: &NODE_IDS[id as usize],
            distance,
        }
    }

    #[test]
    fn test_candidate_list_creation() {
        let list = CandidateList::new(5);
        assert_eq!(list.len(), 0);
        assert_eq!(list.furthest_distance(), None);
    }

    #[test]
    fn test_candidate_list_push_single_item() {
        let mut list = CandidateList::new(3);
        let candidate = create_test_candidate(0, 5.0);

        list.push(candidate);

        assert_eq!(list.len(), 1);
        assert_eq!(list.furthest_distance(), Some(5.0));
        assert_eq!(list.items[0].distance, 5.0);
    }

    #[test]
    fn test_candidate_list_maintains_sorted_order() {
        let mut list = CandidateList::new(5);

        // Add items in random order
        list.push(create_test_candidate(0, 3.0));
        list.push(create_test_candidate(1, 1.0));
        list.push(create_test_candidate(2, 5.0));
        list.push(create_test_candidate(3, 2.0));

        assert_eq!(list.len(), 4);

        // Should be sorted by distance (ascending)
        assert_eq!(list.items[0].distance, 1.0);
        assert_eq!(list.items[1].distance, 2.0);
        assert_eq!(list.items[2].distance, 3.0);
        assert_eq!(list.items[3].distance, 5.0);

        assert_eq!(list.furthest_distance(), Some(5.0));
    }

    #[test]
    fn test_candidate_list_capacity_limit() {
        let mut list = CandidateList::new(3);

        // Fill to capacity
        list.push(create_test_candidate(0, 3.0));
        list.push(create_test_candidate(1, 1.0));
        list.push(create_test_candidate(2, 5.0));

        assert_eq!(list.len(), 3);
        assert_eq!(list.furthest_distance(), Some(5.0));

        // Try to add a worse candidate - should be rejected
        list.push(create_test_candidate(3, 6.0));

        assert_eq!(list.len(), 3);
        assert_eq!(list.furthest_distance(), Some(5.0));

        // Add a better candidate - should replace the worst
        list.push(create_test_candidate(4, 2.0));

        assert_eq!(list.len(), 3);
        assert_eq!(list.furthest_distance(), Some(3.0)); // 5.0 should be removed

        // Verify sorted order is maintained
        assert_eq!(list.items[0].distance, 1.0);
        assert_eq!(list.items[1].distance, 2.0);
        assert_eq!(list.items[2].distance, 3.0);
    }

    #[test]
    fn test_candidate_list_pop_functionality() {
        let mut list = CandidateList::new(5);

        list.push(create_test_candidate(0, 3.0));
        list.push(create_test_candidate(1, 1.0));
        list.push(create_test_candidate(2, 5.0));

        // Pop should remove from the end (furthest)
        let popped = list.pop().unwrap();
        assert_eq!(popped.distance, 5.0);
        assert_eq!(list.len(), 2);
        assert_eq!(list.furthest_distance(), Some(3.0));

        let popped = list.pop().unwrap();
        assert_eq!(popped.distance, 3.0);
        assert_eq!(list.len(), 1);
        assert_eq!(list.furthest_distance(), Some(1.0));

        let popped = list.pop().unwrap();
        assert_eq!(popped.distance, 1.0);
        assert_eq!(list.len(), 0);
        assert_eq!(list.furthest_distance(), None);

        // Pop from empty list
        assert!(list.pop().is_none());
    }

    #[test]
    fn test_candidate_list_edge_cases() {
        // Zero capacity
        let mut list = CandidateList::new(0);
        list.push(create_test_candidate(0, 1.0));
        assert_eq!(list.len(), 0);

        // Single capacity
        let mut list = CandidateList::new(1);
        list.push(create_test_candidate(0, 3.0));
        assert_eq!(list.len(), 1);
        assert_eq!(list.furthest_distance(), Some(3.0));

        // Add better candidate
        list.push(create_test_candidate(1, 1.0));
        assert_eq!(list.len(), 1);
        assert_eq!(list.furthest_distance(), Some(1.0));

        // Add worse candidate
        list.push(create_test_candidate(2, 5.0));
        assert_eq!(list.len(), 1);
        assert_eq!(list.furthest_distance(), Some(1.0));
    }

    #[test]
    fn test_candidate_list_capacity_overflow() {
        let mut list = CandidateList::new(10);

        // Add items that exceed capacity, should maintain only best 10
        for i in 0..100 {
            list.push(create_test_candidate((i % 10) as u8, i as f32));
        }

        assert_eq!(list.len(), 10);

        // Should contain only the 10 best (lowest) distances: 0-9
        for i in 0..10 {
            assert_eq!(list.items[i].distance, i as f32);
        }

        assert_eq!(list.furthest_distance(), Some(9.0));
    }
}
