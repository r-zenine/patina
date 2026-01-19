use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{EMPTY_NEIGHBOR, NodeId};

use super::{
    AllocationStrategy, FreeRange, aligned_chunk::AlignedChunk, skiplist::FreeRangeSkipList,
};

#[derive(Debug)]
pub struct MemoryManager<T, const CHUNK_SIZE: usize> {
    num_chunks: i16,
    free_ranges: FreeRangeSkipList<CHUNK_SIZE>,
    node_id_provider: NodeIDProvider,
    allocation_strategy: AllocationStrategy,
    _marker: std::marker::PhantomData<T>,
}

impl<T, const CHUNK_SIZE: usize> MemoryManager<T, CHUNK_SIZE>
where
    T: Copy + Default + Sized,
{
    pub fn new(allocation_strategy: AllocationStrategy) -> Self {
        Self {
            num_chunks: -1,
            free_ranges: FreeRangeSkipList::new(0, allocation_strategy),
            node_id_provider: NodeIDProvider::new(),
            allocation_strategy,
            _marker: std::marker::PhantomData,
        }
    }

    #[allow(unused)]
    pub fn change_strategy(&mut self, allocation_strategy: AllocationStrategy) {
        self.free_ranges.change_strategy(allocation_strategy);
        self.allocation_strategy = allocation_strategy;
    }

    pub fn next_node_id(&self) -> NodeId {
        self.node_id_provider.peek()
    }

    pub fn allocate(
        &mut self,
        node_size: usize,
    ) -> (NodeId, FreeRange, Option<AlignedChunk<T, CHUNK_SIZE>>) {
        match self.allocation_strategy {
            AllocationStrategy::WithMemoryRecycling => {
                self.allocate_with_memory_recycling(node_size)
            }
            AllocationStrategy::AppendOnly => self.allocate_append(node_size),
        }
    }

    fn allocate_append(
        &mut self,
        node_size: usize,
    ) -> (NodeId, FreeRange, Option<AlignedChunk<T, CHUNK_SIZE>>) {
        // Try to allocate from current furthest range
        if let Some(allocated_range) = self.free_ranges.allocate_from_furthest(node_size) {
            // Successfully allocated from existing furthest range
            return (self.node_id_provider.draw(), allocated_range, None);
        }

        // Need to create new chunk
        let chunk = AlignedChunk::<T, CHUNK_SIZE>::new();
        self.num_chunks += 1;

        let new_range = FreeRange {
            chunk_index: self.num_chunks as u16,
            offset: 0,
            length: CHUNK_SIZE,
        };

        let (allocated_range, remainder) = new_range.split(node_size);

        // Set remainder as new furthest (if any)
        if let Some(remainder_range) = remainder {
            self.free_ranges.set_new_furthest(remainder_range);
        }

        (self.node_id_provider.draw(), allocated_range, Some(chunk))
    }

    fn allocate_with_memory_recycling(
        &mut self,
        node_size: usize,
    ) -> (NodeId, FreeRange, Option<AlignedChunk<T, CHUNK_SIZE>>) {
        if let Some(free_range) = self.free_ranges.find_best_fit(node_size) {
            // Remove the range we are about to use from the free list
            let _removed = self.free_ranges.remove(&free_range);

            let (allocated_range, remainder) = free_range.split(node_size);
            if let Some(r) = remainder {
                self.free_ranges.insert(r)
            }
            (self.node_id_provider.draw(), allocated_range, None)
        } else {
            // there is no freeRange available that fits our requirements, we need to allocate some
            let chunk = AlignedChunk::<T, CHUNK_SIZE>::new();
            let (free_range, remainder) = FreeRange {
                // safe because initial state is always only as small as -1
                chunk_index: (self.num_chunks + 1) as u16,
                offset: 0,
                length: CHUNK_SIZE,
            }
            .split(node_size);

            if let Some(r) = remainder {
                self.free_ranges.insert(r)
            }

            self.num_chunks += 1;
            (self.node_id_provider.draw(), free_range, Some(chunk))
        }
    }

    pub fn free(&mut self, id: NodeId, range: FreeRange) {
        self.free_ranges.insert(range);
        self.node_id_provider.retire(id);
    }
}

#[derive(Debug)]
struct NodeIDProvider {
    available_node_ids: BinaryHeap<Reverse<NodeId>>,
    max_node_id: NodeId,
}

impl NodeIDProvider {
    fn new() -> Self {
        Self {
            available_node_ids: BinaryHeap::new(),
            max_node_id: NodeId([1, 0, 0, 0, 0]), // Start with 1 to avoid EMPTY_NEIGHBOR
        }
    }

    fn peek(&self) -> NodeId {
        if let Some(Reverse(node_id)) = self.available_node_ids.peek() {
            *node_id
        } else {
            self.max_node_id
        }
    }

    fn draw(&mut self) -> NodeId {
        // First try to reuse a retired NodeId
        if let Some(Reverse(node_id)) = self.available_node_ids.pop() {
            node_id
        } else {
            // No recycled IDs available, create a new one
            let current_id = self.max_node_id;
            self.increment_max_node_id();
            current_id
        }
    }

    fn retire(&mut self, id: NodeId) {
        // Never allow EMPTY_NEIGHBOR to be recycled
        if id != EMPTY_NEIGHBOR {
            self.available_node_ids.push(Reverse(id));
        }
    }

    fn increment_max_node_id(&mut self) {
        // Increment the 5-byte little-endian counter
        let mut carry = 1;
        for byte in &mut self.max_node_id.0 {
            let new_val = (*byte as u16) + carry;
            *byte = new_val as u8;
            carry = new_val >> 8;
            if carry == 0 {
                break;
            }
        }
        // Note: If we overflow all 5 bytes, we just wrap around
    }
}

#[cfg(test)]
mod memory_manager_tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);
        // Should start with empty state
        assert_eq!(manager.num_chunks, -1);
    }

    #[test]
    fn test_first_allocation_creates_chunk() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // First allocation should create a new chunk
        let (node_id, range, chunk) = manager.allocate(512);

        assert_eq!(node_id.to_usize(), 1); // First NodeId
        assert_eq!(range.length, 512);
        assert_eq!(range.chunk_index, 0); // First chunk
        assert_eq!(range.offset, 0);
        assert!(chunk.is_some()); // Should return the new chunk
        assert_eq!(manager.num_chunks, 0);
    }

    #[test]
    fn test_node_id_recycling() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // Allocate two nodes
        let (node1, range1, _) = manager.allocate(100);
        let (node2, _range2, _) = manager.allocate(100);

        assert_eq!(node1.to_usize(), 1);
        assert_eq!(node2.to_usize(), 2);

        // Free the first node
        manager.free(node1, range1);

        // Next allocation should recycle the first NodeId
        let (node3, _, _) = manager.allocate(100);
        assert_eq!(node3.to_usize(), 1); // Should be recycled

        // Next allocation should get a new ID
        let (node4, _, _) = manager.allocate(100);
        assert_eq!(node4.to_usize(), 3); // Continue sequence
    }

    #[test]
    fn test_allocation_from_existing_free_space() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // First allocation creates chunk and leaves remainder
        let (_node1, range1, chunk1) = manager.allocate(512);
        assert!(chunk1.is_some());
        assert_eq!(range1.length, 512);

        // Second allocation should use remainder, no new chunk
        let (_node2, range2, chunk2) = manager.allocate(256);
        assert!(chunk2.is_none()); // No new chunk needed
        assert_eq!(range2.chunk_index, 0); // Same chunk
        assert_eq!(range2.offset, 512); // After first allocation
        assert_eq!(range2.length, 256);
        assert_eq!(manager.num_chunks, 0); // Still only one chunk
    }

    #[test]
    fn test_memory_reuse_after_free() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // Allocate and then free
        let (node1, range1, _) = manager.allocate(512);
        manager.free(node1, range1);

        // Next allocation should reuse the freed space
        let (_node2, range2, chunk2) = manager.allocate(400);
        assert!(chunk2.is_none()); // Should reuse existing space
        assert_eq!(range2.chunk_index, 0); // Same chunk as freed space
        assert_eq!(range2.offset, 0); // Should start at beginning
        assert_eq!(range2.length, 400);
    }

    #[test]
    fn test_multiple_chunks_allocation() {
        let mut manager = MemoryManager::<u8, 512>::new(AllocationStrategy::WithMemoryRecycling);

        // Fill first chunk completely
        let (_, _, chunk1) = manager.allocate(512);
        assert!(chunk1.is_some());
        assert_eq!(manager.num_chunks, 0);

        // Next allocation should create second chunk
        let (_, range2, chunk2) = manager.allocate(256);
        assert!(chunk2.is_some());
        assert_eq!(range2.chunk_index, 1); // Second chunk
        assert_eq!(manager.num_chunks, 1);
    }

    #[test]
    fn test_fragmentation_and_best_fit() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // Create fragmentation pattern
        let (_node1, _range1, _) = manager.allocate(200); // 0-200
        let (node2, range2, _) = manager.allocate(300); // 200-500
        let (_node3, _range3, _) = manager.allocate(200); // 500-700

        // Free middle allocation, creating 300-byte hole
        manager.free(node2, range2);

        // Allocate 250 bytes - should fit in the 300-byte hole
        let (_, range4, chunk4) = manager.allocate(250);
        assert!(chunk4.is_none()); // Should reuse existing space
        assert_eq!(range4.offset, 200); // Should be in the freed space
        assert_eq!(range4.length, 250);
    }

    #[test]
    fn test_node_id_recycling_order() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // Allocate several nodes
        let (node1, range1, _) = manager.allocate(100); // ID 1
        let (node2, range2, _) = manager.allocate(100); // ID 2
        let (node3, range3, _) = manager.allocate(100); // ID 3

        // Free them in different order
        manager.free(node2, range2); // Free ID 2
        manager.free(node1, range1); // Free ID 1
        manager.free(node3, range3); // Free ID 3

        // Should get them back in min-heap order (smallest first)
        let (recycled1, _, _) = manager.allocate(100);
        assert_eq!(recycled1.to_usize(), 1);

        let (recycled2, _, _) = manager.allocate(100);
        assert_eq!(recycled2.to_usize(), 2);

        let (recycled3, _, _) = manager.allocate(100);
        assert_eq!(recycled3.to_usize(), 3);

        // Next should continue sequence
        let (new_node, _, _) = manager.allocate(100);
        assert_eq!(new_node.to_usize(), 4);
    }

    #[test]
    fn test_stress_allocation_deallocation() {
        let mut manager = MemoryManager::<u8, 2048>::new(AllocationStrategy::WithMemoryRecycling);
        let mut allocations = Vec::new();

        // Make many allocations
        for i in 0..20 {
            let (node_id, range, _) = manager.allocate(64 + (i % 32));
            allocations.push((node_id, range));
        }

        // Free every third allocation
        let mut to_free = Vec::new();
        for (i, &(node_id, range)) in allocations.iter().enumerate() {
            if i % 3 == 0 {
                to_free.push((node_id, range));
            }
        }

        for (node_id, range) in to_free {
            manager.free(node_id, range);
        }

        // Should be able to make more allocations
        for _ in 0..5 {
            let (_, _, _) = manager.allocate(48);
        }
    }

    #[test]
    fn test_large_allocation_creates_new_chunk() {
        let mut manager = MemoryManager::<u8, 512>::new(AllocationStrategy::WithMemoryRecycling);

        // Make small allocation first
        let (_, _, _) = manager.allocate(100);

        // Large allocation should create new chunk since remainder is too small
        let (_, range, chunk) = manager.allocate(500);
        assert!(chunk.is_some()); // Should create new chunk
        assert_eq!(range.chunk_index, 1); // Second chunk
        assert_eq!(range.length, 500);
        assert_eq!(manager.num_chunks, 1);
    }

    #[test]
    fn test_exact_chunk_size_allocation() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::WithMemoryRecycling);

        // Allocate exactly chunk size
        let (_, range, chunk) = manager.allocate(1024);
        assert!(chunk.is_some());
        assert_eq!(range.length, 1024);
        assert_eq!(range.offset, 0);

        // Next allocation should create new chunk
        let (_, range2, chunk2) = manager.allocate(100);
        assert!(chunk2.is_some());
        assert_eq!(range2.chunk_index, 1);
        assert_eq!(manager.num_chunks, 1);
    }

    #[test]
    fn test_append_only_basic_allocation() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::AppendOnly);

        // First allocation creates chunk
        let (node1, range1, chunk1) = manager.allocate(512);
        assert!(chunk1.is_some());
        assert_eq!(range1.length, 512);
        assert_eq!(range1.chunk_index, 0);
        assert_eq!(range1.offset, 0);
        assert_eq!(node1.to_usize(), 1);

        // Second allocation uses remainder from same chunk (O(1) operation)
        let (node2, range2, chunk2) = manager.allocate(256);
        assert!(chunk2.is_none()); // No new chunk needed
        assert_eq!(range2.length, 256);
        assert_eq!(range2.chunk_index, 0); // Same chunk
        assert_eq!(range2.offset, 512); // After first allocation
        assert_eq!(node2.to_usize(), 2);
    }

    #[test]
    fn test_append_only_new_chunk_creation() {
        let mut manager = MemoryManager::<u8, 512>::new(AllocationStrategy::AppendOnly);

        // Fill first chunk completely
        let (_, range1, chunk1) = manager.allocate(512);
        assert!(chunk1.is_some());
        assert_eq!(range1.length, 512);
        assert_eq!(manager.num_chunks, 0);

        // Next allocation creates new chunk
        let (_, range2, chunk2) = manager.allocate(256);
        assert!(chunk2.is_some()); // New chunk created
        assert_eq!(range2.chunk_index, 1); // Second chunk
        assert_eq!(range2.offset, 0); // Start of new chunk
        assert_eq!(range2.length, 256);
        assert_eq!(manager.num_chunks, 1);
    }

    #[test]
    fn test_append_only_vs_recycling_performance() {
        let mut append_manager = MemoryManager::<u8, 2048>::new(AllocationStrategy::AppendOnly);
        let mut recycle_manager =
            MemoryManager::<u8, 2048>::new(AllocationStrategy::WithMemoryRecycling);

        // Both should produce same allocation pattern initially
        for i in 0..10 {
            let size = 64 + (i % 32);

            let (node1, range1, chunk1) = append_manager.allocate(size);
            let (node2, range2, chunk2) = recycle_manager.allocate(size);

            assert_eq!(node1.to_usize(), node2.to_usize());
            assert_eq!(range1, range2);
            assert_eq!(chunk1.is_some(), chunk2.is_some());
        }
    }

    #[test]
    fn test_strategy_switching() {
        let mut manager = MemoryManager::<u8, 1024>::new(AllocationStrategy::AppendOnly);

        // Make some allocations in AppendOnly mode
        let (node1, range1, _) = manager.allocate(200);
        let (_node2, _range2, _) = manager.allocate(300);

        // Switch to WithMemoryRecycling
        manager.change_strategy(AllocationStrategy::WithMemoryRecycling);

        // Free a range - should work with recycling now
        manager.free(node1, range1);

        // Allocate something that fits in freed space
        let (_node3, range3, chunk3) = manager.allocate(150);
        assert!(chunk3.is_none()); // Should reuse freed space
        assert_eq!(range3.chunk_index, 0); // Should be in first chunk
        assert_eq!(range3.offset, 0); // Should reuse freed space
    }

    #[test]
    fn test_append_only_stress() {
        let mut manager = MemoryManager::<u8, 2048>::new(AllocationStrategy::AppendOnly);
        let mut allocated_ranges = Vec::new();

        // Make many allocations
        for i in 0..50 {
            let size = 32 + (i % 64);
            let (node_id, range, _) = manager.allocate(size);
            allocated_ranges.push((node_id, range));
        }

        // Verify all allocations are valid
        assert_eq!(allocated_ranges.len(), 50);

        // Verify node IDs are sequential
        for (i, &(node_id, _)) in allocated_ranges.iter().enumerate() {
            assert_eq!(node_id.to_usize(), i + 1);
        }
    }
}
