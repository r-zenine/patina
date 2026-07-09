//! TDD test-design (red) for Phase 6 — Detector 8: data clumps
//! (`.plans/plan-patina-detect/implementation-roadmap.md`, Phase 6).
//!
//! These tests are written against an API that panics
//! (`patina_detect::detectors::data_clumps::run_data_clumps` is an
//! `unimplemented!()` stub) — the implementation contribution replaces the
//! stub body with real signature-clump extraction (spec.md:226-248) to make
//! these pass.
//!
//! Fixtures below assume tree-sitter-rust's grammar represents a receiver
//! (`&self`/`&mut self`) as a distinct `self_parameter` node, never a plain
//! `parameter` node — so a method's `self` is naturally excluded from its
//! normalized member set without special-casing. Not verified via a probe
//! test here (following `cognitive_complexity`'s precedent of noting
//! grammar assumptions in fixture comments, not re-deriving them per file).

use patina_detect::detectors::data_clumps::run_data_clumps;
use patina_detect::entities::Evidence;

/// A 3-parameter clump (`id: u64, name: String, email: String`) shared by
/// three free functions, forwarded intact through two call levels
/// (`level_a` -> `level_b` -> `level_c`) — spec.md:238's "clump threaded
/// through >= 2 signature levels" precision-gate example.
const FORWARDED_CLUMP: &str = r#"
fn level_a(id: u64, name: String, email: String) {
    level_b(id, name, email);
}

fn level_b(id: u64, name: String, email: String) {
    level_c(id, name, email);
}

fn level_c(id: u64, name: String, email: String) {
    println!("{id} {name} {email}");
}
"#;

/// The same 3-signature, 3-parameter clump as `FORWARDED_CLUMP`, but no
/// function ever passes all three onward to another call — each just uses
/// its own parameters locally. spec.md:239: "Non-traveling clumps: dropped
/// in v1."
const NON_FORWARDING_CLUMP: &str = r#"
fn level_a(id: u64, name: String, email: String) -> String {
    format!("{id}:{name}")
}

fn level_b(id: u64, name: String, email: String) -> String {
    format!("{name}:{email}")
}

fn level_c(id: u64, name: String, email: String) -> String {
    format!("{id}:{email}")
}
"#;

/// The clump's signature appears once as a trait declaration and three more
/// times as trait-impl methods that mirror it by necessity. spec.md:232-234:
/// "count a signature once per trait, not once per impl" — deduped, the
/// effective occurrence count is 1, below `MIN_OCCURRENCES` (3), so nothing
/// should be reported even though each impl also forwards intact to
/// `persist`.
const TRAIT_IMPL_CLUMP: &str = r#"
trait Repo {
    fn save(id: u64, name: String, email: String);
}

struct A;
impl Repo for A {
    fn save(id: u64, name: String, email: String) {
        persist(id, name, email);
    }
}

struct B;
impl Repo for B {
    fn save(id: u64, name: String, email: String) {
        persist(id, name, email);
    }
}

struct C;
impl Repo for C {
    fn save(id: u64, name: String, email: String) {
        persist(id, name, email);
    }
}

fn persist(x: u64, y: String, z: String) {}
"#;

/// Same clump as `FORWARDED_CLUMP`, but declared in a different parameter
/// order in each of the three signatures, with the last two occurrences
/// taking `&`/`&mut` references instead of owned values — spec.md's clump
/// identity is normalized member set (name + type, refs/mut stripped), not
/// declaration order, per decision D007's "survives parameter reordering".
const REORDERED_AND_REFERENCED_CLUMP: &str = r#"
fn level_a(id: u64, name: String, email: String) {
    Svc::level_b(name, id, email);
}

struct Svc;
impl Svc {
    fn level_b(name: String, id: &u64, email: String) {
        Svc::level_c(email, name, id);
    }
    fn level_c(email: String, name: String, id: &mut u64) {
        println!("{name}");
    }
}
"#;

/// Same clump/forwarding shape as `FORWARDED_CLUMP`, plus a struct whose
/// field set is a superset of the clump's members — spec.md:243-245's bonus
/// evidence ("if the clump's name/type set is a subset of an existing
/// struct's fields, the abstraction already exists").
const CLUMP_WITH_MATCHING_STRUCT: &str = r#"
struct Contact {
    id: u64,
    name: String,
    email: String,
    phone: String,
}

fn level_a(id: u64, name: String, email: String) {
    level_b(id, name, email);
}

fn level_b(id: u64, name: String, email: String) {
    level_c(id, name, email);
}

fn level_c(id: u64, name: String, email: String) {
    println!("{id} {name} {email}");
}
"#;

fn write_fixture(dir: &std::path::Path, relative: &str, content: &str) {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create fixture subdir");
    }
    std::fs::write(path, content).expect("failed to write fixture file");
}

type ClumpEvidence = (Vec<(String, String)>, usize, usize, Option<String>);

fn clump_symptoms(symptoms: &[patina_detect::entities::Symptom]) -> Vec<ClumpEvidence> {
    symptoms
        .iter()
        .filter_map(|s| match &s.evidence {
            Evidence::DataClump {
                members,
                occurrence_count,
                forwarding_chain,
                subset_of_struct,
            } => Some((
                members.clone(),
                *occurrence_count,
                forwarding_chain.len(),
                subset_of_struct.clone(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn a_clump_forwarded_intact_through_two_call_levels_is_reported() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "chain.rs", FORWARDED_CLUMP);

    let symptoms = run_data_clumps(dir.path()).expect("detector run failed");
    let clumps = clump_symptoms(&symptoms);

    assert_eq!(
        clumps.len(),
        1,
        "expected exactly one data-clump symptom, found: {:#?}",
        symptoms
    );
    let (members, occurrence_count, forwarding_sites, subset_of_struct) = &clumps[0];
    assert_eq!(occurrence_count, &3, "expected 3 distinct signatures");
    assert!(
        *forwarding_sites >= 1,
        "expected at least one forwarding site recorded, got {forwarding_sites}"
    );
    assert_eq!(
        members,
        &vec![
            ("email".to_string(), "String".to_string()),
            ("id".to_string(), "u64".to_string()),
            ("name".to_string(), "String".to_string()),
        ]
    );
    assert_eq!(subset_of_struct, &None, "no struct fixture present here");
}

#[test]
fn a_clump_never_forwarded_intact_is_not_reported() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "local_use.rs", NON_FORWARDING_CLUMP);

    let symptoms = run_data_clumps(dir.path()).expect("detector run failed");
    let clumps = clump_symptoms(&symptoms);

    assert!(
        clumps.is_empty(),
        "a non-traveling clump must be dropped in v1, found: {:#?}",
        clumps
    );
}

#[test]
fn trait_impl_signatures_are_deduped_to_one_occurrence_and_stay_below_threshold() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "repo.rs", TRAIT_IMPL_CLUMP);

    let symptoms = run_data_clumps(dir.path()).expect("detector run failed");
    let clumps = clump_symptoms(&symptoms);

    assert!(
        clumps.is_empty(),
        "3 impls of the same trait method must count as 1 occurrence, not 3, found: {:#?}",
        clumps
    );
}

#[test]
fn parameter_order_and_references_do_not_split_the_clump() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "reordered.rs", REORDERED_AND_REFERENCED_CLUMP);

    let symptoms = run_data_clumps(dir.path()).expect("detector run failed");
    let clumps = clump_symptoms(&symptoms);

    assert_eq!(
        clumps.len(),
        1,
        "reordered/referenced signatures must still merge into one clump, found: {:#?}",
        symptoms
    );
    let (members, occurrence_count, _, _) = &clumps[0];
    assert_eq!(occurrence_count, &3);
    assert_eq!(
        members,
        &vec![
            ("email".to_string(), "String".to_string()),
            ("id".to_string(), "u64".to_string()),
            ("name".to_string(), "String".to_string()),
        ],
        "refs/mut must be stripped before comparing member types"
    );
}

#[test]
fn bonus_evidence_names_the_struct_whose_fields_are_a_superset_of_the_clump() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    write_fixture(dir.path(), "with_struct.rs", CLUMP_WITH_MATCHING_STRUCT);

    let symptoms = run_data_clumps(dir.path()).expect("detector run failed");
    let clumps = clump_symptoms(&symptoms);

    assert_eq!(clumps.len(), 1, "found: {:#?}", symptoms);
    let (_, _, _, subset_of_struct) = &clumps[0];
    assert_eq!(
        subset_of_struct,
        &Some("Contact".to_string()),
        "Contact's fields are a superset of the clump's members"
    );
}
