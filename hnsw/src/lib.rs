mod format_utils;
mod hnsw;
mod memory;
mod node;
pub mod persistence;
mod prefetch;
mod simd_distances;
mod tape;
mod tape_mutations;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] // Add Hash for HashSet
pub struct NodeId([u8; 5]);

pub const EMPTY_NEIGHBOR: NodeId = NodeId([0, 0, 0, 0, 0]);

impl NodeId {
    pub fn is_empty(&self) -> bool {
        *self != EMPTY_NEIGHBOR
    }
}

// Re-export distance function types and selectors
pub use hnsw::{HnswIndex, HnswParams};
pub use simd_distances::{
    DistanceFn, select_best_cosine, select_best_dot_product, select_best_euclidean,
    select_best_manhattan,
};

impl NodeId {
    pub fn as_bytes(&self) -> &[u8; 5] {
        &self.0
    }

    pub fn to_usize(&self) -> usize {
        // Convert 5-byte little-endian to usize
        // Take the 5 bytes and pad with zeros for the upper bytes
        let mut bytes = [0u8; 8]; // usize is typically 8 bytes on 64-bit systems
        bytes[0..5].copy_from_slice(&self.0);
        usize::from_le_bytes(bytes)
    }

    pub fn from_usize(value: usize) -> Self {
        let bytes = value.to_le_bytes();
        let mut id = [0u8; 5];
        id.copy_from_slice(&bytes[..5]);
        NodeId(id)
    }
}
