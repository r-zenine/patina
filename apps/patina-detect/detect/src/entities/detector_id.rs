use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable identifier for a detector (e.g. "house-rules", "type2-clones").
///
/// Free-form rather than an enum: each detector phase introduces its own id
/// without touching this crate's core entity module (see spec.md's "nine
/// symptoms" — detectors are added incrementally across many phases).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DetectorId(String);

impl DetectorId {
    pub fn new(id: impl Into<String>) -> Self {
        DetectorId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DetectorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
