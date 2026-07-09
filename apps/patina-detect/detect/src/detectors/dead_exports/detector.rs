use crate::entities::Symptom;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeadExportsError {
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

    #[error("language server error")]
    Lsp(#[from] lspkit::Error),
}

/// Runs the dead-exports detector (spec.md:150-163) against the Rust crate
/// rooted at `root`: every `pub` function and struct field enumerated from
/// tree-sitter is checked via a real `rust-analyzer` process (`lspkit::
/// LspClient::references`) for reference sites outside its own declaration.
/// Trait-impl methods, derive-heavy struct fields, and bin entry points are
/// excluded outright (spec.md's mechanical exclusion list); a candidate
/// whose only references live in test code is tagged `test_only` rather
/// than dropped.
///
/// Not yet implemented (Phase 8's implementation contribution) — this stub
/// exists so the fixture tests in `tests/dead_exports_detection_tests.rs`
/// compile and fail at runtime (TDD red) instead of blocking the
/// workspace-wide `cargo clippy --all-targets -- -D warnings` gate the
/// pre-commit hook enforces on every commit.
pub fn run_dead_exports(root: &Path) -> Result<Vec<Symptom>, DeadExportsError> {
    unimplemented!(
        "Phase 8 implementation: enumerate pub symbols/fields via tree-sitter, \
         query lspkit::LspClient::references per candidate, apply the \
         trait-impl/derive-field/bin-entry-point exclusion list, tag \
         test-only usage (root: {})",
        root.display()
    )
}
