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
    /// Detector 2 (Type-2 clones, spec.md:134-148): a group of function-sized
    /// subtrees whose normalized structure hashes identically. `group_size`
    /// is the member count, `node_count` the shared subtree's semantic-node
    /// count (proves the min-size gate held), `all_test_code` distinguishes
    /// clones confined to `#[cfg(test)]`/`#[test]` code per the spec's FP
    /// control.
    CloneGroup {
        group_size: usize,
        node_count: usize,
        all_test_code: bool,
    },
    /// Detector 5 (cognitive complexity extremes, spec.md:179-192): a
    /// function scoring at or above the threshold (≥25) under the Sonar
    /// cognitive-complexity spec (+1 per branch, +1 extra per nesting
    /// level). `function_length` is the function's line span,
    /// `max_nesting_depth` the deepest nesting level reached.
    ComplexityScore {
        score: usize,
        function_length: usize,
        max_nesting_depth: usize,
    },
}
