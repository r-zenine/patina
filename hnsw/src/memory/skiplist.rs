use std::collections::VecDeque;
use std::hash::Hasher;
use std::ptr::NonNull;

use wyrand::WyRand;

use super::metrics::FragmentationMetrics;
use super::{AllocationStrategy, FreeRange};

type NodePtr = Option<NonNull<SkipNode<FreeRange>>>;

#[derive(Debug)]
pub struct SkipNode<T> {
    value: T,
    forward: Vec<NodePtr>,
    level: usize,
}

impl<T> SkipNode<T> {
    fn new(value: T, level: usize) -> Box<Self> {
        Box::new(Self {
            value,
            forward: vec![None; level + 1],
            level,
        })
    }
}

const MAX_LEVEL: usize = 16;
const LEVEL_PROBABILITY: f64 = 0.5;

#[derive(Debug)]
pub struct FreeRangeSkipList<const CHUNK_SIZE: usize> {
    sentinel: Box<SkipNode<FreeRange>>,
    metrics: FragmentationMetrics,
    current_level: usize,
    rng: WyRand,
    size: usize,
    furthest_free_range: FreeRange,
    // AppendOnly mode optimization
    allocation_strategy: AllocationStrategy,
    pending_ranges: VecDeque<FreeRange>,
    current_furthest: Option<FreeRange>,
}

impl<const CHUNK_SIZE: usize> FreeRangeSkipList<CHUNK_SIZE> {
    pub fn new(num_chunks: usize, allocation_strategy: AllocationStrategy) -> Self {
        // Create sentinel node with minimum possible values that exists at all levels
        let sentinel_range = FreeRange {
            chunk_index: 0,
            offset: 0,
            length: 0,
        };
        let sentinel = SkipNode::new(sentinel_range, MAX_LEVEL);

        Self {
            sentinel,
            metrics: FragmentationMetrics::new(num_chunks),
            current_level: 0,
            rng: WyRand::new(
                std::collections::hash_map::DefaultHasher::new()
                    .finish()
                    .wrapping_add(num_chunks as u64),
            ),
            size: 0,
            furthest_free_range: FreeRange {
                chunk_index: 0,
                offset: 0,
                length: 0,
            },
            allocation_strategy,
            pending_ranges: VecDeque::new(),
            current_furthest: None,
        }
    }

    fn random_level(&mut self) -> usize {
        let mut level = 0;
        while level < MAX_LEVEL && (self.rng.rand() as f64 / u64::MAX as f64) < LEVEL_PROBABILITY {
            level += 1;
        }
        level
    }

    pub fn change_strategy(&mut self, new_strategy: AllocationStrategy) {
        if matches!(self.allocation_strategy, AllocationStrategy::AppendOnly)
            && matches!(new_strategy, AllocationStrategy::WithMemoryRecycling)
        {
            // Drain pending_ranges into skiplist (don't double-count size)
            while let Some(range) = self.pending_ranges.pop_front() {
                self.insert_into_skiplist_no_size_increment(range);
            }

            // Insert current_furthest if it exists
            if let Some(range) = self.current_furthest.take() {
                self.insert_into_skiplist_no_size_increment(range);
            }
        }

        self.allocation_strategy = new_strategy;
    }

    pub fn insert(&mut self, range: FreeRange) {
        match self.allocation_strategy {
            AllocationStrategy::WithMemoryRecycling => {
                self.insert_into_skiplist(range);
            }
            AllocationStrategy::AppendOnly => {
                self.insert_into_queue(range);
            }
        }
    }

    fn insert_into_skiplist(&mut self, range: FreeRange) {
        self.insert_into_skiplist_internal(range, true);
    }

    fn insert_into_skiplist_no_size_increment(&mut self, range: FreeRange) {
        self.insert_into_skiplist_internal(range, false);
    }

    fn insert_into_skiplist_internal(&mut self, range: FreeRange, increment_size: bool) {
        // Update fragmentation metrics
        self.update_metrics_on_insert(&range);

        // Update furthest_free_range if this range is further or if no range exists yet
        if self.furthest_free_range.length == 0
            || range.chunk_index > self.furthest_free_range.chunk_index
            || (range.chunk_index == self.furthest_free_range.chunk_index
                && range.offset > self.furthest_free_range.offset)
        {
            self.furthest_free_range = range;
        }

        let new_level = self.random_level();

        // Create update array to track insertion path
        let mut update: Vec<NodePtr> = vec![None; MAX_LEVEL + 1];

        // Find insertion position and populate update array
        self.find_update_path(&range, &mut update);

        // Create new node
        let mut new_node = SkipNode::new(range, new_level);
        let new_node_ptr = NonNull::from(new_node.as_mut());

        // Update forward pointers using the update array
        (0..=new_level).for_each(|level| {
            if let Some(mut update_node) = update[level] {
                unsafe {
                    new_node.forward[level] = update_node.as_ref().forward[level];
                    update_node.as_mut().forward[level] = Some(new_node_ptr);
                }
            } else {
                // Should never happen with sentinel - all levels should have sentinel as predecessor
                unreachable!("Sentinel should provide predecessor at all levels");
            }
        });

        // Update current_level if necessary
        if new_level > self.current_level {
            self.current_level = new_level;
        }

        // Always leak the node - sentinel handles the head role
        Box::leak(new_node);

        if increment_size {
            self.size += 1;
        }
    }

    fn insert_into_queue(&mut self, range: FreeRange) {
        // Update fragmentation metrics
        self.update_metrics_on_insert(&range);

        // Update current_furthest if this range is further or if no range exists yet
        if let Some(current) = &self.current_furthest {
            if range.chunk_index > current.chunk_index
                || (range.chunk_index == current.chunk_index && range.offset > current.offset)
            {
                // Push old furthest to pending queue and update current
                self.pending_ranges.push_back(*current);
                self.current_furthest = Some(range);
            } else {
                // This range is not the furthest, add to pending queue
                self.pending_ranges.push_back(range);
            }
        } else {
            // No current furthest, this becomes it
            self.current_furthest = Some(range);
        }

        self.size += 1;
    }

    fn update_metrics_on_insert(&mut self, range: &FreeRange) {
        let chunk_idx = range.chunk_index as usize;
        let length = range.length;

        // Ensure chunk vectors are large enough
        while self.metrics.chunk_free_bytes.len() <= chunk_idx {
            self.metrics.chunk_free_bytes.push(0);
        }

        self.metrics.chunk_free_bytes[chunk_idx] += length;
    }

    // Helper function to traverse right at a specific level until predicate is false
    fn traverse_level<F>(&self, level: usize, mut current: NodePtr, predicate: F) -> NodePtr
    where
        F: Fn(&FreeRange) -> bool,
    {
        while let Some(current_ptr) = current {
            unsafe {
                let current_node = current_ptr.as_ref();

                if let Some(next_ptr) = current_node.forward[level] {
                    let next_node = next_ptr.as_ref();

                    if !predicate(&next_node.value) {
                        break;
                    }

                    current = Some(next_ptr);
                } else {
                    break;
                }
            }
        }
        current
    }

    // Helper function to get the next node in traversal order
    fn get_next_node(&self, current: NodePtr) -> NodePtr {
        current.and_then(|ptr| unsafe { ptr.as_ref().forward[0] })
    }

    fn find_update_path(&self, target: &FreeRange, update: &mut [NodePtr]) {
        let mut current = Some(NonNull::from(self.sentinel.as_ref()));

        // Start from the highest level and work down
        for level in (0..=MAX_LEVEL).rev() {
            current = self.traverse_level(level, current, |value| value < target);
            update[level] = current;
        }
    }

    pub fn find_best_fit(&self, required_size: usize) -> Option<FreeRange> {
        match self.allocation_strategy {
            AllocationStrategy::WithMemoryRecycling => self.find_best_fit_skiplist(required_size),
            AllocationStrategy::AppendOnly => self.find_best_fit_queue(required_size),
        }
    }

    fn find_best_fit_skiplist(&self, required_size: usize) -> Option<FreeRange> {
        let mut current = Some(NonNull::from(self.sentinel.as_ref()));

        // Start from highest level and traverse down to find best fit
        for level in (0..=MAX_LEVEL).rev() {
            current = self.traverse_level(level, current, |value| value.length < required_size);
        }

        // Get the next node (first one that satisfies >= required_size)
        self.get_next_node(current).and_then(|ptr| unsafe {
            let node = ptr.as_ref();
            // Skip sentinel node (length = 0) as it's not a real range
            if node.value.length >= required_size && node.value.length > 0 {
                Some(node.value)
            } else {
                None
            }
        })
    }

    fn find_best_fit_queue(&self, required_size: usize) -> Option<FreeRange> {
        // In AppendOnly mode, only check current_furthest
        self.current_furthest.and_then(|range| {
            if range.length >= required_size {
                Some(range)
            } else {
                None
            }
        })
    }

    // New method for AppendOnly allocation - consumes and updates current_furthest
    pub fn allocate_from_furthest(&mut self, required_size: usize) -> Option<FreeRange> {
        if !matches!(self.allocation_strategy, AllocationStrategy::AppendOnly) {
            return None;
        }

        if let Some(furthest) = self.current_furthest.take() {
            if furthest.length >= required_size {
                let (allocated, remainder) = furthest.split(required_size);

                // Update current_furthest with remainder
                self.current_furthest = remainder;

                Some(allocated)
            } else {
                // Put it back and return None
                self.current_furthest = Some(furthest);
                None
            }
        } else {
            None
        }
    }

    // Method to set new furthest range in AppendOnly mode (for new chunks)
    pub fn set_new_furthest(&mut self, range: FreeRange) {
        if !matches!(self.allocation_strategy, AllocationStrategy::AppendOnly) {
            return;
        }

        // Move existing current_furthest to pending queue if it exists
        if let Some(current) = self.current_furthest.take() {
            self.pending_ranges.push_back(current);
        }

        self.current_furthest = Some(range);
        self.update_metrics_on_insert(&range);
        self.size += 1;
    }

    pub fn remove(&mut self, range: &FreeRange) -> bool {
        // Find the node to remove using update path
        let mut update: Vec<NodePtr> = vec![None; MAX_LEVEL + 1];
        self.find_update_path(range, &mut update);

        // Check if the node actually exists
        let target_node = self.get_next_node(update[0]);
        if let Some(node_ptr) = target_node {
            unsafe {
                let node = node_ptr.as_ref();

                // Verify this is the exact node we want to remove
                if node.value != *range {
                    return false; // Node not found
                }

                // Update all forward pointers that point to this node
                (0..=node.level).for_each(|level| {
                    if let Some(mut pred_ptr) = update[level] {
                        let pred_node = pred_ptr.as_mut();
                        pred_node.forward[level] = node.forward[level];
                    }
                });

                // Update metrics
                self.update_metrics_on_removal(range);

                // Convert back to Box to properly deallocate
                // Safety: We know this was originally a Box::leak'd pointer
                let _boxed_node = Box::from_raw(node_ptr.as_ptr());
                // _boxed_node will be automatically dropped here

                self.size -= 1;
                true
            }
        } else {
            false // Node not found
        }
    }

    fn update_metrics_on_removal(&mut self, range: &FreeRange) {
        let chunk_idx = range.chunk_index as usize;
        let length = range.length;

        if chunk_idx < self.metrics.chunk_free_bytes.len() {
            // Decrease free bytes for this chunk
            if self.metrics.chunk_free_bytes[chunk_idx] >= length {
                self.metrics.chunk_free_bytes[chunk_idx] -= length;
            }
        }

        // If we're removing the furthest_free_range, we need to find the new furthest
        if *range == self.furthest_free_range {
            self.update_furthest_free_range();
        }
    }

    fn update_furthest_free_range(&mut self) {
        // Reset to sentinel values
        self.furthest_free_range = FreeRange {
            chunk_index: 0,
            offset: 0,
            length: 0,
        };

        // Traverse the skiplist to find the actual furthest range
        let mut current = Some(NonNull::from(self.sentinel.as_ref()));

        while let Some(current_ptr) = current {
            unsafe {
                let current_node = current_ptr.as_ref();

                // Skip sentinel node (length = 0)
                if current_node.value.length > 0 {
                    let range = &current_node.value;
                    // If no range exists yet (length=0) or this range is further
                    if self.furthest_free_range.length == 0
                        || range.chunk_index > self.furthest_free_range.chunk_index
                        || (range.chunk_index == self.furthest_free_range.chunk_index
                            && range.offset > self.furthest_free_range.offset)
                    {
                        self.furthest_free_range = *range;
                    }
                }

                current = current_node.forward[0];
            }
        }
    }
}

#[cfg(test)]
impl<const CHUNK_SIZE: usize> FreeRangeSkipList<CHUNK_SIZE> {
    pub fn get_furthest_free_range(&self) -> Option<FreeRange> {
        if self.furthest_free_range.length > 0 {
            Some(self.furthest_free_range)
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_range(length: usize, chunk_index: u16, offset: usize) -> FreeRange {
        FreeRange {
            chunk_index,
            offset,
            length,
        }
    }

    #[test]
    fn test_new_skiplist_is_empty() {
        let skiplist = FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        assert_eq!(skiplist.size(), 0);
    }

    #[test]
    fn test_insert_single_range() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        let range = create_test_range(100, 0, 0);

        skiplist.insert(range);

        assert_eq!(skiplist.size(), 1);
    }

    #[test]
    fn test_find_best_fit_exact_match() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        let range = create_test_range(100, 0, 0);
        skiplist.insert(range);

        let result = skiplist.find_best_fit(100);
        assert_eq!(result, Some(range));
    }

    #[test]
    fn test_find_best_fit_larger_available() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        let range = create_test_range(200, 0, 0);
        skiplist.insert(range);

        let result = skiplist.find_best_fit(100);
        assert_eq!(result, Some(range));
    }

    #[test]
    fn test_find_best_fit_no_suitable_range() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        let range = create_test_range(50, 0, 0);
        skiplist.insert(range);

        let result = skiplist.find_best_fit(100);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_best_fit_multiple_ranges() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);

        // Insert ranges in non-sorted order
        skiplist.insert(create_test_range(300, 2, 0));
        skiplist.insert(create_test_range(150, 1, 0));
        skiplist.insert(create_test_range(75, 0, 0));

        // Should find the smallest suitable range (150)
        let result = skiplist.find_best_fit(100);
        assert_eq!(result, Some(create_test_range(150, 1, 0)));
    }

    #[test]
    fn test_remove_existing_range() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        let range = create_test_range(100, 0, 0);
        skiplist.insert(range);

        let removed = skiplist.remove(&range);
        assert!(removed);
        assert_eq!(skiplist.size(), 0);
    }

    #[test]
    fn test_remove_nonexistent_range() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);
        skiplist.insert(create_test_range(100, 0, 0));

        let nonexistent = create_test_range(200, 1, 0);
        let removed = skiplist.remove(&nonexistent);
        assert!(!removed);
        assert_eq!(skiplist.size(), 1);
    }

    #[test]
    fn test_multiple_inserts_and_removes() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);

        let ranges = vec![
            create_test_range(100, 0, 0),
            create_test_range(200, 1, 0),
            create_test_range(50, 0, 100),
        ];

        // Insert all ranges
        for range in &ranges {
            skiplist.insert(*range);
        }
        assert_eq!(skiplist.size(), 3);

        // Remove middle range
        assert!(skiplist.remove(&ranges[1]));
        assert_eq!(skiplist.size(), 2);

        // Verify remaining ranges can still be found
        assert_eq!(skiplist.find_best_fit(50), Some(ranges[2]));
        assert_eq!(skiplist.find_best_fit(100), Some(ranges[0]));
    }

    #[test]
    fn test_ordering_by_length_then_chunk_then_offset() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);

        // Same length, different chunks - should prefer lower chunk_index
        skiplist.insert(create_test_range(100, 2, 0));
        skiplist.insert(create_test_range(100, 1, 0));

        let result = skiplist.find_best_fit(100);
        assert_eq!(result, Some(create_test_range(100, 1, 0)));
    }

    #[test]
    fn test_stress_operations() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(100, AllocationStrategy::WithMemoryRecycling);
        let mut inserted_ranges = Vec::new();

        // Insert many ranges
        for i in 0..50 {
            let range = create_test_range(i * 10 + 50, (i % 10) as u16, i * 100);
            skiplist.insert(range);
            inserted_ranges.push(range);
        }

        assert_eq!(skiplist.size(), 50);

        // Remove every other range
        for (i, range) in inserted_ranges.iter().enumerate() {
            if i % 2 == 0 {
                assert!(skiplist.remove(range));
            }
        }

        assert_eq!(skiplist.size(), 25);

        // Verify remaining ranges can still be found
        for (i, range) in inserted_ranges.iter().enumerate() {
            if i % 2 == 1 {
                let found = skiplist.find_best_fit(range.length);
                assert!(found.is_some());
            }
        }
    }

    #[test]
    fn test_furthest_free_range_tracking() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);

        // Initially should be None
        assert_eq!(skiplist.get_furthest_free_range(), None);

        // Insert first range
        let range1 = create_test_range(100, 0, 0);
        skiplist.insert(range1);
        assert_eq!(skiplist.get_furthest_free_range(), Some(range1));

        // Insert range with higher chunk index - should become furthest
        let range2 = create_test_range(50, 1, 0);
        skiplist.insert(range2);
        assert_eq!(skiplist.get_furthest_free_range(), Some(range2));

        // Insert range with same chunk but higher offset - should become furthest
        let range3 = create_test_range(25, 1, 100);
        skiplist.insert(range3);
        assert_eq!(skiplist.get_furthest_free_range(), Some(range3));

        // Insert range with lower position - should not change furthest
        let range4 = create_test_range(200, 0, 500);
        skiplist.insert(range4);
        assert_eq!(skiplist.get_furthest_free_range(), Some(range3));
    }

    #[test]
    fn test_furthest_free_range_removal() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);

        let range1 = create_test_range(100, 0, 0);
        let range2 = create_test_range(50, 1, 0);
        let range3 = create_test_range(25, 1, 100);

        skiplist.insert(range1);
        skiplist.insert(range2);
        skiplist.insert(range3);

        // range3 should be furthest
        assert_eq!(skiplist.get_furthest_free_range(), Some(range3));

        // Remove furthest range - should update to next furthest (range2)
        assert!(skiplist.remove(&range3));
        assert_eq!(skiplist.get_furthest_free_range(), Some(range2));

        // Remove new furthest - should update to range1
        assert!(skiplist.remove(&range2));
        assert_eq!(skiplist.get_furthest_free_range(), Some(range1));

        // Remove last range - should become None
        assert!(skiplist.remove(&range1));
        assert_eq!(skiplist.get_furthest_free_range(), None);
    }

    #[test]
    fn test_furthest_free_range_with_mixed_operations() {
        let mut skiplist =
            FreeRangeSkipList::<1024>::new(10, AllocationStrategy::WithMemoryRecycling);

        // Insert ranges in mixed order
        let range1 = create_test_range(100, 2, 200); // Furthest initially
        let range2 = create_test_range(50, 1, 100);
        let range3 = create_test_range(25, 0, 50);

        skiplist.insert(range2);
        skiplist.insert(range1);
        skiplist.insert(range3);

        assert_eq!(skiplist.get_furthest_free_range(), Some(range1));

        // Remove some non-furthest ranges
        assert!(skiplist.remove(&range3));
        assert_eq!(skiplist.get_furthest_free_range(), Some(range1));

        // Insert new furthest range
        let range4 = create_test_range(75, 3, 0);
        skiplist.insert(range4);
        assert_eq!(skiplist.get_furthest_free_range(), Some(range4));

        // Remove current furthest
        assert!(skiplist.remove(&range4));
        assert_eq!(skiplist.get_furthest_free_range(), Some(range1));
    }

    #[test]
    fn test_append_only_mode_basic() {
        let mut skiplist = FreeRangeSkipList::<1024>::new(10, AllocationStrategy::AppendOnly);

        let range1 = create_test_range(100, 0, 0);
        let range2 = create_test_range(200, 1, 0);

        skiplist.insert(range1);
        skiplist.insert(range2);

        // In AppendOnly mode, find_best_fit only checks current_furthest
        assert_eq!(skiplist.find_best_fit(150), Some(range2)); // range2 is furthest
        assert_eq!(skiplist.find_best_fit(50), Some(range2)); // Still range2

        // Can't find anything bigger than current furthest
        assert_eq!(skiplist.find_best_fit(250), None);
    }

    #[test]
    fn test_append_only_allocate_from_furthest() {
        let mut skiplist = FreeRangeSkipList::<1024>::new(10, AllocationStrategy::AppendOnly);

        let range = create_test_range(1000, 2, 100);
        skiplist.insert(range);

        // Allocate from furthest range
        let allocated = skiplist.allocate_from_furthest(300);
        assert!(allocated.is_some());
        let allocated_range = allocated.unwrap();
        assert_eq!(allocated_range.length, 300);
        assert_eq!(allocated_range.chunk_index, 2);
        assert_eq!(allocated_range.offset, 100);

        // Check that remainder is now the current furthest
        let remaining = skiplist.find_best_fit(500);
        assert!(remaining.is_some());
        let remaining_range = remaining.unwrap();
        assert_eq!(remaining_range.length, 700); // 1000 - 300
        assert_eq!(remaining_range.chunk_index, 2);
        assert_eq!(remaining_range.offset, 400); // 100 + 300
    }

    #[test]
    fn test_append_only_set_new_furthest() {
        let mut skiplist = FreeRangeSkipList::<1024>::new(10, AllocationStrategy::AppendOnly);

        let range1 = create_test_range(500, 1, 0);
        skiplist.insert(range1);

        // Set new furthest (simulating new chunk allocation)
        let new_range = create_test_range(1000, 2, 0);
        skiplist.set_new_furthest(new_range);

        // Should now use the new furthest
        assert_eq!(skiplist.find_best_fit(800), Some(new_range));

        // Old range should be in pending queue (not directly accessible)
        assert_eq!(skiplist.pending_ranges.len(), 1);
        assert_eq!(skiplist.pending_ranges[0], range1);
    }

    #[test]
    fn test_mode_switching_drains_queue() {
        let mut skiplist = FreeRangeSkipList::<1024>::new(10, AllocationStrategy::AppendOnly);

        // Add ranges in AppendOnly mode
        let range1 = create_test_range(100, 0, 0);
        let range2 = create_test_range(200, 1, 0);
        let range3 = create_test_range(300, 2, 0);

        skiplist.insert(range1);
        skiplist.insert(range2);
        skiplist.set_new_furthest(range3);

        // Should have 2 ranges in pending queue and 1 as current_furthest
        assert_eq!(skiplist.pending_ranges.len(), 2);
        assert_eq!(skiplist.current_furthest, Some(range3));

        // Switch to WithMemoryRecycling mode
        skiplist.change_strategy(AllocationStrategy::WithMemoryRecycling);

        // Queue should be drained and all ranges should be in skiplist
        assert_eq!(skiplist.pending_ranges.len(), 0);
        assert_eq!(skiplist.current_furthest, None);
        assert_eq!(skiplist.size(), 3);

        // Should now be able to find best fit using full skiplist
        assert_eq!(skiplist.find_best_fit(100), Some(range1)); // Smallest that fits
        assert_eq!(skiplist.find_best_fit(250), Some(range3)); // Best fit for 250
    }

    #[test]
    fn test_append_only_performance_characteristics() {
        let mut skiplist = FreeRangeSkipList::<1024>::new(10, AllocationStrategy::AppendOnly);

        // Insert many ranges - all should go to queue
        for i in 0..100 {
            let range = create_test_range(100 + i, (i % 10) as u16, i * 100);
            skiplist.insert(range);
        }

        // Should have many ranges in pending queue
        assert!(skiplist.pending_ranges.len() > 50);

        // find_best_fit should only consider current_furthest (O(1))
        let result = skiplist.find_best_fit(50);
        assert!(result.is_some());

        // allocate_from_furthest should be O(1)
        let allocated = skiplist.allocate_from_furthest(100);
        assert!(allocated.is_some());
    }
}
