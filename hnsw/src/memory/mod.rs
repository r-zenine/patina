mod aligned_chunk;
mod memory_manager;
mod metrics;
mod skiplist;

pub use aligned_chunk::AlignedChunk;
pub use memory_manager::MemoryManager;

#[derive(Debug, Clone, Copy)]
pub enum AllocationStrategy {
    // Enable recycling free ranges in the tape
    // this mode is usefull when serving the graph
    // and dealing with a mixed workload that balances
    // new node insertions, node removals
    WithMemoryRecycling,
    // This is the allocation mode to use when we are
    // building the graph or doing a heavy batch ingestion
    AppendOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeRange {
    pub chunk_index: u16,
    pub offset: usize,
    pub length: usize,
}

impl FreeRange {
    fn split(self, size: usize) -> (FreeRange, Option<FreeRange>) {
        if self.length <= size {
            (self, None)
        } else {
            (
                FreeRange {
                    chunk_index: self.chunk_index,
                    offset: self.offset,
                    length: size,
                },
                Some(FreeRange {
                    chunk_index: self.chunk_index,
                    offset: self.offset + size,
                    length: self.length - size,
                }),
            )
        }
    }
}

impl PartialOrd for FreeRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FreeRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Primary: order by length for best-fit allocation
        // Secondary: order by chunk_index for merging adjacent ranges
        // Tertiary: order by offset for position within chunk
        self.length
            .cmp(&other.length)
            .then(self.chunk_index.cmp(&other.chunk_index))
            .then(self.offset.cmp(&other.offset))
    }
}
