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
    /// Detector 3 (dead exports and write-only code, spec.md:150-163): a
    /// `pub` function or struct field with no reference found outside its
    /// own declaration via lspkit's `references`. `qualified_name` is the
    /// symbol's dotted/`::`-joined path (identifies which candidate this
    /// finding is about, since a detector run reports many). `reference_count`
    /// is the number of non-declaration references found (0 for a genuinely
    /// dead symbol). `test_only` distinguishes "production code only tests
    /// exercise" (spec.md's explicit non-drop case: tagged, not excluded)
    /// from a symbol with zero references anywhere.
    DeadExport {
        qualified_name: String,
        reference_count: usize,
        test_only: bool,
    },
    /// Detector 4 (middleman delegation chains, spec.md:165-177): a function
    /// whose body is a single delegating call, confirmed via lspkit's
    /// `incoming_calls` to have exactly one same-crate caller. `chain` is
    /// the composed sequence of such functions' qualified names, in call
    /// order (length 1 when no composition occurs, longer when middlemen
    /// chain into each other, e.g. `[facade, inner_helper]`).
    /// `caller_count` is always 1 for a reported finding (the gate that
    /// produced it), kept explicit as evidence rather than implied.
    /// `body_shape` names the detected body pattern (currently always
    /// "single delegating call").
    MiddlemanChain {
        chain: Vec<String>,
        caller_count: usize,
        body_shape: String,
    },
    /// Detector 6 (near-duplicate data structures, spec.md:194-210): a pair
    /// of structs whose normalized `(name, type)` field sets overlap at or
    /// above the Jaccard threshold (>= 0.7) with >= 4 shared fields,
    /// promoted only once lspkit's `references` confirms real conversion
    /// code exists between them (the conversion-evidence gate).
    /// `shared_field_count`/`total_field_count` are the Jaccard ratio's
    /// numerator/denominator (union size), `overlap_percent` the same ratio
    /// as a rounded percentage, `conversion_sites` the qualified names of
    /// the functions/impls found referencing both types, and
    /// `footprint_file_count` the number of distinct files touching either
    /// struct's declaration or a conversion site.
    NearDuplicateStructs {
        struct_a: String,
        struct_b: String,
        shared_field_count: usize,
        total_field_count: usize,
        overlap_percent: u8,
        conversion_sites: Vec<String>,
        footprint_file_count: usize,
    },
    /// Detector 7 (parallel dispatch, spec.md:212-224): an enum whose value
    /// is matched at `>= 3` sites across `>= 2` files, resolved via
    /// lspkit's `definition`/`hover` on each match's scrutinee type.
    /// `enum_name` is the enum's container-qualified name, `site_count`/
    /// `file_count` are the thresholds' numerator/denominator that produced
    /// the finding, and `arm_counts` lists each match site's arm count (one
    /// entry per site, same order as `Symptom::sites`).
    ParallelDispatch {
        enum_name: String,
        site_count: usize,
        file_count: usize,
        arm_counts: Vec<usize>,
    },
    /// Detector 9 (single-impl traits, spec.md:250-259): a trait with
    /// exactly one production implementor found via lspkit's
    /// `implementations`, surviving the marker/sealed-trait exclusion and
    /// the DI/Environment-pattern exclusion (a trait with a test-double
    /// impl alongside its one production impl is never reported —
    /// `test_doubles_present` is therefore always `false` on a symptom that
    /// actually exists; the field is kept for schema symmetry with the
    /// exclusion it documents). `impl_locations` are `path:line` strings
    /// for the production implementor(s) found.
    SingleImplTrait {
        trait_name: String,
        impl_count: usize,
        impl_locations: Vec<String>,
        test_doubles_present: bool,
    },
}
