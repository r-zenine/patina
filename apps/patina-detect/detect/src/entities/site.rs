use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An inclusive range of line numbers within a `Site`'s file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

/// What role a `Site` plays within its `Symptom`. Per-detector: each detector
/// phase may add the roles it needs (e.g. `CloneMember` for Type-2 clones,
/// `ConversionSite` for near-duplicate structs) — this enum is expected to
/// grow across the plan's detector phases, not be finalized now.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteRole {
    /// A code location that itself matched a detector's rule/pattern.
    MatchSite,
    CloneMember,
    ConversionSite,
    Definition,
    Caller,
}

/// A single line-range location a triager should look at, with a role
/// explaining why it's part of the finding and a note explaining what to
/// scrutinize there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub file: PathBuf,
    pub line_ranges: Vec<LineRange>,
    pub role: SiteRole,
    pub note: String,
}
