use std::fs::File;
use std::io::{self, Write, BufWriter};
use std::path::Path;

use crate::hnsw::HnswIndex;
use super::format::{FileHeader, MAGIC_TAPE, MAGIC_VECTORS, MAGIC_META, MAGIC_OFFSETS};

pub struct IndexWriter;

impl IndexWriter {
    /// Serialize an HNSW index to disk files
    pub fn write<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>(
        index: &HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>,
        base_path: &Path,
    ) -> io::Result<()>
    where
        K: Copy + crate::node::KeySerialization,
    {
        // Create directory if needed
        std::fs::create_dir_all(base_path)?;

        // Write graph tape
        Self::write_graph_tape(index, base_path)?;

        // Write vector tape
        Self::write_vector_tape(index, base_path)?;

        // Write offsets
        Self::write_offsets(index, base_path)?;

        // Write metadata
        Self::write_metadata(index, base_path)?;

        Ok(())
    }

    fn write_graph_tape<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>(
        index: &HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>,
        base_path: &Path,
    ) -> io::Result<()>
    where
        K: Copy + crate::node::KeySerialization,
    {
        let path = base_path.join("main.tape");
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        let header = FileHeader::new(MAGIC_TAPE);
        let mut header_buf = [0u8; 16];
        header.write_to(&mut header_buf[0..6]);
        // Write chunk count at offset 8
        let chunk_count = index.graph_chunks_count() as u64;
        header_buf[8..16].copy_from_slice(&chunk_count.to_le_bytes());
        writer.write_all(&header_buf)?;

        // Write all chunks sequentially
        for chunk_idx in 0..chunk_count as usize {
            let chunk = index.get_graph_chunk(chunk_idx)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            writer.write_all(chunk.as_ref())?;
        }

        writer.flush()?;
        Ok(())
    }

    fn write_vector_tape<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>(
        index: &HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>,
        base_path: &Path,
    ) -> io::Result<()>
    where
        K: Copy + crate::node::KeySerialization,
    {
        let path = base_path.join("main.vectors");
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        let header = FileHeader::new(MAGIC_VECTORS);
        let mut header_buf = [0u8; 16];
        header.write_to(&mut header_buf[0..6]);
        // Write chunk count at offset 8
        let chunk_count = index.vector_chunks_count() as u64;
        header_buf[8..16].copy_from_slice(&chunk_count.to_le_bytes());
        writer.write_all(&header_buf)?;

        // Write all chunks sequentially
        for chunk_idx in 0..chunk_count as usize {
            let chunk = index.get_vector_chunk(chunk_idx)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            // chunk is &[f32; VECTOR_CHUNK_SIZE], write as bytes
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    chunk.as_ptr() as *const u8,
                    chunk.len() * std::mem::size_of::<f32>(),
                )
            };
            writer.write_all(bytes)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn write_offsets<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>(
        index: &HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>,
        base_path: &Path,
    ) -> io::Result<()>
    where
        K: Copy + crate::node::KeySerialization,
    {
        let path = base_path.join("main.offsets");
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        let header = FileHeader::new(MAGIC_OFFSETS);
        let mut header_buf = [0u8; 16];
        header.write_to(&mut header_buf[0..6]);

        // Get vector offsets from index
        let offsets = index.get_vector_offsets()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Write offset count at offset 8
        let count = offsets.len() as u64;
        header_buf[8..16].copy_from_slice(&count.to_le_bytes());
        writer.write_all(&header_buf)?;

        // Write all offsets as usize (8 bytes each in little-endian)
        for offset in offsets {
            writer.write_all(&offset.to_le_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    fn write_metadata<K, const DIM: usize, const GRAPH_CHUNK_SIZE: usize, const VECTOR_CHUNK_SIZE: usize>(
        index: &HnswIndex<K, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>,
        base_path: &Path,
    ) -> io::Result<()>
    where
        K: Copy + crate::node::KeySerialization,
    {
        let path = base_path.join("main.meta");
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Write header (16 bytes, with zeros for count field)
        let header = FileHeader::new(MAGIC_META);
        let mut header_buf = [0u8; 16];
        header.write_to(&mut header_buf[0..6]);
        writer.write_all(&header_buf)?;

        // Collect metadata from index
        let metadata = index.get_metadata()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Write metadata
        let mut meta_buf = Vec::new();
        metadata.write_to(&mut meta_buf);
        writer.write_all(&meta_buf)?;

        writer.flush()?;
        Ok(())
    }
}
