//! TDD test-design (red) for Phase 4 — Detector 2: Type-2 clones
//! (`.plans/plan-patina-detect/implementation-roadmap.md`, Phase 4).
//!
//! These tests are written against an API that does not exist yet
//! (`patina_detect::detectors::type2_clones::run_type2_clones` and
//! `Evidence::CloneGroup`) — the crate is expected to fail to compile until
//! the implementation contribution adds them. That compile failure is this
//! phase's "red" state; the fixture shapes below are what the implementation
//! must satisfy to go green.
//!
//! Fixture sizing note: fixture functions below are written to be clearly
//! above or clearly below the spec's "~30 semantic nodes" min-size gate
//! (spec.md:143), but the exact node count depends on the counting
//! function's own definition of "semantic node" (`LanguageDescriptor::
//! semantic_kind_map`), which doesn't exist yet either. The implementation
//! contribution must confirm actual counts against real fixtures rather than
//! trusting these numbers blindly.

use patina_detect::detectors::type2_clones::run_type2_clones;
use patina_detect::entities::{Evidence, SiteRole};

/// Two structurally-identical, differently-named/differently-variable-named
/// functions, well above the min-size gate, in two different files.
const MEMBER_A: &str = r#"
pub fn compute_score(values: &[i32], threshold: i32) -> i32 {
    let mut total = 0;
    for value in values {
        if *value > threshold {
            total += value * 2;
        } else if *value < 0 {
            total -= value;
        } else {
            total += 1;
        }
    }
    if total > 100 {
        total = 100;
    }
    total
}
"#;

const MEMBER_B: &str = r#"
pub fn compute_rating(numbers: &[i32], limit: i32) -> i32 {
    let mut sum = 0;
    for number in numbers {
        if *number > limit {
            sum += number * 2;
        } else if *number < 0 {
            sum -= number;
        } else {
            sum += 1;
        }
    }
    if sum > 100 {
        sum = 100;
    }
    sum
}
"#;

/// Trivial function, well under the min-size gate — duplicated but must not
/// be reported.
const TRIVIAL: &str = "pub fn add_one(x: i32) -> i32 {\n    x + 1\n}\n";

/// A clone pair with a *different* shape from `MEMBER_A`/`MEMBER_B` (a
/// `while` loop instead of `for`, no `else if` branch) — used to build a
/// same-file clone group that is structurally distinct from the cross-file
/// group, so the two don't collapse into one 4-member equivalence class.
const MEMBER_C: &str = r#"
fn accumulate_totals(values: &[i32], step: i32, floor: i32) -> i32 {
    let mut index = 0;
    let mut total = 0;
    while index < values.len() {
        if values[index] > step {
            total += values[index];
        } else if values[index] < floor {
            total -= 1;
        } else {
            total += 2;
        }
        index += 1;
    }
    total
}
"#;

const MEMBER_D: &str = r#"
fn accumulate_counts(numbers: &[i32], gap: i32, minimum: i32) -> i32 {
    let mut cursor = 0;
    let mut sum = 0;
    while cursor < numbers.len() {
        if numbers[cursor] > gap {
            sum += numbers[cursor];
        } else if numbers[cursor] < minimum {
            sum -= 1;
        } else {
            sum += 2;
        }
        cursor += 1;
    }
    sum
}
"#;

fn write_fixture(dir: &std::path::Path, relative: &str, content: &str) {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create fixture subdir");
    }
    std::fs::write(path, content).expect("failed to write fixture file");
}

#[test]
fn cross_file_clone_pair_produces_one_clone_group_symptom_with_two_sites() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "a.rs", MEMBER_A);
    write_fixture(dir.path(), "b.rs", MEMBER_B);

    let symptoms = run_type2_clones(dir.path()).expect("detector run failed");

    let clone_groups: Vec<_> = symptoms
        .iter()
        .filter(|s| matches!(&s.evidence, Evidence::CloneGroup { .. }))
        .collect();
    assert_eq!(
        clone_groups.len(),
        1,
        "expected exactly one clone-group symptom, found: {:#?}",
        symptoms
    );

    let group = clone_groups[0];
    assert_eq!(group.sites.len(), 2, "one Site per clone member");
    assert!(
        group.sites.iter().all(|s| s.role == SiteRole::CloneMember),
        "every site in a clone-group symptom is a CloneMember"
    );

    let files: Vec<_> = group.sites.iter().map(|s| s.file.clone()).collect();
    assert_ne!(
        files[0], files[1],
        "the two clone members live in different files"
    );
}

#[test]
fn trivial_function_pair_below_min_size_is_not_reported() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "a.rs", TRIVIAL);
    write_fixture(dir.path(), "b.rs", TRIVIAL);

    let symptoms = run_type2_clones(dir.path()).expect("detector run failed");
    let clone_groups: Vec<_> = symptoms
        .iter()
        .filter(|s| matches!(&s.evidence, Evidence::CloneGroup { .. }))
        .collect();

    assert!(
        clone_groups.is_empty(),
        "trivial functions below the ~30 semantic node gate must not produce a clone-group symptom, found: {:#?}",
        clone_groups
    );
}

#[test]
fn cross_file_clone_group_is_ranked_above_a_same_file_clone_group() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    // Cross-file group: MEMBER_A / MEMBER_B split across two files.
    write_fixture(dir.path(), "cross_a.rs", MEMBER_A);
    write_fixture(dir.path(), "cross_b.rs", MEMBER_B);
    // Same-file group: a structurally distinct pair (MEMBER_C/MEMBER_D) so
    // it doesn't collapse into the same equivalence class as MEMBER_A/B.
    let same_file_source = format!("{MEMBER_C}\n{MEMBER_D}");
    write_fixture(dir.path(), "same_file.rs", &same_file_source);

    let symptoms = run_type2_clones(dir.path()).expect("detector run failed");
    let clone_groups: Vec<_> = symptoms
        .iter()
        .filter(|s| matches!(&s.evidence, Evidence::CloneGroup { .. }))
        .collect();

    assert_eq!(
        clone_groups.len(),
        2,
        "expected both clone groups to be found"
    );

    let is_cross_file = |s: &&patina_detect::entities::Symptom| {
        let files: std::collections::HashSet<_> = s.sites.iter().map(|site| &site.file).collect();
        files.len() > 1
    };
    assert!(
        is_cross_file(&clone_groups[0]),
        "the first-ranked clone group must be the cross-file one, order was: {:#?}",
        clone_groups
    );
}

#[test]
fn all_test_code_clone_group_is_tagged_separately() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let test_module = format!(
        "#[cfg(test)]\nmod tests {{\n{}\n{}\n}}\n",
        MEMBER_A.replace("pub fn", "fn"),
        MEMBER_B.replace("pub fn", "fn")
    );
    write_fixture(dir.path(), "tests_only.rs", &test_module);

    let symptoms = run_type2_clones(dir.path()).expect("detector run failed");
    let clone_groups: Vec<_> = symptoms
        .iter()
        .filter_map(|s| match &s.evidence {
            Evidence::CloneGroup { all_test_code, .. } => Some(*all_test_code),
            _ => None,
        })
        .collect();

    assert_eq!(clone_groups.len(), 1, "expected one clone-group symptom");
    assert!(
        clone_groups[0],
        "a clone group whose members are entirely inside #[cfg(test)] must be tagged all_test_code"
    );
}

#[test]
fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "a.rs", MEMBER_A);
    write_fixture(dir.path(), "b.rs", MEMBER_B);

    let first = run_type2_clones(dir.path()).expect("first run failed");
    let second = run_type2_clones(dir.path()).expect("second run failed");

    let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
    let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
    assert_eq!(first_ids, second_ids);
}
