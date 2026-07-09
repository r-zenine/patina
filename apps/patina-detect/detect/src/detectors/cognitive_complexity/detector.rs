use crate::entities::Symptom;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The detector id every cognitive-complexity `Symptom`/`SymptomId` is
/// tagged with.
pub const DETECTOR_ID: &str = "cognitive-complexity";

/// Sonar's default threshold (15) flags fine code; spec.md:186 pins this
/// detector's threshold at 25 instead.
pub const COMPLEXITY_THRESHOLD: usize = 25;

#[derive(Debug, Error)]
pub enum CognitiveComplexityError {
    #[error("failed to walk directory {path}")]
    Walk {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read file {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to configure tree-sitter Rust grammar")]
    Language(#[from] tree_sitter::LanguageError),

    #[error("failed to parse {path} as Rust")]
    Parse { path: PathBuf },
}

/// TDD test-design stub for Phase 5 (cognitive complexity extremes,
/// spec.md:179-192) — see `.plans/plan-patina-detect/implementation-roadmap.md`
/// Phase 5. Real scoring logic lands in the implementation contribution.
pub fn run_cognitive_complexity(_root: &Path) -> Result<Vec<Symptom>, CognitiveComplexityError> {
    unimplemented!("Phase 5 implementation contribution replaces this stub")
}
