use crate::entities::Symptom;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataClumpsError {
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

/// TDD test-design stub for Phase 6 (data clumps, spec.md:226-248) — see
/// `.plans/plan-patina-detect/implementation-roadmap.md` Phase 6. Real
/// signature-clump extraction, trait-impl dedup, and the forwarding-intact
/// precision gate land in the implementation contribution.
pub fn run_data_clumps(_root: &Path) -> Result<Vec<Symptom>, DataClumpsError> {
    unimplemented!("Phase 6 implementation contribution replaces this stub")
}
