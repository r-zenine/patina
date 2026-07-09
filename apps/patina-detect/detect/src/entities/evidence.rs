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
    /// Detector 8 (data clumps, spec.md:226-248): a group of `>= 3`
    /// normalized `(name, type)` parameters recurring together across
    /// `>= 3` distinct signatures, promoted only once the precision gate
    /// (forwarded intact to another call) holds. `members` is the clump's
    /// sorted normalized parameter set, `occurrence_count` the number of
    /// distinct signatures it recurred in (trait-impl-deduped),
    /// `forwarding_chain` the qualified names of the functions that pass it
    /// on unchanged, and `subset_of_struct` the bonus evidence naming an
    /// existing struct whose fields already cover the clump, if any.
    DataClump {
        members: Vec<(String, String)>,
        occurrence_count: usize,
        forwarding_chain: Vec<String>,
        subset_of_struct: Option<String>,
    },
}
