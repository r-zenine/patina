use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Key is required but not set")]
    MissingKey,
    #[error("Too many neighbors at level {level}: {count} (max: {max})")]
    TooManyNeighbors { level: u8, count: usize, max: usize },
    #[error("Write error: {0}")]
    WriteError(#[from] WriteError),
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("Too many neighbors for level {level}: {count} (max: {max})")]
    TooManyNeighbors { level: u8, count: usize, max: usize },
}

const NODE_ID_SIZE: usize = 5;
const _: () = assert!(NODE_ID_SIZE == std::mem::size_of::<NodeId>());
const NEIGHBORS_COUNT_SIZE: usize = 1;

use crate::{EMPTY_NEIGHBOR, NodeId};

// Helper trait for types that can be serialized to bytes
pub trait KeySerialization {
    fn write_to_vec(&self, buffer: &mut Vec<u8>);
}

impl KeySerialization for u32 {
    fn write_to_vec(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

impl KeySerialization for u64 {
    fn write_to_vec(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

// Flexible NodeBuilder for user-friendly construction
pub struct NodeBuilder<K: Copy + Sized> {
    key: Option<K>,
    levels: Vec<Vec<NodeId>>, // levels[i] = neighbors at level i (active neighbors only)
    max_level: usize,
    _phantom: PhantomData<K>,
    max_connections_level0: usize,
    max_connections: usize,
}

impl<K: Copy + Sized> NodeBuilder<K> {
    pub fn new(max_connections_level0: usize, max_connections: usize) -> Self {
        Self {
            key: None,
            levels: Vec::new(),
            max_level: 0,
            _phantom: PhantomData,
            max_connections_level0,
            max_connections,
        }
    }

    #[inline(always)]
    fn level_capacity(&self, level: usize) -> usize {
        if level == 0 {
            self.max_connections_level0
        } else {
            self.max_connections
        }
    }

    fn ensure_level_exists(&mut self, level: usize) {
        while self.levels.len() <= level {
            self.levels.push(Vec::new());
        }
    }

    fn normalize_neighbors(
        &self,
        level: usize,
        neighbors: Vec<NodeId>,
    ) -> Result<Vec<NodeId>, BuildError> {
        let capacity = self.level_capacity(level);
        let mut normalized = Vec::with_capacity(neighbors.len().min(capacity));

        for neighbor in neighbors.into_iter() {
            if neighbor == EMPTY_NEIGHBOR {
                continue;
            }

            if normalized.contains(&neighbor) {
                continue;
            }

            if normalized.len() >= capacity {
                return Err(BuildError::TooManyNeighbors {
                    level: level as u8,
                    count: normalized.len() + 1,
                    max: capacity,
                });
            }

            normalized.push(neighbor);
        }

        Ok(normalized)
    }

    pub fn with_key(mut self, key: K) -> Self {
        self.key = Some(key);
        self
    }

    pub fn set_neighbors_at_level(
        &mut self,
        level: usize,
        neighbors: Vec<NodeId>,
    ) -> Result<(), BuildError> {
        let normalized = self.normalize_neighbors(level, neighbors)?;

        self.ensure_level_exists(level);
        self.levels[level] = normalized;
        self.max_level = self.max_level.max(level);
        Ok(())
    }

    /// Create a NodeBuilder with HNSW-style empty neighbor slots
    /// This pre-allocates EMPTY_NEIGHBOR placeholders for all levels
    pub fn with_hnsw_levels(
        key: K,
        level: u8,
        max_connections_level0: usize,
        max_connections: usize,
    ) -> Result<Self, BuildError> {
        use crate::EMPTY_NEIGHBOR;

        let mut builder = Self::new(max_connections_level0, max_connections).with_key(key);

        for lvl in 0..=level {
            let max_conns = if lvl == 0 {
                max_connections_level0
            } else {
                max_connections
            };

            let empty_neighbors = vec![EMPTY_NEIGHBOR; max_conns];
            builder.set_neighbors_at_level(lvl as usize, empty_neighbors)?;
        }

        Ok(builder)
    }

    // Convert to wire format using NodeWriter
    pub fn to_bytes(&self) -> Result<Vec<u8>, BuildError>
    where
        K: KeySerialization,
    {
        let key = self.key.as_ref().ok_or(BuildError::MissingKey)?;

        let mut writer = NodeWriter::new(
            self.max_connections_level0,
            self.max_connections,
            self.max_level,
        )
        .write_key(key)?;

        for (level, level_neighbors) in self.levels.iter().enumerate() {
            writer = writer.write_level(level as u8, level_neighbors)?;
        }

        Ok(writer.finish())
    }
}

#[cfg(test)]
impl<K: Copy + Sized> NodeBuilder<K> {
    pub fn add_neighbor(&mut self, level: usize, neighbor: NodeId) -> Result<(), BuildError> {
        self.ensure_level_exists(level);

        let capacity = self.level_capacity(level);

        // Check for duplicates
        if !self.levels[level].contains(&neighbor) {
            if self.levels[level].len() >= capacity {
                return Err(BuildError::TooManyNeighbors {
                    level: level as u8,
                    count: self.levels[level].len() + 1,
                    max: capacity,
                });
            }
            self.levels[level].push(neighbor);
            self.max_level = self.max_level.max(level);
        }

        Ok(())
    }

    pub fn remove_neighbor(&mut self, level: usize, neighbor: NodeId) -> bool {
        if level < self.levels.len() {
            if let Some(pos) = self.levels[level].iter().position(|&n| n == neighbor) {
                self.levels[level].remove(pos);
                return true;
            }
        }
        false
    }

    pub fn get_neighbors_at_level(&self, level: usize) -> Option<&[NodeId]> {
        self.levels.get(level).map(|v| v.as_slice())
    }

    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

// Type-safe NodeWriter for wire format serialization
pub trait WriterState {}

pub struct KeyToWrite;
impl WriterState for KeyToWrite {}

pub struct KeyWritten;
impl WriterState for KeyWritten {}

pub struct NodeWriter<S: WriterState> {
    buffer: Vec<u8>,
    max_connections: usize,
    max_connections_level0: usize,
    _phantom: PhantomData<S>,
}

impl<S: WriterState> NodeWriter<S> {
    fn level_capacity(&self, level: u8) -> usize {
        if level == 0 {
            self.max_connections_level0
        } else {
            self.max_connections
        }
    }
}

impl NodeWriter<KeyToWrite> {
    pub fn new(max_connections_level0: usize, max_connections: usize, max_levels: usize) -> Self {
        let node_size = NODE_ID_SIZE
            + (NODE_ID_SIZE * max_connections + 1) * max_levels
            + (NODE_ID_SIZE * max_connections_level0 + 1);
        Self {
            buffer: Vec::with_capacity(node_size),
            _phantom: PhantomData,
            max_connections,
            max_connections_level0,
        }
    }

    pub fn write_key<K: KeySerialization>(
        mut self,
        key: &K,
    ) -> Result<NodeWriter<KeyWritten>, WriteError> {
        key.write_to_vec(&mut self.buffer);
        Ok(NodeWriter {
            buffer: self.buffer,
            _phantom: PhantomData,
            max_connections: self.max_connections,
            max_connections_level0: self.max_connections_level0,
        })
    }
}

impl NodeWriter<KeyWritten> {
    pub fn write_level(mut self, level: u8, neighbors: &[NodeId]) -> Result<Self, WriteError> {
        let expected = self.level_capacity(level);

        if expected > u8::MAX as usize {
            return Err(WriteError::TooManyNeighbors {
                level,
                count: expected,
                max: u8::MAX as usize,
            });
        }

        let active_count = neighbors.iter().filter(|n| **n != EMPTY_NEIGHBOR).count();

        if active_count > expected {
            return Err(WriteError::TooManyNeighbors {
                level,
                count: active_count,
                max: expected,
            });
        }

        // Write number of active neighbors
        self.buffer.push(active_count as u8);

        for neighbor in neighbors.iter().filter(|n| **n != EMPTY_NEIGHBOR) {
            self.buffer.extend_from_slice(neighbor.as_bytes());
        }

        for _ in active_count..expected {
            self.buffer.extend_from_slice(EMPTY_NEIGHBOR.as_bytes());
        }

        Ok(self)
    }

    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

pub fn node_parser<'a, K: Copy + Sized>(
    bytes: &'a [u8],
    max_connections_level0: usize,
    max_connections: usize,
) -> NodeParser<'a, K, KeyToRead> {
    NodeParser {
        remaining: bytes,
        next_level: 0,
        consumed: 0,
        max_connections_level0,
        max_connections,
        _phantom: PhantomData,
    }
}

pub trait ParserState {}

pub struct KeyToRead;
impl ParserState for KeyToRead {}

pub struct LevelsToRead;
impl ParserState for LevelsToRead {}

type LevelReadResult<'a, K> = (
    Option<(u8, &'a [NodeId], usize, usize, usize)>,
    Option<NodeParser<'a, K, LevelsToRead>>,
    usize,
);

pub struct NodeParser<'a, K, ReaderState: ParserState> {
    remaining: &'a [u8],
    next_level: u8,
    consumed: usize,
    max_connections_level0: usize,
    max_connections: usize,
    _phantom: PhantomData<(K, ReaderState)>,
}

impl<'a, K, ReaderState: ParserState> NodeParser<'a, K, ReaderState> {
    fn level_size(&self, level: u8) -> usize {
        NEIGHBORS_COUNT_SIZE + self.level_capacity(level) * NODE_ID_SIZE
    }

    pub fn level_offset(&self, target_level: u8) -> Option<usize> {
        if target_level < self.next_level {
            return None;
        }

        let mut offset = self.consumed;
        let mut level = self.next_level;
        let mut bytes = self.remaining;

        while level < target_level {
            let size = self.level_size(level);
            if bytes.len() < size {
                return None;
            }
            bytes = &bytes[size..];
            offset += size;
            level = level.saturating_add(1);
        }

        if bytes.len() < self.level_size(target_level) {
            return None;
        }

        Some(offset)
    }

    pub fn level_bytes(&self, target_level: u8) -> Option<&'a [u8]> {
        if target_level < self.next_level {
            return None;
        }

        let mut level = self.next_level;
        let mut bytes = self.remaining;

        while level < target_level {
            let size = self.level_size(level);
            if bytes.len() < size {
                return None;
            }
            bytes = &bytes[size..];
            level = level.saturating_add(1);
        }

        let size = self.level_size(target_level);
        if bytes.len() < size {
            return None;
        }

        Some(&bytes[..size])
    }

    pub fn seek_to_level(self, target_level: u8) -> Option<Self> {
        if target_level < self.next_level {
            return None;
        }

        let mut bytes = self.remaining;
        let mut consumed = self.consumed;
        let mut level = self.next_level;

        while level < target_level {
            let size = self.level_size(level);
            if bytes.len() < size {
                return None;
            }
            bytes = &bytes[size..];
            consumed += size;
            level = level.saturating_add(1);
        }

        Some(NodeParser {
            remaining: bytes,
            next_level: level,
            consumed,
            max_connections_level0: self.max_connections_level0,
            max_connections: self.max_connections,
            _phantom: PhantomData,
        })
    }

    fn level_capacity(&self, level: u8) -> usize {
        if level == 0 {
            self.max_connections_level0
        } else {
            self.max_connections
        }
    }
}

// Initial state
impl<'a, K: Copy + Sized> NodeParser<'a, K, KeyToRead> {
    pub fn read_key(self) -> (K, NodeParser<'a, K, LevelsToRead>, usize) {
        let size = std::mem::size_of::<K>();
        let key = unsafe { std::ptr::read_unaligned(self.remaining.as_ptr() as *const K) };

        (
            key,
            NodeParser {
                remaining: &self.remaining[size..],
                next_level: 0,
                consumed: 0,
                max_connections_level0: self.max_connections_level0,
                max_connections: self.max_connections,
                _phantom: PhantomData,
            },
            size, // Return bytes consumed
        )
    }
}

impl<'a, K: Copy + Sized> NodeParser<'a, K, LevelsToRead> {
    fn read_level_internal(self) -> LevelReadResult<'a, K> {
        if self.remaining.is_empty() {
            return (None, None, 0);
        }

        if self.remaining.len() < NEIGHBORS_COUNT_SIZE {
            return (None, None, 0);
        }

        let level = self.next_level;
        let active_count = self.remaining[0] as usize;
        let expected = self.level_capacity(level);

        if active_count > expected {
            return (None, None, 0);
        }

        let remaining_after_count = &self.remaining[NEIGHBORS_COUNT_SIZE..];
        let neighbors_size = expected * NODE_ID_SIZE;

        if remaining_after_count.len() < neighbors_size {
            return (None, None, 0);
        }

        let neighbors = unsafe {
            std::slice::from_raw_parts(remaining_after_count.as_ptr() as *const NodeId, expected)
        };
        let active_neighbors = &neighbors[..active_count];

        let remaining_after_neighbors = &remaining_after_count[neighbors_size..];
        let level_start_offset = self.consumed;
        let total_consumed = NEIGHBORS_COUNT_SIZE + neighbors_size;

        let next_parser = if remaining_after_neighbors.is_empty() {
            None
        } else {
            Some(NodeParser {
                remaining: remaining_after_neighbors,
                next_level: level.saturating_add(1),
                consumed: self.consumed + total_consumed,
                max_connections_level0: self.max_connections_level0,
                max_connections: self.max_connections,
                _phantom: PhantomData,
            })
        };

        (
            Some((
                level,
                active_neighbors,
                active_count,
                expected,
                level_start_offset,
            )),
            next_parser,
            total_consumed,
        )
    }

    #[allow(clippy::type_complexity)]
    pub fn read_level(
        self,
    ) -> (
        Option<(u8, &'a [NodeId])>,
        Option<NodeParser<'a, K, LevelsToRead>>,
        usize,
    ) {
        let (level_data, next_parser, consumed) = self.read_level_internal();
        (
            level_data.map(|(level, neighbors, _, _, _)| (level, neighbors)),
            next_parser,
            consumed,
        )
    }

    #[allow(clippy::type_complexity)]
    pub fn read_level_with_counts(
        self,
    ) -> (
        Option<(u8, Vec<NodeId>, usize)>,
        Option<NodeParser<'a, K, LevelsToRead>>,
        usize,
    ) {
        let (level_data, next_parser, consumed) = self.read_level_internal();

        if let Some((level, neighbors, active_count, capacity, _offset)) = level_data {
            let mut active_neighbors = Vec::with_capacity(active_count);

            for &neighbor in neighbors.iter().take(active_count) {
                active_neighbors.push(neighbor);
            }

            let empty_count = capacity.saturating_sub(active_count);

            (
                Some((level, active_neighbors, empty_count)),
                next_parser,
                consumed,
            )
        } else {
            (None, next_parser, consumed)
        }
    }

    pub fn skip_level(self) -> Option<NodeParser<'a, K, LevelsToRead>> {
        let size = self.level_size(self.next_level);
        if self.remaining.len() < size {
            return None;
        }

        let remaining_after = &self.remaining[size..];

        if remaining_after.is_empty() {
            None
        } else {
            Some(NodeParser {
                remaining: remaining_after,
                next_level: self.next_level.saturating_add(1),
                consumed: self.consumed + size,
                max_connections_level0: self.max_connections_level0,
                max_connections: self.max_connections,
                _phantom: PhantomData,
            })
        }
    }
}

impl<'a, K> fmt::Debug for NodeParser<'a, K, KeyToRead> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::format_utils::format_hex_preview;
        write!(
            f,
            "NodeParser {{ state: KeyToRead, remaining: {} bytes, expected: {} bytes for key type, preview: {} }}",
            self.remaining.len(),
            std::mem::size_of::<K>(),
            format_hex_preview(self.remaining, 8)
        )
    }
}

impl<'a, K> fmt::Debug for NodeParser<'a, K, LevelsToRead> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::format_utils::format_hex_preview;

        if self.remaining.is_empty() {
            write!(
                f,
                "NodeParser {{ state: LevelsToRead, remaining: 0 bytes, status: exhausted }}"
            )
        } else if self.remaining.len() < NEIGHBORS_COUNT_SIZE {
            write!(
                f,
                "NodeParser {{ state: LevelsToRead, remaining: {} bytes, status: insufficient for level header, preview: {} }}",
                self.remaining.len(),
                format_hex_preview(self.remaining, 4)
            )
        } else {
            let level = self.next_level;
            let active = self.remaining[0];
            let capacity = self.level_capacity(level);
            let needed = NEIGHBORS_COUNT_SIZE + capacity * NODE_ID_SIZE;
            write!(
                f,
                "NodeParser {{ state: LevelsToRead, level: {}, active: {}, capacity: {}, remaining: {} bytes, needed: {} bytes, preview: {} }}",
                level,
                active,
                capacity,
                self.remaining.len(),
                needed,
                format_hex_preview(self.remaining, 12)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::node::*;

    const TEST_MAX_CONNECTIONS_LEVEL0: usize = 2;
    const TEST_MAX_CONNECTIONS: usize = 1;

    fn create_test_node_data() -> Vec<u8> {
        let mut builder = NodeBuilder::new(TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS)
            .with_key(0x123456789ABCDEF0u64);

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
        builder.set_neighbors_at_level(2, Vec::new()).unwrap();

        builder.to_bytes().unwrap()
    }

    fn create_empty_node_data() -> Vec<u8> {
        let mut builder =
            NodeBuilder::new(TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS).with_key(42u32);
        builder.set_neighbors_at_level(0, Vec::new()).unwrap();
        builder.to_bytes().unwrap()
    }

    #[test]
    fn test_read_key_u64() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (key, _, bytes_consumed) = parser.read_key();
        assert_eq!(key, 0x123456789ABCDEF0u64);
        assert_eq!(bytes_consumed, 8); // u64 is 8 bytes
    }

    #[test]
    fn test_read_key_u32() {
        let data = create_empty_node_data();
        let parser = node_parser::<u32>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (key, _, bytes_consumed) = parser.read_key();
        assert_eq!(key, 42u32);
        assert_eq!(bytes_consumed, 4); // u32 is 4 bytes
    }

    #[test]
    fn test_read_single_level_with_neighbors() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (key, parser, _) = parser.read_key();
        assert_eq!(key, 0x123456789ABCDEF0u64);

        let (level_data, parser, _level_bytes) = parser.read_level();
        assert!(level_data.is_some());
        assert!(parser.is_some());

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(neighbors.len(), 2);

        // Verify neighbor data
        let neighbor1_bytes = neighbors[0].as_bytes();
        let neighbor2_bytes = neighbors[1].as_bytes();

        assert_eq!(neighbor1_bytes, &[0x01, 0x02, 0x03, 0x04, 0x05]);
        assert_eq!(neighbor2_bytes, &[0x06, 0x07, 0x08, 0x09, 0x0A]);
    }

    #[test]
    fn test_read_multiple_levels() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (_, mut parser, _) = parser.read_key();

        // Read level 0
        let (level_data, next_parser, _) = parser.read_level();
        assert!(level_data.is_some());
        assert!(next_parser.is_some());

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(neighbors.len(), 2);

        parser = next_parser.unwrap();

        // Read level 1
        let (level_data, next_parser, _) = parser.read_level();
        assert!(level_data.is_some());
        assert!(next_parser.is_some());

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(neighbors.len(), 1);

        let neighbor_bytes = neighbors[0].as_bytes();
        assert_eq!(neighbor_bytes, &[0x0B, 0x0C, 0x0D, 0x0E, 0x0F]);

        parser = next_parser.unwrap();

        // Read level 2 (empty)
        let (level_data, next_parser, _) = parser.read_level();
        assert!(level_data.is_some());
        assert!(next_parser.is_none()); // No more levels

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 2);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn test_level_offset_and_bytes_helpers() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);
        let (_, parser, _) = parser.read_key();

        assert_eq!(parser.level_offset(0), Some(0));
        assert_eq!(
            parser.level_offset(1),
            Some(NEIGHBORS_COUNT_SIZE + TEST_MAX_CONNECTIONS_LEVEL0 * NODE_ID_SIZE)
        );

        let level_one_bytes = parser.level_bytes(1).expect("level bytes");
        assert_eq!(
            level_one_bytes.len(),
            NEIGHBORS_COUNT_SIZE + TEST_MAX_CONNECTIONS * NODE_ID_SIZE
        );

        let level_two_parser = parser.seek_to_level(2).expect("seek to level 2");
        let (level_data, _, _) = level_two_parser.read_level();
        assert!(level_data.is_some());
        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 2);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn test_read_empty_level() {
        let data = create_empty_node_data();
        let parser = node_parser::<u32>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (_, parser, _) = parser.read_key();
        let (level_data, next_parser, _) = parser.read_level();

        assert!(level_data.is_some());
        assert!(next_parser.is_none());

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn test_skip_level() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (_, parser, _) = parser.read_key();

        // Skip level 0
        let parser = parser.skip_level();
        assert!(parser.is_some());

        let parser = parser.unwrap();

        // Read level 1 directly
        let (level_data, _, _) = parser.read_level();
        assert!(level_data.is_some());

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(neighbors.len(), 1);
    }

    #[test]
    fn test_skip_multiple_levels() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (_, parser, _) = parser.read_key();

        // Skip level 0
        let parser = parser.skip_level().unwrap();
        // Skip level 1
        let parser = parser.skip_level().unwrap();

        // Read level 2 directly
        let (level_data, next_parser, _) = parser.read_level();
        assert!(level_data.is_some());
        assert!(next_parser.is_none());

        let (level, neighbors) = level_data.unwrap();
        assert_eq!(level, 2);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn test_parser_exhaustion() {
        let data = create_empty_node_data();
        let parser = node_parser::<u32>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (_, parser, _) = parser.read_key();
        let (_, next_parser, _) = parser.read_level();

        // Should be None - no more data
        assert!(next_parser.is_none());
    }

    #[test]
    fn test_insufficient_level_data() {
        let mut data = Vec::new();
        data.extend_from_slice(&42u32.to_le_bytes()); // Valid key
        data.push(0); // Level byte
        // Missing count byte - should be handled gracefully

        let parser = node_parser::<u32>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);
        let (_, parser, _) = parser.read_key();
        let (level_data, next_parser, _) = parser.read_level();

        assert!(level_data.is_none());
        assert!(next_parser.is_none());
    }

    #[test]
    fn test_node_id_conversion() {
        // Test that NodeId can be created and read properly
        let test_bytes = [0x01, 0x02, 0x03, 0x04, 0x05];
        let node_id = NodeId(test_bytes);

        assert_eq!(node_id.as_bytes(), &test_bytes);
    }

    #[test]
    fn test_compile_time_size_assertion() {
        // This test ensures our compile-time assertion works
        assert_eq!(std::mem::size_of::<NodeId>(), 5);
    }

    #[test]
    fn test_full_iteration_pattern() {
        let data = create_test_node_data();
        let parser = node_parser::<u64>(&data, TEST_MAX_CONNECTIONS_LEVEL0, TEST_MAX_CONNECTIONS);

        let (key, mut parser, _) = parser.read_key();
        assert_eq!(key, 0x123456789ABCDEF0u64);

        let mut level_count = 0;
        let mut total_neighbors = 0;

        while let (Some((level, active_neighbors, empty_count)), next, _) =
            parser.read_level_with_counts()
        {
            println!(
                "Level {}: {} neighbors ({} empty)",
                level,
                active_neighbors.len(),
                empty_count
            );
            assert_eq!(level as usize, level_count);
            total_neighbors += active_neighbors.len();
            level_count += 1;

            if let Some(p) = next {
                parser = p;
            } else {
                break;
            }
        }

        assert_eq!(level_count, 3); // 3 levels total
        assert_eq!(total_neighbors, 3); // 2 + 1 + 0 neighbors
    }

    // Round-trip tests for NodeBuilder/NodeWriter/NodeParser compatibility
    #[test]
    fn test_node_builder_writer_parser_roundtrip_u64() {
        let original_key = 0x123456789ABCDEF0u64;
        let neighbors_level_0 = vec![
            NodeId([0x01, 0x02, 0x03, 0x04, 0x05]),
            NodeId([0x06, 0x07, 0x08, 0x09, 0x0A]),
        ];
        let neighbors_level_1 = vec![NodeId([0x0B, 0x0C, 0x0D, 0x0E, 0x0F])];
        let neighbors_level_2: Vec<NodeId> = vec![]; // Empty level

        // Build with NodeBuilder
        let mut builder = NodeBuilder::new(5, 5).with_key(original_key);

        builder
            .set_neighbors_at_level(0, neighbors_level_0.clone())
            .unwrap();
        builder
            .set_neighbors_at_level(1, neighbors_level_1.clone())
            .unwrap();
        builder
            .set_neighbors_at_level(2, neighbors_level_2.clone())
            .unwrap();

        let wire_data = builder.to_bytes().unwrap();

        // Parse with NodeParser
        let parser = node_parser::<u64>(&wire_data, 5, 5);
        let (parsed_key, mut parser, _) = parser.read_key();

        // Verify key
        assert_eq!(parsed_key, original_key);

        // Parse level 0
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), neighbors_level_0.len());
        assert_eq!(empty_count, 5 - neighbors_level_0.len());
        assert_eq!(active_neighbors, neighbors_level_0);
        parser = next_parser.unwrap();

        // Parse level 1
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(active_neighbors.len(), neighbors_level_1.len());
        assert_eq!(empty_count, 5 - neighbors_level_1.len());
        assert_eq!(active_neighbors, neighbors_level_1);
        parser = next_parser.unwrap();

        // Parse level 2 (empty)
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 2);
        assert!(active_neighbors.is_empty());
        assert_eq!(empty_count, 5);
        assert!(next_parser.is_none()); // No more levels
    }

    #[test]
    fn test_node_builder_writer_parser_roundtrip_u32() {
        let original_key = 42u32;
        let neighbors_level_0 = [NodeId([0x01, 0x02, 0x03, 0x04, 0x05])];

        // Build with NodeBuilder
        let mut builder = NodeBuilder::new(5, 5).with_key(original_key);
        builder.add_neighbor(0, neighbors_level_0[0]).unwrap();

        let wire_data = builder.to_bytes().unwrap();

        // Parse with NodeParser
        let parser = node_parser::<u32>(&wire_data, 5, 5);
        let (parsed_key, parser, _) = parser.read_key();

        // Verify key
        assert_eq!(parsed_key, original_key);

        // Parse level 0
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 1);
        assert_eq!(active_neighbors[0], neighbors_level_0[0]);
        assert_eq!(empty_count, 4);
        assert!(next_parser.is_none()); // Only one level
    }

    #[test]
    fn test_node_builder_add_remove_neighbors() {
        let mut builder = NodeBuilder::<u64>::new(5, 5).with_key(123);

        let neighbor1 = NodeId([1, 2, 3, 4, 5]);
        let neighbor2 = NodeId([6, 7, 8, 9, 10]);

        // Add neighbors
        builder.add_neighbor(0, neighbor1).unwrap();
        builder.add_neighbor(0, neighbor2).unwrap();
        builder.add_neighbor(1, neighbor1).unwrap();

        assert_eq!(builder.get_neighbors_at_level(0).unwrap().len(), 2);
        assert_eq!(builder.get_neighbors_at_level(1).unwrap().len(), 1);
        assert_eq!(builder.level_count(), 2);

        // Remove neighbor
        assert!(builder.remove_neighbor(0, neighbor1));
        assert_eq!(builder.get_neighbors_at_level(0).unwrap().len(), 1);
        assert_eq!(builder.get_neighbors_at_level(0).unwrap()[0], neighbor2);

        // Try to remove non-existent neighbor
        assert!(!builder.remove_neighbor(0, neighbor1));
        assert_eq!(builder.get_neighbors_at_level(0).unwrap().len(), 1);
    }

    #[test]
    fn test_node_builder_duplicate_neighbors() {
        let mut builder = NodeBuilder::<u32>::new(3, 3).with_key(42);
        let neighbor = NodeId([1, 2, 3, 4, 5]);

        // Add same neighbor twice
        builder.add_neighbor(0, neighbor).unwrap();
        builder.add_neighbor(0, neighbor).unwrap(); // Should not add duplicate

        assert_eq!(builder.get_neighbors_at_level(0).unwrap().len(), 1);
    }

    #[test]
    fn test_node_builder_error_missing_key() {
        let builder = NodeBuilder::<u64>::new(1, 1); // No key set
        let result = builder.to_bytes();
        assert!(matches!(result, Err(BuildError::MissingKey)));
    }

    #[test]
    fn test_node_writer_direct_usage() {
        let key = 42u64;
        let neighbors_level_0 = vec![NodeId([1, 2, 3, 4, 5])];
        let neighbors_level_1 = vec![NodeId([6, 7, 8, 9, 10]), NodeId([11, 12, 13, 14, 15])];

        // Use NodeWriter directly
        let wire_data = NodeWriter::new(4, 2, 3)
            .write_key(&key)
            .unwrap()
            .write_level(0, &neighbors_level_0)
            .unwrap()
            .write_level(1, &neighbors_level_1)
            .unwrap()
            .finish();

        // Verify with NodeParser
        let parser = node_parser::<u64>(&wire_data, 4, 2);
        let (parsed_key, mut parser, _) = parser.read_key();
        assert_eq!(parsed_key, key);

        // Parse levels
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 1);
        assert_eq!(active_neighbors[0], neighbors_level_0[0]);
        assert_eq!(empty_count, 3);
        parser = next_parser.unwrap();

        let (level_data, next_parser, _) = parser.read_level_with_counts();
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(active_neighbors.len(), 2);
        assert_eq!(active_neighbors[0], neighbors_level_1[0]);
        assert_eq!(active_neighbors[1], neighbors_level_1[1]);
        assert_eq!(empty_count, 0);
        assert!(next_parser.is_none());
    }

    #[test]
    fn test_neighbor_slot_provisioning_with_empty_neighbors() {
        use crate::EMPTY_NEIGHBOR;

        // Test that nodes can be created with EMPTY_NEIGHBOR placeholders
        // and that read_level_with_counts correctly identifies them as empty slots
        let key = 42u64;
        let max_connections_level0 = 32;
        let max_connections = 16;

        let mut builder = NodeBuilder::new(max_connections_level0, max_connections).with_key(key);

        // Level 0: provision with EMPTY_NEIGHBOR placeholders (simulating HNSW node creation)
        let empty_neighbors_level0 = vec![EMPTY_NEIGHBOR; max_connections_level0];
        builder
            .set_neighbors_at_level(0, empty_neighbors_level0)
            .unwrap();

        // Level 1: provision with EMPTY_NEIGHBOR placeholders
        let empty_neighbors_level1 = vec![EMPTY_NEIGHBOR; max_connections];
        builder
            .set_neighbors_at_level(1, empty_neighbors_level1)
            .unwrap();

        let wire_data = builder.to_bytes().unwrap();

        // Parse and verify empty slots are correctly identified
        let parser = node_parser::<u64>(&wire_data, max_connections_level0, max_connections);
        let (parsed_key, mut parser, _) = parser.read_key();
        assert_eq!(parsed_key, key);

        // Check level 0 - should show all neighbors as empty
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 0); // No active neighbors
        assert_eq!(empty_count, max_connections_level0); // All slots are empty
        parser = next_parser.unwrap();

        // Check level 1 - should show all neighbors as empty
        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 1);
        assert_eq!(active_neighbors.len(), 0); // No active neighbors
        assert_eq!(empty_count, max_connections); // All slots are empty
        assert!(next_parser.is_none());
    }

    #[test]
    fn test_mixed_empty_and_active_neighbors() {
        use crate::EMPTY_NEIGHBOR;

        // Test a node with some active neighbors and some empty slots
        let key = 123u32;
        let active_neighbor1 = NodeId([1, 2, 3, 4, 5]);
        let active_neighbor2 = NodeId([6, 7, 8, 9, 10]);

        // Create a level with 2 active neighbors and 3 empty slots (total 5 slots)
        let mixed_neighbors = vec![
            active_neighbor1,
            EMPTY_NEIGHBOR,
            active_neighbor2,
            EMPTY_NEIGHBOR,
            EMPTY_NEIGHBOR,
        ];

        let mut builder = NodeBuilder::new(5, 5).with_key(key);
        builder.set_neighbors_at_level(0, mixed_neighbors).unwrap();

        let wire_data = builder.to_bytes().unwrap();

        // Parse and verify counts
        let parser = node_parser::<u32>(&wire_data, 5, 5);
        let (parsed_key, parser, _) = parser.read_key();
        assert_eq!(parsed_key, key);

        let (level_data, next_parser, _) = parser.read_level_with_counts();
        assert!(level_data.is_some());
        let (level, active_neighbors, empty_count) = level_data.unwrap();
        assert_eq!(level, 0);
        assert_eq!(active_neighbors.len(), 2); // 2 active neighbors
        assert_eq!(empty_count, 3); // 3 empty slots
        assert_eq!(active_neighbors[0], active_neighbor1);
        assert_eq!(active_neighbors[1], active_neighbor2);
        assert!(next_parser.is_none());
    }

    #[test]
    fn test_empty_neighbor_constant_value() {
        use crate::EMPTY_NEIGHBOR;

        // Verify EMPTY_NEIGHBOR is indeed all zeros
        assert_eq!(EMPTY_NEIGHBOR, NodeId([0, 0, 0, 0, 0]));

        // Test that multiple EMPTY_NEIGHBOR instances are equal
        let empty1 = EMPTY_NEIGHBOR;
        let empty2 = NodeId([0, 0, 0, 0, 0]);
        assert_eq!(empty1, empty2);
    }

    #[test]
    fn test_set_neighbors_at_level_limits() {
        // Test what happens when we try to set too many neighbors
        let mut builder = NodeBuilder::<u64>::new(255, 255).with_key(42);

        // Test with exactly u8::MAX neighbors (255) - should succeed
        let max_neighbors: Vec<NodeId> = (0..255).map(|i| NodeId::from_usize(i + 1)).collect();

        let result = builder.set_neighbors_at_level(0, max_neighbors);
        assert!(result.is_ok(), "Setting 255 neighbors should succeed");

        // Test with u8::MAX + 1 neighbors (256) - should fail
        let too_many_neighbors: Vec<NodeId> = (0..256).map(|i| NodeId::from_usize(i + 1)).collect();

        let result = builder.set_neighbors_at_level(1, too_many_neighbors);
        assert!(result.is_err(), "Setting 256 neighbors should fail");

        if let Err(BuildError::TooManyNeighbors { level, count, max }) = result {
            assert_eq!(level, 1);
            assert_eq!(count, 256);
            assert_eq!(max, 255);
        } else {
            panic!("Expected TooManyNeighbors error, got {result:?}");
        }

        // Test with HNSW typical limits (32 for level 0, 16 for other levels) - should succeed
        let level0_neighbors: Vec<NodeId> =
            (0..32).map(|i| NodeId([i as u8, 0, 0, 0, 0])).collect();
        let result = builder.set_neighbors_at_level(0, level0_neighbors);
        assert!(
            result.is_ok(),
            "Setting 32 neighbors at level 0 should succeed"
        );

        let level1_neighbors: Vec<NodeId> =
            (0..16).map(|i| NodeId([i as u8, 1, 0, 0, 0])).collect();
        let result = builder.set_neighbors_at_level(1, level1_neighbors);
        assert!(
            result.is_ok(),
            "Setting 16 neighbors at level 1 should succeed"
        );
    }
}
