/// Magic numbers and version constants for persistence format

pub const MAGIC_TAPE: &[u8; 4] = b"HTAP";
pub const MAGIC_VECTORS: &[u8; 4] = b"HVEC";
pub const MAGIC_META: &[u8; 4] = b"HMET";
pub const MAGIC_OFFSETS: &[u8; 4] = b"HOFF";

pub const VERSION: u16 = 1;
pub const CHUNK_SIZE: usize = 4096; // 4KB page-aligned

// File header size: magic (4) + version (2) + padding (2) + chunk_count (8) = 16 bytes (aligned for f32)
pub const FILE_HEADER_SIZE: usize = 16;

/// Header for tape, vector, and offset files
#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u16,
}

impl FileHeader {
    pub fn new(magic: &[u8; 4]) -> Self {
        FileHeader {
            magic: *magic,
            version: VERSION,
        }
    }

    pub fn write_to(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        // buf[6..8] is padding, left as zeros
    }

    pub fn read_from(buf: &[u8]) -> Option<Self> {
        if buf.len() < 6 {
            return None;
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        let version = u16::from_le_bytes([buf[4], buf[5]]);
        Some(FileHeader { magic, version })
    }

    pub fn validate(&self, expected_magic: &[u8; 4]) -> bool {
        self.magic == *expected_magic && self.version == VERSION
    }
}

/// Metadata stored in .meta file
#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub entry_point: Option<crate::NodeId>,
    pub node_count: usize,
    pub max_connections: usize,
    pub max_connections_level0: usize,
    pub max_level: u8,
}

impl IndexMetadata {
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        // Write entry point (6 bytes: 1 byte flag + 5 bytes NodeId)
        if let Some(ep) = self.entry_point {
            buf.push(1);
            buf.extend_from_slice(ep.as_bytes());
        } else {
            buf.push(0);
            buf.extend_from_slice(&[0u8; 5]);
        }

        // Write node_count (8 bytes)
        buf.extend_from_slice(&(self.node_count as u64).to_le_bytes());

        // Write connections (3 x 2 bytes: max_connections, max_connections_level0, max_level)
        buf.extend_from_slice(&(self.max_connections as u16).to_le_bytes());
        buf.extend_from_slice(&(self.max_connections_level0 as u16).to_le_bytes());
        buf.push(self.max_level);
    }

    pub fn read_from(buf: &[u8]) -> Option<Self> {
        if buf.len() < 1 + 5 + 8 + 2 + 2 + 1 {
            return None;
        }

        let has_entry = buf[0] != 0;
        let entry_point = if has_entry {
            let mut bytes = [0u8; 5];
            bytes.copy_from_slice(&buf[1..6]);
            Some(crate::NodeId(bytes))
        } else {
            None
        };

        let node_count = u64::from_le_bytes([
            buf[6], buf[7], buf[8], buf[9], buf[10], buf[11], buf[12], buf[13],
        ]) as usize;

        let max_connections = u16::from_le_bytes([buf[14], buf[15]]) as usize;
        let max_connections_level0 = u16::from_le_bytes([buf[16], buf[17]]) as usize;
        let max_level = buf[18];

        Some(IndexMetadata {
            entry_point,
            node_count,
            max_connections,
            max_connections_level0,
            max_level,
        })
    }
}
