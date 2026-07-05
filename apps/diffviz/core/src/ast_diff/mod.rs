//! AST infrastructure: source access, node abstractions, relevance scoring, and change types

pub mod changes;
pub mod error;
pub mod line_index;
pub mod nodes;
pub mod source;

pub use changes::{ASTChangeType, BACKGROUND, ESSENTIAL, IMPORTANT, NOISE, RelevanceScore};
pub use error::SourceError;
pub use line_index::LineIndex;
pub use nodes::{NodeLike, OwnedNodeData};
pub use source::{LineRange, SourceCode, SourceProvider};
