use serde::{Deserialize, Serialize};

/// Typed evidence a detector attaches to a `Symptom`, carrying the numbers
/// that produced the finding so triage is a glance, not an investigation
/// (spec.md's "Evidence in the rationale" design rule).
///
/// One variant per detector's evidence shape. `RuleMatch` covers detector 1
/// (house-rule violations, spec.md:130: "rule id, matched snippet") since
/// it's the next phase to land; later detector phases add their own
/// variants (clone group size, similarity scores, reference counts, ...)
/// rather than this crate speculating on their shape now.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Evidence {
    RuleMatch {
        rule_id: String,
        matched_snippet: String,
    },
}
