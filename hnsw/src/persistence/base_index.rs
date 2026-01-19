use std::fs::File;
use std::io;
use std::path::Path;
use memmap2::Mmap;

use crate::NodeId;
use super::format::{FileHeader, IndexMetadata, MAGIC_META, MAGIC_OFFSETS, CHUNK_SIZE, FILE_HEADER_SIZE};

/// Read-only index backed by memory-mapped files
pub struct BaseIndex {
    graph_mmap: Mmap,
    vector_mmap: Mmap,
    offsets: Vec<usize>,
    metadata: IndexMetadata,
}

impl BaseIndex {
    /// Open and load index from disk
    pub fn open(base_path: &Path) -> io::Result<Self> {
        // Open and map graph file
        let graph_file = File::open(base_path.join("main.tape"))?;
        let graph_mmap = unsafe { Mmap::map(&graph_file)? };

        // Open and map vector file
        let vector_file = File::open(base_path.join("main.vectors"))?;
        let vector_mmap = unsafe { Mmap::map(&vector_file)? };

        // Load offsets
        let offsets_path = base_path.join("main.offsets");
        let offsets = Self::load_offsets(&offsets_path)?;

        // Load metadata
        let metadata_path = base_path.join("main.meta");
        let metadata = Self::load_metadata(&metadata_path)?;

        Ok(BaseIndex {
            graph_mmap,
            vector_mmap,
            offsets,
            metadata,
        })
    }

    fn load_offsets(path: &Path) -> io::Result<Vec<usize>> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Read header
        if mmap.len() < FILE_HEADER_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Offsets file too small"));
        }

        let header = FileHeader::read_from(&mmap[0..6])
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid offsets header"))?;

        if !header.validate(MAGIC_OFFSETS) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid offsets magic"));
        }

        // Read offset count (at offset 8)
        let count = u64::from_le_bytes([
            mmap[8], mmap[9], mmap[10], mmap[11], mmap[12], mmap[13], mmap[14], mmap[15],
        ]) as usize;

        // Read offsets
        let mut offsets = Vec::with_capacity(count);
        for i in 0..count {
            let offset_pos = FILE_HEADER_SIZE + i * 8;
            if offset_pos + 8 > mmap.len() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Offsets file truncated"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&mmap[offset_pos..offset_pos + 8]);
            offsets.push(usize::from_le_bytes(bytes));
        }

        Ok(offsets)
    }

    fn load_metadata(path: &Path) -> io::Result<IndexMetadata> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Read header
        if mmap.len() < FILE_HEADER_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Meta file too small"));
        }

        let header = FileHeader::read_from(&mmap[0..6])
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid meta header"))?;

        if !header.validate(MAGIC_META) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid meta magic"));
        }

        // Read metadata (starts at offset FILE_HEADER_SIZE)
        let metadata = IndexMetadata::read_from(&mmap[FILE_HEADER_SIZE..])
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid metadata"))?;

        Ok(metadata)
    }

    pub fn entry_point(&self) -> Option<NodeId> {
        self.metadata.entry_point
    }

    pub fn node_count(&self) -> usize {
        self.metadata.node_count
    }

    pub fn max_connections(&self) -> usize {
        self.metadata.max_connections
    }

    pub fn max_connections_level0(&self) -> usize {
        self.metadata.max_connections_level0
    }

    pub fn max_level(&self) -> u8 {
        self.metadata.max_level
    }

    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    /// Get graph chunk data
    pub fn get_graph_chunk(&self, chunk_idx: usize) -> Option<&[u8]> {
        // Account for file header
        let offset = FILE_HEADER_SIZE + chunk_idx * CHUNK_SIZE;
        if offset + CHUNK_SIZE > self.graph_mmap.len() {
            return None;
        }
        Some(&self.graph_mmap[offset..offset + CHUNK_SIZE])
    }

    /// Get vector chunk data as f32 slice
    pub fn get_vector_chunk(&self, chunk_idx: usize) -> Option<&[f32]> {
        // Account for file header
        let offset = FILE_HEADER_SIZE + chunk_idx * CHUNK_SIZE * std::mem::size_of::<f32>();
        if offset + CHUNK_SIZE * std::mem::size_of::<f32>() > self.vector_mmap.len() {
            return None;
        }

        let slice = &self.vector_mmap[offset..offset + CHUNK_SIZE * std::mem::size_of::<f32>()];
        let ptr = slice.as_ptr() as *const f32;
        unsafe {
            Some(std::slice::from_raw_parts(ptr, CHUNK_SIZE))
        }
    }

    /// Get vector for a specific node
    pub fn get_vector<const DIM: usize>(&self, node_id: &NodeId) -> Option<&[f32; DIM]> {
        let node_idx = node_id.to_usize();
        if node_idx >= self.offsets.len() {
            return None;
        }

        let offset = self.offsets[node_idx];
        let chunk_idx = offset / (CHUNK_SIZE * std::mem::size_of::<f32>());
        let elem_offset = (offset % (CHUNK_SIZE * std::mem::size_of::<f32>())) / std::mem::size_of::<f32>();

        let chunk = self.get_vector_chunk(chunk_idx)?;
        if elem_offset + DIM > chunk.len() {
            return None;
        }

        let slice = &chunk[elem_offset..elem_offset + DIM];
        let ptr = slice.as_ptr() as *const [f32; DIM];
        Some(unsafe { &*ptr })
    }

    /// Get neighbors at level from graph data
    pub fn neighbors_at_level(&self, node_id: &NodeId, level: u8) -> Option<Vec<NodeId>> {
        let node_idx = node_id.to_usize();
        if node_idx >= self.offsets.len() {
            return None;
        }

        let offset = self.offsets[node_idx];
        let chunk_idx = offset / CHUNK_SIZE;
        let chunk_offset = offset % CHUNK_SIZE;

        let chunk = self.get_graph_chunk(chunk_idx)?;

        // Read node length
        if chunk_offset + 2 > CHUNK_SIZE {
            return None;
        }
        let node_length = u16::from_le_bytes([chunk[chunk_offset], chunk[chunk_offset + 1]]) as usize;

        // Parse node to get neighbors at level
        let node_start = chunk_offset + 2;
        if node_start + node_length > CHUNK_SIZE {
            return None;
        }

        let node_bytes = &chunk[node_start..node_start + node_length];
        Self::parse_neighbors_at_level(node_bytes, level, self.metadata.max_connections_level0, self.metadata.max_connections)
    }

    fn parse_neighbors_at_level(
        node_bytes: &[u8],
        target_level: u8,
        max_connections_level0: usize,
        max_connections: usize,
    ) -> Option<Vec<NodeId>> {
        let mut offset = 0;

        // Skip key - we don't know size without knowing K type, so estimate
        // For now, assume keys are u32 or u64 (4 or 8 bytes)
        // This is a limitation - ideally we'd pass K type information
        // For now, hard-code assuming u32 keys (4 bytes)
        offset += 4; // Key size

        let mut current_level = 0u8;

        // Find the target level
        loop {
            if offset >= node_bytes.len() {
                return None;
            }

            // Read active neighbor count
            let active_count = node_bytes[offset] as usize;
            offset += 1;

            let level_capacity = if current_level == 0 {
                max_connections_level0
            } else {
                max_connections
            };

            if current_level == target_level {
                // Found target level, collect neighbors
                let mut neighbors = Vec::new();
                for _ in 0..active_count {
                    if offset + 5 > node_bytes.len() {
                        return None;
                    }
                    let mut bytes = [0u8; 5];
                    bytes.copy_from_slice(&node_bytes[offset..offset + 5]);
                    neighbors.push(NodeId(bytes));
                    offset += 5;
                }
                return Some(neighbors);
            }

            // Skip to next level
            offset += level_capacity * 5; // capacity * NodeId size

            if current_level >= max_connections_level0 as u8 {
                return None; // Exceeded max level
            }

            current_level += 1;
        }
    }

    #[cfg(test)]
    pub fn graph_mmap_size(&self) -> usize {
        self.graph_mmap.len()
    }

    #[cfg(test)]
    pub fn vector_mmap_size(&self) -> usize {
        self.vector_mmap.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::format::MAGIC_TAPE;

    #[test]
    fn test_file_header_roundtrip() {
        let header = FileHeader::new(MAGIC_TAPE);
        let mut buf = [0u8; 6];
        header.write_to(&mut buf);

        let parsed = FileHeader::read_from(&buf).unwrap();
        assert_eq!(parsed.magic, *MAGIC_TAPE);
        assert_eq!(parsed.version, 1);
        assert!(parsed.validate(MAGIC_TAPE));
    }

    #[test]
    fn test_metadata_roundtrip() {
        let meta = IndexMetadata {
            entry_point: Some(NodeId([1, 2, 3, 4, 5])),
            node_count: 100,
            max_connections: 16,
            max_connections_level0: 32,
            max_level: 16,
        };

        let mut buf = Vec::new();
        meta.write_to(&mut buf);

        let parsed = IndexMetadata::read_from(&buf).unwrap();
        assert_eq!(parsed.entry_point, meta.entry_point);
        assert_eq!(parsed.node_count, meta.node_count);
        assert_eq!(parsed.max_connections, meta.max_connections);
        assert_eq!(parsed.max_connections_level0, meta.max_connections_level0);
        assert_eq!(parsed.max_level, meta.max_level);
    }
}
