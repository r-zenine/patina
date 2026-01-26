use std::ops::{Deref, DerefMut};

#[repr(align(64))]
#[derive(Debug)]
pub struct AlignedChunk<T, const CHUNK_SIZE: usize> {
    data: [T; CHUNK_SIZE],
}

impl<T, const CHUNK_SIZE: usize> Default for AlignedChunk<T, CHUNK_SIZE>
where
    T: Copy + Default + Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CHUNK_SIZE: usize> AlignedChunk<T, CHUNK_SIZE>
where
    T: Copy + Default + Sized,
{
    pub fn new() -> Self {
        AlignedChunk {
            data: [T::default(); CHUNK_SIZE],
        }
    }
}

impl<T, const CHUNK_SIZE: usize> Deref for AlignedChunk<T, CHUNK_SIZE> {
    type Target = [T; CHUNK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, const CHUNK_SIZE: usize> DerefMut for AlignedChunk<T, CHUNK_SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
