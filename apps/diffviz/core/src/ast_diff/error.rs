/// Error types for source code operations
#[derive(Debug)]
pub enum SourceError {
    /// Node byte range is out of bounds for the source content
    NodeRangeOutOfBounds {
        node_start: usize,
        node_end: usize,
        source_length: usize,
    },
    /// Source content contains invalid UTF-8
    InvalidUtf8(std::str::Utf8Error),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceError::NodeRangeOutOfBounds {
                node_start,
                node_end,
                source_length,
            } => write!(
                f,
                "Node byte range {node_start}..{node_end} is out of bounds for source of length {source_length}"
            ),
            SourceError::InvalidUtf8(e) => write!(f, "Invalid UTF-8 in source: {e}"),
        }
    }
}

impl std::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SourceError::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::str::Utf8Error> for SourceError {
    fn from(e: std::str::Utf8Error) -> Self {
        SourceError::InvalidUtf8(e)
    }
}
