//! TDD test-design (red) for Phase 5 — Detector 5: cognitive complexity
//! extremes (`.plans/plan-patina-detect/implementation-roadmap.md`, Phase 5).
//!
//! These tests are written against an API that panics
//! (`patina_detect::detectors::cognitive_complexity::run_cognitive_complexity`
//! is a `unimplemented!()` stub) — the implementation contribution replaces
//! the stub body with real Sonar-cognitive-complexity scoring
//! (spec.md:179-192) to make these pass.
//!
//! Score derivations below were computed by hand against the tree-sitter-rust
//! grammar shapes confirmed via a throwaway probe test (not committed):
//! `if_expression` has `condition`/`consequence`/`alternative` fields, where
//! `alternative` wraps an `else_clause` whose single child is either another
//! `if_expression` (else-if, continuation — no nesting increment) or a
//! `block` (else, flat +1); `while_expression`/`for_expression`/
//! `loop_expression`/`match_expression` each contribute `1 + nesting` and
//! nest their body/arms one level deeper; `binary_expression`'s `operator`
//! field node's own `kind()` is the literal operator text (e.g. `"&&"`), and
//! a maximal run of the *same* logical operator counts once, a change in
//! operator starts a new run. Implementation must confirm these derivations
//! against real scoring — the exact score assertions here are the
//! acceptance bar for the increment/nesting table, not a guess to preserve
//! blindly.

use patina_detect::detectors::cognitive_complexity::run_cognitive_complexity;
use patina_detect::entities::Evidence;

/// Deeply nested control flow (if → for → while → if&& → if/else-if/else,
/// with a for loop nested inside the innermost branch) — hand-derived score
/// 28, well above the ≥25 threshold.
const HIGH_COMPLEXITY: &str = r#"
fn deeply_nested(a: i32, b: i32, c: i32) -> i32 {
    let mut result = 0;
    if a > 0 {
        for i in 0..a {
            while i < b {
                if c > 0 && b > 0 {
                    if a == b {
                        for _ in 0..a {
                            result += 1;
                        }
                    } else if a == c {
                        result += 2;
                    } else {
                        result += 3;
                    }
                }
            }
        }
    }
    result
}
"#;

/// A chain of 6 nested `if`s, each condition combining two terms with `&&`
/// on the first 4 levels — hand-derived score 25 (21 from nesting increments
/// 1+2+3+4+5+6, plus 4 flat `&&` increments), exactly at the threshold.
/// Distinct from `HIGH_COMPLEXITY`'s score so ranking is unambiguous.
const MEDIUM_COMPLEXITY: &str = r#"
fn medium_nesting(a: i32, b: i32) -> i32 {
    let mut result = 0;
    if a > 0 && b > 0 {
        if a > 1 && b > 1 {
            if a > 2 && b > 2 {
                if a > 3 && b > 3 {
                    if a > 4 {
                        if a > 5 {
                            result = 1;
                        }
                    }
                }
            }
        }
    }
    result
}
"#;

/// A flat `match` with 8 trivial arms and no nested branching — Rust's
/// match-heavy dispatch idiom the spec explicitly calls out as a case Sonar's
/// default threshold (15) would false-positive on. Hand-derived score 1
/// (the match's own base increment; no arm body nests further).
const FLAT_MATCH_DISPATCH: &str = r#"
fn dispatch(x: i32) -> i32 {
    match x {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 5,
        5 => 6,
        6 => 7,
        _ => 0,
    }
}
"#;

fn write_fixture(dir: &std::path::Path, relative: &str, content: &str) {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create fixture subdir");
    }
    std::fs::write(path, content).expect("failed to write fixture file");
}

fn complexity_symptoms(
    symptoms: &[patina_detect::entities::Symptom],
) -> Vec<(usize, usize, usize)> {
    symptoms
        .iter()
        .filter_map(|s| match s.evidence {
            Evidence::ComplexityScore {
                score,
                function_length,
                max_nesting_depth,
            } => Some((score, function_length, max_nesting_depth)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_deeply_nested_function_scores_at_or_above_the_threshold_and_is_reported() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "high.rs", HIGH_COMPLEXITY);

    let symptoms = run_cognitive_complexity(dir.path()).expect("detector run failed");
    let scores = complexity_symptoms(&symptoms);

    assert_eq!(
        scores.len(),
        1,
        "expected exactly one complexity symptom, found: {:#?}",
        symptoms
    );
    assert!(
        scores[0].0 >= 25,
        "deeply nested function must score >= 25, got {}",
        scores[0].0
    );
}

#[test]
fn a_flat_match_heavy_dispatch_function_does_not_false_positive() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "dispatch.rs", FLAT_MATCH_DISPATCH);

    let symptoms = run_cognitive_complexity(dir.path()).expect("detector run failed");
    let scores = complexity_symptoms(&symptoms);

    assert!(
        scores.is_empty(),
        "a flat match dispatch function must not be reported, found: {:#?}",
        scores
    );
}

#[test]
fn reported_functions_are_ranked_by_score_descending() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "high.rs", HIGH_COMPLEXITY);
    write_fixture(dir.path(), "medium.rs", MEDIUM_COMPLEXITY);

    let symptoms = run_cognitive_complexity(dir.path()).expect("detector run failed");
    let scores = complexity_symptoms(&symptoms);

    assert_eq!(scores.len(), 2, "expected both functions to be reported");
    assert!(
        scores[0].0 > scores[1].0,
        "expected descending score order, got: {:?}",
        scores
    );
}

#[test]
fn fingerprint_survives_leading_lines_added_before_the_function() {
    let dir_a = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir_a.path(), "shifted.rs", HIGH_COMPLEXITY);

    let padded = format!("// a leading comment\n\n\n// another one\n\n{HIGH_COMPLEXITY}");
    let dir_b = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir_b.path(), "shifted.rs", &padded);

    let first = run_cognitive_complexity(dir_a.path()).expect("first run failed");
    let second = run_cognitive_complexity(dir_b.path()).expect("second run failed");

    let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
    let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
    assert_eq!(
        first_ids, second_ids,
        "padding lines before the function must not change its SymptomId"
    );
}

#[test]
fn rewriting_the_function_body_changes_the_symptom_id() {
    let dir_a = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir_a.path(), "rewritten.rs", HIGH_COMPLEXITY);

    let rewritten = HIGH_COMPLEXITY.replace("a == b", "a == b + 1");
    let dir_b = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir_b.path(), "rewritten.rs", &rewritten);

    let first = run_cognitive_complexity(dir_a.path()).expect("first run failed");
    let second = run_cognitive_complexity(dir_b.path()).expect("second run failed");

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_ne!(
        first[0].id.to_string(),
        second[0].id.to_string(),
        "a rewritten function body must produce a different SymptomId"
    );
}
