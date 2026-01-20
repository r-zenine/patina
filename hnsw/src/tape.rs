use std::marker::PhantomData;

use crate::node::{self, NodeParser};

use crate::NodeId;
use crate::memory::{AlignedChunk, AllocationStrategy, MemoryManager};
use crate::tape_mutations::MutationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapeOffset(usize);

pub struct VectorTape<const DIM: usize, const CHUNK_SIZE: usize> {
    tape: Vec<AlignedChunk<f32, CHUNK_SIZE>>,
    offsets: Vec<TapeOffset>,
    memory_manager: MemoryManager<f32, CHUNK_SIZE>,
}

impl<const DIM: usize, const CHUNK_SIZE: usize> VectorTape<DIM, CHUNK_SIZE> {
    pub fn new() -> Self {
        Self {
            tape: Vec::new(),
            offsets: Vec::new(),
            memory_manager: MemoryManager::new(AllocationStrategy::AppendOnly),
        }
    }

    pub fn append_vector(
        &mut self,
        node_id: crate::NodeId,
        vector_data: &[f32; DIM],
    ) -> Result<(), MutationError> {
        let (_, range, new_chunk) = self.memory_manager.allocate(DIM);

        if let Some(chunk) = new_chunk {
            while self.tape.len() <= range.chunk_index as usize {
                self.tape.push(AlignedChunk::<f32, CHUNK_SIZE>::new());
            }
            self.tape[range.chunk_index as usize] = chunk;
        }

        let start = range.offset;
        let end = start + DIM;
        debug_assert!(end <= CHUNK_SIZE);

        let chunk = &mut self.tape[range.chunk_index as usize];
        chunk[start..end].copy_from_slice(vector_data);

        let byte_offset =
            (range.chunk_index as usize * CHUNK_SIZE + range.offset) * core::mem::size_of::<f32>();

        if node_id.to_usize() >= self.offsets.len() {
            self.offsets.resize(node_id.to_usize() + 1, TapeOffset(0));
        }
        self.offsets[node_id.to_usize()] = TapeOffset(byte_offset);

        Ok(())
    }

    pub fn get_vector(&self, node_id: &crate::NodeId) -> Option<&[f32; DIM]> {
        let (chunk_idx, elem_offset) = self
            .get_offset(node_id)
            .map(Self::tape_offset_to_chunk_offset)?;
        let aligned_chunk = self.tape.get(chunk_idx)?;

        if elem_offset + DIM > CHUNK_SIZE {
            return None;
        }

        let slice = &aligned_chunk[elem_offset..elem_offset + DIM];
        let ptr = slice.as_ptr() as *const [f32; DIM];

        Some(unsafe { &*ptr })
    }

    pub fn get_vector_ptr(&self, node_id: &crate::NodeId) -> Option<*const f32> {
        let (chunk_idx, elem_offset) = self
            .get_offset(node_id)
            .map(Self::tape_offset_to_chunk_offset)?;
        let aligned_chunk = self.tape.get(chunk_idx)?;
        if elem_offset + DIM > CHUNK_SIZE {
            return None;
        }
        Some(aligned_chunk.as_ptr().wrapping_add(elem_offset))
    }

    fn get_offset(&self, node_id: &crate::NodeId) -> Option<TapeOffset> {
        self.offsets.get(node_id.to_usize()).copied()
    }

    /// Get number of chunks
    pub fn chunks_count(&self) -> usize {
        self.tape.len()
    }

    /// Get reference to a chunk by index
    pub fn get_chunk(
        &self,
        idx: usize,
    ) -> Result<&crate::memory::AlignedChunk<f32, CHUNK_SIZE>, String> {
        self.tape
            .get(idx)
            .ok_or_else(|| format!("Chunk {} out of bounds", idx))
    }

    /// Get all offsets for serialization
    pub fn get_offsets(&self) -> Result<Vec<usize>, String> {
        Ok(self.offsets.iter().map(|offset| offset.0).collect())
    }

    fn tape_offset_to_chunk_offset(offset: TapeOffset) -> (usize, usize) {
        let bytes_per_chunk = CHUNK_SIZE * core::mem::size_of::<f32>();
        let chunk_index = offset.0 / bytes_per_chunk;
        let chunk_offset_bytes = offset.0 % bytes_per_chunk;
        let elem_offset = chunk_offset_bytes / core::mem::size_of::<f32>();
        (chunk_index, elem_offset)
    }
}

#[cfg(test)]
mod vector_tape_tests {
    use super::*;

    const DIM: usize = 128;
    const CHUNK_SIZE: usize = 256;

    fn make_vector(val: f32) -> [f32; DIM] {
        let mut data = [0.0; DIM];
        for (idx, item) in data.iter_mut().enumerate() {
            *item = val + idx as f32;
        }
        data
    }

    fn make_node_id(id: usize) -> NodeId {
        let mut bytes = [0u8; 5];
        let id_bytes = (id as u64).to_le_bytes();
        bytes.copy_from_slice(&id_bytes[..5]);
        NodeId(bytes)
    }

    #[test]
    fn append_and_get_single_vector() {
        let mut tape = VectorTape::<DIM, CHUNK_SIZE>::new();
        let node_id = make_node_id(1);
        let vector = make_vector(3.0);

        tape.append_vector(node_id, &vector).unwrap();
        let retrieved = tape.get_vector(&node_id).expect("vector should exist");

        assert_eq!(retrieved, &vector);
    }

    #[test]
    fn append_multiple_vectors_same_chunk() {
        let mut tape = VectorTape::<DIM, CHUNK_SIZE>::new();

        for i in 0..4 {
            let node = make_node_id(i);
            let vec = make_vector(i as f32);
            tape.append_vector(node, &vec).unwrap();
            assert_eq!(tape.get_vector(&node).unwrap(), &vec);
        }
    }

    #[test]
    fn append_vectors_cross_chunk_boundary() {
        let mut tape = VectorTape::<DIM, DIM>::new();

        for i in 0..4 {
            let node = &make_node_id(i);
            let vec = make_vector(i as f32 + 0.5);
            tape.append_vector(*node, &vec).unwrap();
            assert_eq!(tape.get_vector(node).unwrap(), &vec);
        }
        assert!(tape.tape.len() >= 2);
    }

    #[test]
    fn get_vector_missing_node_returns_none() {
        let tape = VectorTape::<DIM, CHUNK_SIZE>::new();
        assert!(tape.get_vector(&make_node_id(999)).is_none());
    }
}

pub struct GraphTape<K: Sized + Copy, const CHUNK_SIZE: usize> {
    tape: Vec<AlignedChunk<u8, CHUNK_SIZE>>,
    offests: Vec<TapeOffset>,
    memory_manager: MemoryManager<u8, CHUNK_SIZE>,
    _phantom: PhantomData<K>,
    max_connections_level0: usize,
    max_connections: usize,
    max_level: usize,
}

const NODE_LENGTH_SIZE: usize = 2; // u16 for node length prefix
const NEIGHBOR_COUNT_SIZE: usize = 1;
const NODE_ID_SIZE: usize = std::mem::size_of::<crate::NodeId>();

impl<K: Sized + Copy, const CHUNK_SIZE: usize> GraphTape<K, CHUNK_SIZE> {
    pub fn new(max_connections_level0: usize, max_connections: usize, max_level: usize) -> Self {
        Self {
            tape: Vec::new(),
            offests: Vec::new(),
            memory_manager: MemoryManager::new(AllocationStrategy::AppendOnly),
            _phantom: PhantomData,
            max_connections_level0,
            max_connections,
            max_level,
        }
    }

    fn level_capacity(&self, level: u8) -> usize {
        if level == 0 {
            self.max_connections_level0
        } else {
            self.max_connections
        }
    }

    pub fn neighbors_at_level<'a>(&'a self, node_id: &NodeId, level: u8) -> Option<&'a [NodeId]> {
        // Use the optimized pointer-based implementation
        if let Some((ptr, active_count)) = self.neighbors_at_level_ptr(node_id, level) {
            // Return slice with only active neighbors (not full capacity)
            let active_slice = unsafe { std::slice::from_raw_parts(ptr, active_count) };
            Some(active_slice)
        } else {
            None
        }
    }

    /// Prefetch neighbors array at a specific level for efficient graph traversal
    #[inline(always)]
    pub fn prefetch_neighbors_at_level(&self, node_id: &NodeId, level: u8) {
        if let Some((neighbors_ptr, active_count)) = self.neighbors_at_level_ptr(node_id, level) {
            // Only prefetch the neighbors array itself (NodeIds)
            crate::prefetch::prefetch_region(
                neighbors_ptr,
                active_count,
                &crate::prefetch::PrefetchKind::Temporal(crate::prefetch::CacheLevel::L1),
            );
        }
    }

    /// Get pointer to neighbors at a specific level using direct pointer arithmetic
    /// Returns (ptr, count) where ptr points to the start of neighbors array and count is the number of active neighbors
    pub fn neighbors_at_level_ptr(
        &self,
        node_id: &NodeId,
        level: u8,
    ) -> Option<(*const NodeId, usize)> {
        // Fast bounds check using GraphTape max_level
        if level as usize > self.max_level {
            return None;
        }

        let offset = self.get_offset(node_id)?;
        let (chunk_index, chunk_offset) = Self::tape_offset_to_chunk_offset(offset);

        let chunk = &self.tape[chunk_index];

        // Skip node length prefix and key - we know bounds from GraphTape members
        let mut current_offset = chunk_offset + NODE_LENGTH_SIZE + std::mem::size_of::<K>();

        // Calculate direct offset to target level using GraphTape members
        if level > 0 {
            let level0_size = NEIGHBOR_COUNT_SIZE + self.max_connections_level0 * NODE_ID_SIZE;
            let other_level_size = NEIGHBOR_COUNT_SIZE + self.max_connections * NODE_ID_SIZE;
            current_offset += level0_size + (level as usize - 1) * other_level_size;
        }

        // Read active neighbor count and validate against GraphTape capacity
        let active_count = chunk[current_offset] as usize;
        let level_capacity = if level == 0 {
            self.max_connections_level0
        } else {
            self.max_connections
        };

        if active_count > level_capacity {
            return None;
        }

        current_offset += NEIGHBOR_COUNT_SIZE;

        // Return pointer to neighbors array
        let neighbors_ptr = chunk[current_offset..].as_ptr() as *const NodeId;
        Some((neighbors_ptr, active_count))
    }

    /// Remove a node from the tape
    pub fn remove_node(&mut self, node_id: crate::NodeId) -> bool {
        if node_id.to_usize() >= self.offests.len() {
            return false;
        }

        let tape_offset = self.offests[node_id.to_usize()];
        let (chunk_index, chunk_offset) = Self::tape_offset_to_chunk_offset(tape_offset);

        if chunk_index >= self.tape.len() {
            return false;
        }

        let chunk = &self.tape[chunk_index];

        // Read node length to determine range to free
        if chunk_offset + NODE_LENGTH_SIZE > CHUNK_SIZE {
            return false;
        }

        let length_bytes = &chunk[chunk_offset..chunk_offset + NODE_LENGTH_SIZE];
        let node_length = u16::from_le_bytes([length_bytes[0], length_bytes[1]]) as usize;
        let total_size = NODE_LENGTH_SIZE + node_length;

        // Free the memory range
        let range = crate::memory::FreeRange {
            chunk_index: chunk_index as u16,
            offset: chunk_offset,
            length: total_size,
        };

        self.memory_manager.free(node_id, range);
        true
    }

    /// Swap a neighbor in a specific level of a node
    pub fn swap_neighbor(
        &mut self,
        node_id: crate::NodeId,
        level: u8,
        old_neighbor: crate::NodeId,
        new_neighbor: crate::NodeId,
    ) -> bool {
        if node_id.to_usize() >= self.offests.len() {
            return false;
        }

        let tape_offset = self.offests[node_id.to_usize()];
        let (chunk_index, chunk_offset) = Self::tape_offset_to_chunk_offset(tape_offset);

        if chunk_index >= self.tape.len() {
            return false;
        }

        // Parse the node to find the target level
        if let Some(parser) = self.node_at_offset(tape_offset) {
            let (_, mut parser, key_bytes) = parser.read_key(); // Now returns bytes consumed

            // Start tracking absolute position: skip length prefix + key
            let mut absolute_offset = chunk_offset + NODE_LENGTH_SIZE + key_bytes;

            // Iterate through levels to find the target level
            while let (Some((lvl, _)), next_parser, level_bytes) = parser.read_level() {
                let capacity = self.level_capacity(lvl);
                if lvl == level {
                    let chunk = &mut self.tape[chunk_index];
                    let header_offset = absolute_offset;
                    let active_count = chunk[header_offset] as usize;
                    let start_offset = header_offset + NEIGHBOR_COUNT_SIZE;
                    let empty_bytes = crate::EMPTY_NEIGHBOR.as_bytes();

                    #[cfg(debug_assertions)]
                    {
                        let mut current = Vec::new();
                        for i in 0..active_count {
                            let offset = start_offset + i * NODE_ID_SIZE;
                            let mut bytes = [0u8; NODE_ID_SIZE];
                            bytes.copy_from_slice(&chunk[offset..offset + NODE_ID_SIZE]);
                            current.push(crate::NodeId(bytes));
                        }
                        eprintln!(
                            "swap search node {} level {} looking for {} in {:?}",
                            node_id.to_usize(),
                            level,
                            old_neighbor.to_usize(),
                            current
                        );
                    }

                    let old_bytes = old_neighbor.as_bytes();
                    let new_bytes = new_neighbor.as_bytes();

                    if old_neighbor == crate::EMPTY_NEIGHBOR {
                        if new_neighbor == crate::EMPTY_NEIGHBOR {
                            return true;
                        }
                        if active_count >= capacity {
                            return false;
                        }
                        let slot_offset = start_offset + active_count * NODE_ID_SIZE;
                        chunk[slot_offset..slot_offset + NODE_ID_SIZE].copy_from_slice(new_bytes);
                        chunk[header_offset] = (active_count + 1) as u8;
                        return true;
                    }

                    let mut index = None;
                    for i in 0..active_count {
                        let offset = start_offset + i * NODE_ID_SIZE;
                        if chunk[offset..offset + NODE_ID_SIZE] == *old_bytes {
                            index = Some(i);
                            break;
                        }
                    }

                    let Some(idx) = index else {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "swap miss node {} level {} old {}",
                            node_id.to_usize(),
                            level,
                            old_neighbor.to_usize()
                        );
                        return false;
                    };

                    if new_neighbor == crate::EMPTY_NEIGHBOR {
                        if active_count == 0 {
                            return false;
                        }

                        let last_index = active_count - 1;
                        if idx != last_index {
                            let from_offset = start_offset + last_index * NODE_ID_SIZE;
                            let dest_offset = start_offset + idx * NODE_ID_SIZE;
                            let mut buf = [0u8; NODE_ID_SIZE];
                            buf.copy_from_slice(&chunk[from_offset..from_offset + NODE_ID_SIZE]);
                            chunk[dest_offset..dest_offset + NODE_ID_SIZE].copy_from_slice(&buf);
                        }

                        let last_offset = start_offset + (active_count - 1) * NODE_ID_SIZE;
                        chunk[last_offset..last_offset + NODE_ID_SIZE].copy_from_slice(empty_bytes);
                        chunk[header_offset] = (active_count - 1) as u8;
                        return true;
                    }

                    let offset = start_offset + idx * NODE_ID_SIZE;
                    chunk[offset..offset + NODE_ID_SIZE].copy_from_slice(new_bytes);
                    return true;
                }

                // Move to next level: advance by bytes consumed by this level
                absolute_offset += level_bytes;

                if let Some(p) = next_parser {
                    parser = p;
                } else {
                    break;
                }
            }
        }

        false // Level not found
    }

    // TODO make private
    pub fn get_offset(&self, node_id: &crate::NodeId) -> Option<TapeOffset> {
        self.offests.get(node_id.to_usize()).copied()
    }

    /// Get the next available node ID
    pub fn next_node_id(&self) -> crate::NodeId {
        self.memory_manager.next_node_id()
    }

    /// Get number of chunks
    pub fn chunks_count(&self) -> usize {
        self.tape.len()
    }

    /// Get reference to a chunk by index
    pub fn get_chunk(
        &self,
        idx: usize,
    ) -> Result<&crate::memory::AlignedChunk<u8, CHUNK_SIZE>, String> {
        self.tape
            .get(idx)
            .ok_or_else(|| format!("Chunk {} out of bounds", idx))
    }

    /// Get all offsets for serialization
    pub fn get_offsets(&self) -> Result<Vec<usize>, String> {
        Ok(self.offests.iter().map(|offset| offset.0).collect())
    }

    /// Append a node with a specific node ID (used during insertion)
    pub fn append_node(
        &mut self,
        node_id: crate::NodeId,
        node_data: &[u8],
    ) -> Result<(), crate::tape_mutations::MutationError> {
        // Materialize the node data into an owned buffer for writing
        let data = node_data.to_vec();
        let total_size = NODE_LENGTH_SIZE + data.len();
        let (_, range, new_chunk) = self.memory_manager.allocate(total_size);

        // Add new chunk to tape if needed
        if let Some(chunk) = new_chunk {
            // Ensure tape has enough chunks
            while self.tape.len() <= range.chunk_index as usize {
                self.tape.push(AlignedChunk::<u8, CHUNK_SIZE>::new());
            }
            self.tape[range.chunk_index as usize] = chunk;
        }

        // Write node length prefix (u16 little-endian)
        let length_bytes = (data.len() as u16).to_le_bytes();
        let chunk = &mut self.tape[range.chunk_index as usize];
        chunk[range.offset..range.offset + NODE_LENGTH_SIZE].copy_from_slice(&length_bytes);

        // Write node data
        let data_start = range.offset + NODE_LENGTH_SIZE;
        chunk[data_start..data_start + data.len()].copy_from_slice(&data);

        // Store offset mapping
        let tape_offset = TapeOffset(range.chunk_index as usize * CHUNK_SIZE + range.offset);
        if node_id.to_usize() >= self.offests.len() {
            self.offests.resize(node_id.to_usize() + 1, TapeOffset(0));
        }
        self.offests[node_id.to_usize()] = tape_offset;

        Ok(())
    }

    pub fn node_at_offset<'a>(
        &'a self,
        offset: TapeOffset,
    ) -> Option<NodeParser<'a, K, node::KeyToRead>> {
        let (chunk_index, chunk_offset) = Self::tape_offset_to_chunk_offset(offset);

        // Check if chunk exists
        if chunk_index >= self.tape.len() {
            return None;
        }

        let chunk = &self.tape[chunk_index];

        // Check if offset is within chunk bounds
        if chunk_offset >= CHUNK_SIZE {
            return None;
        }

        // First, read the node length (u16 at the beginning)
        if chunk_offset + NODE_LENGTH_SIZE > CHUNK_SIZE {
            return None; // Not enough space for length header
        }

        let length_bytes = &chunk[chunk_offset..chunk_offset + NODE_LENGTH_SIZE];
        let node_length = u16::from_le_bytes([length_bytes[0], length_bytes[1]]) as usize;

        // Check if the entire node fits in the remaining chunk space
        let node_start = chunk_offset + NODE_LENGTH_SIZE; // Skip the length prefix
        if node_start + node_length > CHUNK_SIZE {
            return None; // Node would extend beyond chunk boundary
        }

        // Create a slice containing just the node data (without length prefix)
        let node_bytes = &chunk[node_start..node_start + node_length];

        Some(crate::node::node_parser(
            node_bytes,
            self.max_connections_level0,
            self.max_connections,
        ))
    }

    /// Add a neighbor to a specific level of a node, if there's an empty slot
    pub fn add_neighbor(
        &mut self,
        node_id: crate::NodeId,
        level: u8,
        neighbor: crate::NodeId,
    ) -> bool {
        self.swap_neighbor(node_id, level, crate::EMPTY_NEIGHBOR, neighbor)
    }

    /// Remove a neighbor from a specific level of a node by replacing it with EMPTY_NEIGHBOR
    pub fn drop_neighbor(
        &mut self,
        node_id: crate::NodeId,
        level: u8,
        neighbor: crate::NodeId,
    ) -> bool {
        self.swap_neighbor(node_id, level, neighbor, crate::EMPTY_NEIGHBOR)
    }

    fn tape_offset_to_chunk_offset(offset: TapeOffset) -> (usize, usize) {
        let byte_offset = offset.0;
        let chunk_index = byte_offset / CHUNK_SIZE;
        let chunk_offset = byte_offset % CHUNK_SIZE;
        (chunk_index, chunk_offset)
    }
}

#[cfg(test)]
mod graph_tape_tests {
    use super::*; // Import from the parent module where ChunkedTape is defined
    use crate::NodeId;
    use crate::node::NodeBuilder;

    const TEST_MAX_CONNECTIONS_LEVEL0: usize = 2;
    const TEST_MAX_CONNECTIONS: usize = 1;
    const TEST_MAX_LEVEL: usize = 2;

    fn create_test_hnsw_node_data() -> Vec<u8> {
        let mut builder =
            NodeBuilder::new(TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS).with_key(1000u32);

        builder
            .set_neighbors_at_level(
                0,
                vec![
                    NodeId([0x01, 0x02, 0x03, 0x04, 0x05]),
                    NodeId([0x06, 0x07, 0x08, 0x09, 0x0A]),
                ],
            )
            .unwrap();
        builder
            .set_neighbors_at_level(1, vec![NodeId([0x0B, 0x0C, 0x0D, 0x0E, 0x0F])])
            .unwrap();

        builder.to_bytes().unwrap()
    }

    fn create_simple_node_data() -> Vec<u8> {
        let mut builder =
            NodeBuilder::new(TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS).with_key(42u32);
        builder
            .set_neighbors_at_level(0, vec![NodeId([0x01, 0x02, 0x03, 0x04, 0x05])])
            .unwrap();

        builder.to_bytes().unwrap()
    }

    #[test]
    fn test_chunked_tape_new() {
        let tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        assert_eq!(tape.tape.len(), 0);
        assert_eq!(tape.offests.len(), 0);
    }

    #[test]
    fn test_append_node_basic() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_simple_node_data();

        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Should have created one chunk
        assert_eq!(tape.tape.len(), 1);
        assert_eq!(tape.offests.len(), 2);

        // Verify the node can be read back
        let tape_offset = tape.offests[node_id.to_usize()];
        if let Some(parser) = tape.node_at_offset(tape_offset) {
            let (key, _, _) = parser.read_key();
            assert_eq!(key, 42u32);
        } else {
            panic!("Could not read back added node");
        }
    }

    #[test]
    fn test_append_multiple_nodes() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );

        let node1_data = create_simple_node_data();
        let node2_data = create_test_hnsw_node_data();

        let node1_id = tape.next_node_id();
        tape.append_node(node1_id, &node1_data).unwrap();
        let node2_id = tape.next_node_id();
        tape.append_node(node2_id, &node2_data).unwrap();

        // Should still be in same chunk (1024 bytes is plenty)
        assert_eq!(tape.tape.len(), 1);
        assert_eq!(tape.offests.len(), 3);

        // Verify both nodes can be read back
        let tape_offset1 = tape.offests[node1_id.to_usize()];
        if let Some(parser) = tape.node_at_offset(tape_offset1) {
            let (key, _, _) = parser.read_key();
            assert_eq!(key, 42u32);
        } else {
            panic!("Could not read back first node");
        }

        let tape_offset2 = tape.offests[node2_id.to_usize()];
        if let Some(parser) = tape.node_at_offset(tape_offset2) {
            let (key, _, _) = parser.read_key();
            assert_eq!(key, 1000u32);
        } else {
            panic!("Could not read back second node");
        }
    }

    #[test]
    fn test_remove_node() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_simple_node_data();

        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Remove the node
        let removed = tape.remove_node(node_id);
        assert!(removed);

        // Note: Current implementation frees memory but doesn't invalidate offset mapping
        // So a second remove might succeed but be redundant
        // This is fine for the memory manager architecture as it handles double-frees gracefully
    }

    #[test]
    fn test_remove_nonexistent_node() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let fake_node_id = NodeId([1, 2, 3, 4, 5]);

        let removed = tape.remove_node(fake_node_id);
        assert!(!removed);
    }

    #[test]
    fn test_swap_neighbor_basic() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_simple_node_data();

        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Original neighbor is NodeId([0x01, 0x02, 0x03, 0x04, 0x05])
        let old_neighbor = NodeId([0x01, 0x02, 0x03, 0x04, 0x05]);
        let new_neighbor = NodeId([0xFF, 0xFE, 0xFD, 0xFC, 0xFB]);

        // Swap neighbor at level 0
        let swapped = tape.swap_neighbor(node_id, 0, old_neighbor, new_neighbor);
        assert!(swapped);

        // Verify the swap worked by reading the node back
        let tape_offset = tape.offests[node_id.to_usize()];
        if let Some(parser) = tape.node_at_offset(tape_offset) {
            let (_, parser, _) = parser.read_key();

            let (level_data, _, _) = parser.read_level_with_counts();
            assert!(level_data.is_some());
            let (level, active_neighbors, empty_count) = level_data.unwrap();
            assert_eq!(level, 0);
            assert_eq!(active_neighbors.len(), 1);
            assert_eq!(active_neighbors[0], new_neighbor);
            assert_eq!(empty_count, TEST_MAX_CONNECTIONS_LEVEL0 - 1);
        } else {
            panic!("Could not read back modified node");
        }
    }

    #[test]
    fn test_swap_neighbor_multiple_levels() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_test_hnsw_node_data();

        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Swap neighbor in level 1 (originally NodeId([0x0B, 0x0C, 0x0D, 0x0E, 0x0F]))
        let old_neighbor = NodeId([0x0B, 0x0C, 0x0D, 0x0E, 0x0F]);
        let new_neighbor = NodeId([0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

        let swapped = tape.swap_neighbor(node_id, 1, old_neighbor, new_neighbor);
        assert!(swapped);

        // Verify level 0 is unchanged and level 1 is updated
        let tape_offset = tape.offests[node_id.to_usize()];
        if let Some(parser) = tape.node_at_offset(tape_offset) {
            let (_, parser, _) = parser.read_key();

            // Check level 0 - should be unchanged
            let (level_data, next_parser, _) = parser.read_level_with_counts();
            assert!(level_data.is_some());
            let (level, active_neighbors, empty_count) = level_data.unwrap();
            assert_eq!(level, 0);
            assert_eq!(active_neighbors.len(), 2);
            assert_eq!(empty_count, TEST_MAX_CONNECTIONS_LEVEL0 - 2);
            assert_eq!(active_neighbors[0], NodeId([0x01, 0x02, 0x03, 0x04, 0x05]));
            assert_eq!(active_neighbors[1], NodeId([0x06, 0x07, 0x08, 0x09, 0x0A]));

            // Check level 1 - should be updated
            let parser = next_parser.unwrap();
            let (level_data, _, _) = parser.read_level_with_counts();
            assert!(level_data.is_some());
            let (level, active_neighbors, empty_count) = level_data.unwrap();
            assert_eq!(level, 1);
            assert_eq!(active_neighbors.len(), 1);
            assert_eq!(empty_count, TEST_MAX_CONNECTIONS - 1);
            assert_eq!(active_neighbors[0], new_neighbor);
        } else {
            panic!("Could not read back modified node");
        }
    }

    #[test]
    fn test_swap_neighbor_nonexistent_level() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_simple_node_data();

        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        let old_neighbor = NodeId([0x01, 0x02, 0x03, 0x04, 0x05]);
        let new_neighbor = NodeId([0xFF, 0xFE, 0xFD, 0xFC, 0xFB]);

        // Try to swap at level 5 (doesn't exist)
        let swapped = tape.swap_neighbor(node_id, 5, old_neighbor, new_neighbor);
        assert!(!swapped);
    }

    #[test]
    fn test_swap_neighbor_nonexistent_neighbor() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_simple_node_data();

        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        let fake_old_neighbor = NodeId([0x99, 0x98, 0x97, 0x96, 0x95]);
        let new_neighbor = NodeId([0xFF, 0xFE, 0xFD, 0xFC, 0xFB]);

        // Try to swap a neighbor that doesn't exist
        let swapped = tape.swap_neighbor(node_id, 0, fake_old_neighbor, new_neighbor);
        assert!(!swapped);
    }

    #[test]
    fn test_swap_neighbor_nonexistent_node() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let fake_node_id = NodeId([1, 2, 3, 4, 5]);

        let old_neighbor = NodeId([0x01, 0x02, 0x03, 0x04, 0x05]);
        let new_neighbor = NodeId([0xFF, 0xFE, 0xFD, 0xFC, 0xFB]);

        let swapped = tape.swap_neighbor(fake_node_id, 0, old_neighbor, new_neighbor);
        assert!(!swapped);
    }

    #[test]
    fn test_node_id_recycling_after_remove() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );
        let node_data = create_simple_node_data();

        // Add and remove a node
        let node1_id = tape.next_node_id();
        tape.append_node(node1_id, &node_data).unwrap();

        let removed = tape.remove_node(node1_id);
        assert!(removed);

        // Add another node - should recycle the NodeId
        let node2_id = tape.next_node_id();
        tape.append_node(node2_id, &node_data).unwrap();
        assert_eq!(node2_id, node1_id); // Should reuse the same NodeId
    }

    #[test]
    fn test_chunk_boundary_handling() {
        // Use very small chunk size to test boundary conditions
        let mut tape = GraphTape::<u32, 32>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        ); // Even smaller chunks
        let node_data = create_test_hnsw_node_data(); // This is about 25 bytes + 2 length prefix = ~27 bytes

        // Add multiple nodes to force chunk boundary crossings
        let mut node_ids = Vec::new();
        for _ in 0..3 {
            let node_id = tape.next_node_id();
            tape.append_node(node_id, &node_data).unwrap();
            node_ids.push(node_id);
        }

        // Should have created multiple chunks since each node is ~27 bytes and chunk is 32 bytes
        assert!(
            tape.tape.len() >= 2,
            "Expected at least 2 chunks, got {}",
            tape.tape.len()
        );
        assert_eq!(tape.offests.len(), 4);

        // Verify all nodes can be read back correctly
        for (i, &node_id) in node_ids.iter().enumerate() {
            let tape_offset = tape.offests[node_id.to_usize()];
            if let Some(parser) = tape.node_at_offset(tape_offset) {
                let (key, _, _) = parser.read_key();
                assert_eq!(key, 1000u32);
            } else {
                panic!(
                    "Could not read back node {} at index {}",
                    node_id.to_usize(),
                    i
                );
            }
        }
    }

    #[test]
    fn test_empty_slot_detection_and_add_neighbor() {
        use crate::EMPTY_NEIGHBOR;

        // Create a node with empty neighbor slots (simulating HNSW node creation)
        let key = 42u64;
        let max_connections_level0 = 32;
        let max_connections = 16;

        let mut tape =
            GraphTape::<u64, 1024>::new(max_connections_level0, max_connections, TEST_MAX_LEVEL);

        let mut builder = NodeBuilder::new(max_connections_level0, max_connections).with_key(key);

        // Level 0: provision with EMPTY_NEIGHBOR placeholders
        let empty_neighbors_level0 = vec![EMPTY_NEIGHBOR; max_connections_level0];
        builder
            .set_neighbors_at_level(0, empty_neighbors_level0)
            .unwrap();

        // Level 1: provision with EMPTY_NEIGHBOR placeholders
        let empty_neighbors_level1 = vec![EMPTY_NEIGHBOR; max_connections];
        builder
            .set_neighbors_at_level(1, empty_neighbors_level1)
            .unwrap();

        let node_data = builder.to_bytes().unwrap();
        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Verify the node was created and we can read it back
        let tape_offset = tape.offests[node_id.to_usize()];
        let parser = tape
            .node_at_offset(tape_offset)
            .expect("Should be able to read node");
        let (parsed_key, mut parser, _) = parser.read_key();
        assert_eq!(parsed_key, key);

        // Verify level 0 has all empty slots
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 0);
        assert_eq!(empty_count, max_connections_level0);
        parser = next_parser.unwrap();

        // Verify level 1 has all empty slots
        let (level_data, _next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(active_neighbors.len(), 0);
        assert_eq!(empty_count, max_connections);

        // Now test add_neighbor functionality
        let new_neighbor = crate::NodeId([1, 2, 3, 4, 5]);

        // This should succeed since we have empty slots at level 0
        let result = tape.add_neighbor(node_id, 0, new_neighbor);
        assert!(
            result,
            "add_neighbor should succeed when empty slots are available"
        );

        // Verify the neighbor was added
        let tape_offset = tape.offests[node_id.to_usize()];
        let parser = tape
            .node_at_offset(tape_offset)
            .expect("Should be able to read node after neighbor addition");
        let (_, mut parser, _) = parser.read_key();

        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 1, "Should have 1 active neighbor");
        assert_eq!(
            empty_count,
            max_connections_level0 - 1,
            "Should have 1 less empty slot"
        );
        assert_eq!(
            active_neighbors[0], new_neighbor,
            "Should contain the added neighbor"
        );
        parser = next_parser.unwrap();

        // Verify level 1 is still unchanged
        let (level_data, _next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(active_neighbors.len(), 0);
        assert_eq!(empty_count, max_connections);

        // Test adding another neighbor at level 1
        let another_neighbor = crate::NodeId([6, 7, 8, 9, 10]);
        let result = tape.add_neighbor(node_id, 1, another_neighbor);
        assert!(
            result,
            "add_neighbor should succeed at level 1 when empty slots are available"
        );
    }

    #[test]
    fn test_add_neighbor_no_empty_slots() {
        let mut tape = GraphTape::<u32, 1024>::new(2, 2, TEST_MAX_LEVEL);

        // Create a node with NO empty slots (all slots filled)
        let key = 123u32;
        let neighbor1 = crate::NodeId([1, 1, 1, 1, 1]);
        let neighbor2 = crate::NodeId([2, 2, 2, 2, 2]);

        let mut builder = NodeBuilder::new(2, 2).with_key(key);
        let filled_neighbors = vec![neighbor1, neighbor2]; // Only 2 slots, both filled
        builder.set_neighbors_at_level(0, filled_neighbors).unwrap();

        let node_data = builder.to_bytes().unwrap();
        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Verify no empty slots
        let tape_offset = tape.offests[node_id.to_usize()];
        let parser = tape
            .node_at_offset(tape_offset)
            .expect("Should be able to read node");
        let (_, parser, _) = parser.read_key();
        let (level_data, _next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 2);
        assert_eq!(empty_count, 0); // No empty slots

        // Try to add a neighbor - should fail
        let new_neighbor = crate::NodeId([3, 3, 3, 3, 3]);
        let result = tape.add_neighbor(node_id, 0, new_neighbor);
        assert!(
            !result,
            "add_neighbor should fail when no empty slots are available"
        );
    }

    #[test]
    fn test_neighbors_at_level_ptr() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );

        // Create a test node with multiple levels
        let node_data = create_test_hnsw_node_data();
        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Test level 0 (should have 2 neighbors)
        let result = tape.neighbors_at_level_ptr(&node_id, 0);
        assert!(result.is_some());
        let (ptr, count) = result.unwrap();
        assert_eq!(count, 2);

        // Verify the actual neighbor data
        let neighbors_slice = unsafe { std::slice::from_raw_parts(ptr, count) };
        assert_eq!(neighbors_slice[0], NodeId([0x01, 0x02, 0x03, 0x04, 0x05]));
        assert_eq!(neighbors_slice[1], NodeId([0x06, 0x07, 0x08, 0x09, 0x0A]));

        // Test level 1 (should have 1 neighbor)
        let result = tape.neighbors_at_level_ptr(&node_id, 1);
        assert!(result.is_some());
        let (ptr, count) = result.unwrap();
        assert_eq!(count, 1);

        let neighbors_slice = unsafe { std::slice::from_raw_parts(ptr, count) };
        assert_eq!(neighbors_slice[0], NodeId([0x0B, 0x0C, 0x0D, 0x0E, 0x0F]));

        // Test non-existent level
        let result = tape.neighbors_at_level_ptr(&node_id, 99);
        assert!(result.is_none());

        // Test non-existent node
        let fake_node_id = NodeId([0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let result = tape.neighbors_at_level_ptr(&fake_node_id, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_neighbors_at_level_ptr_vs_original() {
        let mut tape = GraphTape::<u32, 1024>::new(
            TEST_MAX_CONNECTIONS_LEVEL0,
            TEST_MAX_CONNECTIONS,
            TEST_MAX_LEVEL,
        );

        let node_data = create_test_hnsw_node_data();
        let node_id = tape.next_node_id();
        tape.append_node(node_id, &node_data).unwrap();

        // Compare results between original and new implementation
        for level in 0..3 {
            let original_result = tape.neighbors_at_level(&node_id, level);
            let ptr_result = tape.neighbors_at_level_ptr(&node_id, level);

            match (original_result, ptr_result) {
                (Some(original_slice), Some((ptr, count))) => {
                    let ptr_slice = unsafe { std::slice::from_raw_parts(ptr, count) };
                    // Only compare active neighbors (filter out EMPTY_NEIGHBOR)
                    let original_active: Vec<_> = original_slice
                        .iter()
                        .filter(|n| **n != crate::EMPTY_NEIGHBOR)
                        .collect();
                    assert_eq!(original_active.len(), count);
                    for (i, &neighbor) in original_active.iter().enumerate() {
                        assert_eq!(*neighbor, ptr_slice[i]);
                    }
                }
                (None, None) => {
                    // Both should return None for invalid levels
                }
                _ => {
                    panic!("Mismatch between original and ptr implementation at level {level}");
                }
            }
        }
    }
}
