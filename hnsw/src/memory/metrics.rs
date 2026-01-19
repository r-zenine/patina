#[derive(Debug)]
pub struct FragmentationMetrics {
    pub chunk_free_bytes: Vec<usize>,
}

#[allow(unused)]
impl FragmentationMetrics {
    pub fn new(num_chunks: usize) -> Self {
        Self {
            chunk_free_bytes: vec![0; num_chunks],
        }
    }

    pub fn chunk_fragmentation_ratio<const CHUNK_SIZE: usize>(&self, chunk_index: usize) -> f32 {
        if chunk_index >= self.chunk_free_bytes.len() {
            return 0.0;
        }
        let free_bytes = self.chunk_free_bytes[chunk_index];
        if CHUNK_SIZE == 0 {
            0.0
        } else {
            1.0 - (free_bytes as f32 / CHUNK_SIZE as f32)
        }
    }

    pub fn total_free_bytes(&self) -> usize {
        self.chunk_free_bytes.iter().sum()
    }
}
