use crate::entities::Symptom;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Type2ClonesError {
    #[error("type-2 clones detector is not yet implemented (Phase 4 test-design stub)")]
    NotYetImplemented,
}

/// Stub for the Type-2 clones detector (spec.md:134-148): hashes
/// function-sized `SemanticNode` subtrees (identifiers/literals normalized
/// to placeholders, structure + node kinds retained) and reports clone
/// groups. Signature exists so `type2_clones_detection_tests.rs`'s red
/// fixture tests (written in this phase's test-design contribution) compile
/// and fail at runtime rather than blocking `cargo clippy --workspace
/// --all-targets` at compile time — the real implementation lands in the
/// next (implementation) contribution for Phase 4.
pub fn run_type2_clones(_root: &Path) -> Result<Vec<Symptom>, Type2ClonesError> {
    unimplemented!(
        "Type-2 clones detector — stub added by the Phase 4 test-design contribution; \
         real subtree-hashing implementation lands in the Phase 4 implementation contribution"
    )
}
