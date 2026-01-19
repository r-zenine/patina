//! Test for Go semantic analysis duplication and over-decomposition bug
//!
//! This test reproduces a bug where DiffViz Go parser creates duplicate and
//! over-decomposed semantic boundaries. Specifically:
//! 1. Package declarations shown as deleted+added instead of unchanged
//! 2. Struct definitions decomposed into multiple separate boundaries
//! 3. Import block changes not properly detected/shown
//! 4. Missing semantic grouping for related constructs

use diffviz_core::{
    ast_diff::SourceCode,
    common::{LanguageParser, ProgrammingLanguage},
    parsers::GoParser,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Test that Go semantic analysis produces correct semantic boundaries without duplication
///
/// This test reproduces the issue where adding struct and methods to a basic Go program
/// results in 8 semantic boundaries (with duplicates) instead of the expected ~5.
#[test]
#[ignore = "Bug #3: Go semantic analysis creates duplicate boundaries and over-decomposition"]
fn bug_3_go_duplicate_boundaries() {
    // Create old version: basic Go program with main function
    let old_code = r#"package main

import "fmt"

func main() {
	fmt.Println("Web server starting...")
}
"#;

    // Create new version: adds multiple imports, struct, and methods
    let new_code = r#"package main

import (
	"fmt"
	"net/http"
	"log"
)

type Server struct {
	port   string
	routes map[string]http.HandlerFunc
}

func NewServer(port string) *Server {
	return &Server{
		port:   port,
		routes: make(map[string]http.HandlerFunc),
	}
}

func (s *Server) AddRoute(path string, handler http.HandlerFunc) {
	s.routes[path] = handler
}

func main() {
	fmt.Println("Web server starting...")
	server := NewServer(":8080")
	server.AddRoute("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello, World!")
	})
}
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    // Parse and analyze using Go parser
    let parser = GoParser::new();
    let mut ts_parser = tree_sitter::Parser::new();
    ts_parser.set_language(parser.get_language()).unwrap();

    let old_tree = ts_parser.parse(old_code, None).unwrap();
    let new_tree = ts_parser.parse(new_code, None).unwrap();

    let old_semantic = parser.build_semantic_tree(&old_tree, old_code).unwrap();
    let new_semantic = parser.build_semantic_tree(&new_tree, new_code).unwrap();

    let semantic_pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .unwrap();

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &semantic_pairs,
        ProgrammingLanguage::Go,
        &old_source,
        &new_source,
    );

    // EXPECTED BEHAVIOR: Should have ~5 semantic boundaries:
    // 1. Import block changes (single import -> multiple imports)
    // 2. Server struct definition
    // 3. NewServer function
    // 4. AddRoute method
    // 5. Main function modifications
    //
    // ACTUAL BEHAVIOR: Creates 8+ boundaries with duplicates and over-decomposition

    // This should pass when the bug is fixed
    assert!(
        reviewable_diffs.len() <= 6,
        "Expected at most 6 semantic boundaries, got {}. Boundaries: {:#?}",
        reviewable_diffs.len(),
        reviewable_diffs
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<_>>()
    );

    // Verify no duplicate package boundaries
    let package_boundaries: Vec<&_> = reviewable_diffs
        .iter()
        .filter(|d| d.boundary.node_type.contains("package"))
        .collect();

    assert!(
        package_boundaries.len() <= 1,
        "Expected at most 1 package boundary, got {}: {:?}",
        package_boundaries.len(),
        package_boundaries
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<&String>>()
    );

    // Verify struct is not over-decomposed
    let struct_boundaries: Vec<&_> = reviewable_diffs
        .iter()
        .filter(|d| {
            d.boundary.node_type.contains("struct") || d.boundary.node_type.contains("Server")
        })
        .collect();

    assert!(
        struct_boundaries.len() <= 2,
        "Expected at most 2 struct-related boundaries, got {}: {:?}",
        struct_boundaries.len(),
        struct_boundaries
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<&String>>()
    );
}

/// Test for simpler Go struct addition to verify basic decomposition issues
#[test]
#[ignore = "Bug #3: Go semantic analysis creates duplicate boundaries and over-decomposition"]
fn bug_3_go_simple_struct_addition() {
    let old_code = r#"package main

func main() {
    println("Hello")
}
"#;

    let new_code = r#"package main

type User struct {
    Name string
    Age  int
}

func main() {
    println("Hello")
}
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    let parser = GoParser::new();
    let mut ts_parser = tree_sitter::Parser::new();
    ts_parser.set_language(parser.get_language()).unwrap();

    let old_tree = ts_parser.parse(old_code, None).unwrap();
    let new_tree = ts_parser.parse(new_code, None).unwrap();

    let old_semantic = parser.build_semantic_tree(&old_tree, old_code).unwrap();
    let new_semantic = parser.build_semantic_tree(&new_tree, new_code).unwrap();

    let semantic_pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .unwrap();

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &semantic_pairs,
        ProgrammingLanguage::Go,
        &old_source,
        &new_source,
    );

    // EXPECTED: Should have exactly 1 boundary for the new struct
    // ACTUAL: May create multiple boundaries or decompose struct incorrectly

    assert!(
        reviewable_diffs.len() <= 2,
        "Expected at most 2 semantic boundaries for adding one struct, got {}: {:?}",
        reviewable_diffs.len(),
        reviewable_diffs
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<_>>()
    );

    // Verify struct is properly identified
    let has_struct_boundary = reviewable_diffs
        .iter()
        .any(|d| d.boundary.node_type.contains("struct") || d.boundary.node_type.contains("User"));

    assert!(
        has_struct_boundary,
        "Expected to find a struct-related boundary, got: {:?}",
        reviewable_diffs
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<_>>()
    );
}

/// Test for Go package preservation (should not show as delete+add)
#[test]
#[ignore = "Bug #3: Go semantic analysis creates duplicate boundaries and over-decomposition"]
fn bug_3_go_package_preservation() {
    let old_code = r#"package main

import "fmt"

func hello() {
    fmt.Println("Hello")
}
"#;

    let new_code = r#"package main

import "fmt"

func hello() {
    fmt.Println("Hello")
}

func world() {
    fmt.Println("World")
}
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    let parser = GoParser::new();
    let mut ts_parser = tree_sitter::Parser::new();
    ts_parser.set_language(parser.get_language()).unwrap();

    let old_tree = ts_parser.parse(old_code, None).unwrap();
    let new_tree = ts_parser.parse(new_code, None).unwrap();

    let old_semantic = parser.build_semantic_tree(&old_tree, old_code).unwrap();
    let new_semantic = parser.build_semantic_tree(&new_tree, new_code).unwrap();

    let semantic_pairs = build_semantic_pairs(
        &old_semantic,
        &new_semantic,
        &old_source,
        &new_source,
        &parser,
    )
    .unwrap();

    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &semantic_pairs,
        ProgrammingLanguage::Go,
        &old_source,
        &new_source,
    );

    // EXPECTED: Should have exactly 1 boundary for the new function
    // Package and import should be unchanged (not show as separate boundaries)
    // ACTUAL: May show package as deleted+added, creating extra boundaries

    assert_eq!(
        reviewable_diffs.len(),
        1,
        "Expected exactly 1 semantic boundary for adding one function, got {}: {:?}",
        reviewable_diffs.len(),
        reviewable_diffs
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<_>>()
    );

    assert!(
        reviewable_diffs[0].boundary.node_type.contains("world")
            || reviewable_diffs[0].boundary.node_type.contains("function"),
        "Expected boundary to be for 'world' function, got: {}",
        reviewable_diffs[0].boundary.node_type
    );
}
