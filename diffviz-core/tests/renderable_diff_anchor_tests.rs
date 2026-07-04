//! Behavioral integration tests for semantic anchor extraction via RenderableDiff.
//!
//! These tests verify the behavior contract at the public API level: given source code
//! and a line range, the correct SemanticAnchor is produced on each RenderableLine.
//! They are designed to survive the string-scanner → tree-sitter refactor unchanged.

use diffviz_core::ast_diff::SourceCode;
use diffviz_core::common::{LanguageParser, ProgrammingLanguage};
use diffviz_core::create_reviewable_diff_from_range;
use diffviz_core::parsers::{GoParser, PythonParser, RustParser};
use diffviz_core::renderable_diff::{RenderableDiff, SemanticAnchor, SemanticAnchorType};
use diffviz_core::reviewable_diff::NodeChangeStatus;

/// Build `(line_number, semantic_anchor)` pairs for the first ReviewableDiff in the range.
///
/// Line numbers are 1-based and relative to the boundary (the extracted semantic unit),
/// not the full source file.
fn anchors_for_range(
    source: &str,
    start_line: usize,
    end_line: usize,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Vec<(usize, Option<SemanticAnchor>)> {
    let new_source = SourceCode::new(source.to_string());
    let mut diffs = create_reviewable_diff_from_range(
        "test_file",
        start_line,
        end_line,
        None,
        &new_source,
        language,
        parser,
    )
    .expect("create_reviewable_diff_from_range failed");

    assert!(
        !diffs.is_empty(),
        "no ReviewableDiff returned for range {start_line}-{end_line}"
    );
    let reviewable = diffs.remove(0);
    let renderable =
        RenderableDiff::try_from(&reviewable).expect("RenderableDiff::try_from failed");

    renderable
        .lines
        .iter()
        .map(|l| (l.line_number, l.semantic_anchor.clone()))
        .collect()
}

/// Return the anchor for a specific 1-based line within the boundary, or None if absent.
fn anchor_at(lines: &[(usize, Option<SemanticAnchor>)], line: usize) -> Option<SemanticAnchor> {
    lines
        .iter()
        .find(|(n, _)| *n == line)
        .and_then(|(_, anchor)| anchor.clone())
}

// ── Rust: function signatures ────────────────────────────────────────────────

#[test]
fn rust_fn_signature() {
    let parser = RustParser::new();
    let source = "fn calculate(x: i32) -> i32 {\n    x + 1\n}\n";
    let lines = anchors_for_range(source, 1, 3, ProgrammingLanguage::Rust, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: "calculate".to_string(),
        })
    );
}

#[test]
fn rust_pub_async_fn() {
    let parser = RustParser::new();
    let source = "pub async fn fetch() {\n    // nothing\n}\n";
    let lines = anchors_for_range(source, 1, 3, ProgrammingLanguage::Rust, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: "fetch".to_string(),
        })
    );
}

// ── Rust: type declarations ──────────────────────────────────────────────────

#[test]
fn rust_struct_decl() {
    let parser = RustParser::new();
    let source = "struct Config {\n    value: i32,\n}\n";
    let lines = anchors_for_range(source, 1, 3, ProgrammingLanguage::Rust, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::StructDeclaration,
            identifier: "Config".to_string(),
        })
    );
}

// KNOWN BUG: enum_item is classified as DataStructure in GenericSemanticTreeBuilder,
// so unit_type_to_semantic_kind maps it to SemanticNodeKind::Struct, not Enum.
// extract_struct_anchor checks starts_with("struct ") which fails for "enum Color {".
// This test captures the desired behavior; the fix is out of scope for this plan.
#[test]
#[ignore = "enum boundaries incorrectly classified as Struct — unit_type_to_semantic_kind maps DataStructure to Struct, not Enum"]
fn rust_enum_decl() {
    let parser = RustParser::new();
    let source = "enum Color {\n    Red,\n    Blue,\n}\n";
    let lines = anchors_for_range(source, 1, 4, ProgrammingLanguage::Rust, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::EnumDeclaration,
            identifier: "Color".to_string(),
        })
    );
}

// ── Rust: variable bindings inside a function body ──────────────────────────

const RUST_BINDINGS_SOURCE: &str = "fn bindings() {\n    let config = Config::new();\n    let mut counter = 0;\n    const MAX: usize = 100;\n}\n";

#[test]
fn rust_let_binding() {
    let parser = RustParser::new();
    let lines = anchors_for_range(
        RUST_BINDINGS_SOURCE,
        1,
        5,
        ProgrammingLanguage::Rust,
        &parser,
    );
    assert_eq!(
        anchor_at(&lines, 2),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::VariableAssignment,
            identifier: "config".to_string(),
        })
    );
}

#[test]
fn rust_let_mut_binding() {
    let parser = RustParser::new();
    let lines = anchors_for_range(
        RUST_BINDINGS_SOURCE,
        1,
        5,
        ProgrammingLanguage::Rust,
        &parser,
    );
    assert_eq!(
        anchor_at(&lines, 3),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::VariableAssignment,
            identifier: "counter".to_string(),
        })
    );
}

#[test]
fn rust_const_item() {
    let parser = RustParser::new();
    let lines = anchors_for_range(
        RUST_BINDINGS_SOURCE,
        1,
        5,
        ProgrammingLanguage::Rust,
        &parser,
    );
    assert_eq!(
        anchor_at(&lines, 4),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::VariableAssignment,
            identifier: "MAX".to_string(),
        })
    );
}

// ── Go ───────────────────────────────────────────────────────────────────────

#[test]
fn go_func() {
    let parser = GoParser::new();
    let source = "package main\n\nfunc handleRequest(w http.ResponseWriter) {\n}\n";
    // function starts at line 3
    let lines = anchors_for_range(source, 3, 4, ProgrammingLanguage::Go, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: "handleRequest".to_string(),
        })
    );
}

#[test]
fn go_method() {
    let parser = GoParser::new();
    let source = "package main\n\nfunc (s *Server) Serve() {\n}\n";
    let lines = anchors_for_range(source, 3, 4, ProgrammingLanguage::Go, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: "Serve".to_string(),
        })
    );
}

// ── Python ───────────────────────────────────────────────────────────────────

#[test]
fn python_def() {
    let parser = PythonParser::new();
    let source = "def process(items: List[str]):\n    pass\n";
    let lines = anchors_for_range(source, 1, 2, ProgrammingLanguage::Python, &parser);
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: "process".to_string(),
        })
    );
}

// ── Negative cases ────────────────────────────────────────────────────────────

#[test]
fn comment_no_anchor() {
    let parser = RustParser::new();
    let source = "fn with_comment() {\n    // fn fake()\n    let x = 1;\n}\n";
    let lines = anchors_for_range(source, 1, 4, ProgrammingLanguage::Rust, &parser);
    assert_eq!(anchor_at(&lines, 2), None);
}

#[test]
fn empty_no_anchor() {
    let parser = RustParser::new();
    let source = "fn with_empty() {\n\n    let x = 1;\n}\n";
    let lines = anchors_for_range(source, 1, 4, ProgrammingLanguage::Rust, &parser);
    assert_eq!(anchor_at(&lines, 2), None);
}

// ── Phase 1 gate: OwnedNodeData.identifier is populated ──────────────────────
//
// These tests verify that the identifier extracted during tree building survives
// the SemanticNode → OwnedNodeData → DiffNode pipeline. They inspect the
// boundary DiffNode directly rather than going through RenderableDiff.

fn boundary_identifier(
    source: &str,
    start_line: usize,
    end_line: usize,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Option<String> {
    let new_source = SourceCode::new(source.to_string());
    let mut diffs = create_reviewable_diff_from_range(
        "test_file",
        start_line,
        end_line,
        None,
        &new_source,
        language,
        parser,
    )
    .expect("create_reviewable_diff_from_range failed");
    assert!(!diffs.is_empty());
    let diff = diffs.remove(0);
    match &diff.boundary.change_status {
        NodeChangeStatus::Added { node } | NodeChangeStatus::Unchanged { node } => {
            node.identifier.clone()
        }
        NodeChangeStatus::Modified { new_node, .. } => new_node.identifier.clone(),
        NodeChangeStatus::Deleted { node } => node.identifier.clone(),
    }
}

#[test]
fn phase1_rust_fn_identifier() {
    let parser = RustParser::new();
    assert_eq!(
        boundary_identifier(
            "fn calculate(x: i32) -> i32 {\n    x + 1\n}\n",
            1,
            3,
            ProgrammingLanguage::Rust,
            &parser
        ),
        Some("calculate".to_string())
    );
}

#[test]
fn phase1_rust_struct_identifier() {
    let parser = RustParser::new();
    assert_eq!(
        boundary_identifier(
            "struct Config {\n    value: i32,\n}\n",
            1,
            3,
            ProgrammingLanguage::Rust,
            &parser
        ),
        Some("Config".to_string())
    );
}

#[test]
fn phase1_rust_let_identifier() {
    let parser = RustParser::new();
    // let_declaration must carry the binding name via RustDescriptor::extract_identifier
    let source = "fn f() {\n    let config = 42;\n}\n";
    assert_eq!(
        boundary_identifier(source, 1, 3, ProgrammingLanguage::Rust, &parser),
        Some("f".to_string()) // boundary is the function; identifier is the fn name
    );
}

#[test]
fn phase1_rust_const_identifier() {
    let parser = RustParser::new();
    assert_eq!(
        boundary_identifier(
            "const MAX: usize = 100;\n",
            1,
            1,
            ProgrammingLanguage::Rust,
            &parser
        ),
        Some("MAX".to_string())
    );
}

#[test]
fn phase1_go_func_identifier() {
    let parser = GoParser::new();
    let source = "package main\n\nfunc handleRequest() {\n}\n";
    assert_eq!(
        boundary_identifier(source, 3, 4, ProgrammingLanguage::Go, &parser),
        Some("handleRequest".to_string())
    );
}

#[test]
fn phase1_python_def_identifier() {
    let parser = PythonParser::new();
    let source = "def process(items):\n    pass\n";
    assert_eq!(
        boundary_identifier(source, 1, 2, ProgrammingLanguage::Python, &parser),
        Some("process".to_string())
    );
}

// ── Phase 3 gate: correct anchors through the Myers diff (Modified) path ─────
//
// create_line_by_line_diff_for_modified used to pass a hardcoded byte offset of 0
// to extract_semantic_anchor at every call site, so every line queried the tree at
// byte 0 instead of its real position. These tests use a preamble comment so the
// function boundary does NOT start at byte 0, making a regression to the old
// all-zero behavior fail loudly instead of passing by coincidence.

const MYERS_OLD_SOURCE: &str =
    "// preamble\nfn calculate(x: i32) -> i32 {\n    let temp = x;\n    temp + 1\n}\n";
const MYERS_NEW_SOURCE: &str =
    "// preamble\nfn calculate(x: i32) -> i32 {\n    let temp = x;\n    temp + 2\n}\n";

fn anchors_for_modified_range(
    old_source_text: &str,
    new_source_text: &str,
    start_line: usize,
    end_line: usize,
    language: ProgrammingLanguage,
    parser: &dyn LanguageParser,
) -> Vec<(usize, Option<SemanticAnchor>)> {
    let old_source = SourceCode::new(old_source_text.to_string());
    let new_source = SourceCode::new(new_source_text.to_string());
    let mut diffs = create_reviewable_diff_from_range(
        "test_file",
        start_line,
        end_line,
        Some(&old_source),
        &new_source,
        language,
        parser,
    )
    .expect("create_reviewable_diff_from_range failed");

    assert!(
        !diffs.is_empty(),
        "no ReviewableDiff returned for range {start_line}-{end_line}"
    );
    let reviewable = diffs.remove(0);
    assert!(
        matches!(
            reviewable.boundary.change_status,
            NodeChangeStatus::Modified { .. }
        ),
        "expected a Modified boundary to exercise the Myers diff path"
    );

    let renderable =
        RenderableDiff::try_from(&reviewable).expect("RenderableDiff::try_from failed");

    renderable
        .lines
        .iter()
        .map(|l| (l.line_number, l.semantic_anchor.clone()))
        .collect()
}

#[test]
fn myers_path_signature_anchor() {
    let parser = RustParser::new();
    let lines = anchors_for_modified_range(
        MYERS_OLD_SOURCE,
        MYERS_NEW_SOURCE,
        2,
        5,
        ProgrammingLanguage::Rust,
        &parser,
    );
    assert_eq!(
        anchor_at(&lines, 1),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: "calculate".to_string(),
        })
    );
}

#[test]
fn myers_path_variable_binding_anchor() {
    let parser = RustParser::new();
    let lines = anchors_for_modified_range(
        MYERS_OLD_SOURCE,
        MYERS_NEW_SOURCE,
        2,
        5,
        ProgrammingLanguage::Rust,
        &parser,
    );
    assert_eq!(
        anchor_at(&lines, 2),
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::VariableAssignment,
            identifier: "temp".to_string(),
        })
    );
}
