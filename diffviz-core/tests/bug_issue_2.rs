//! Test for Python semantic analysis false positives bug
//!
//! This test reproduces a bug where DiffViz incorrectly over-decomposes Python code
//! into excessive semantic boundaries, creating false positives. Specifically:
//! 1. Enum values are treated as separate functions and duplicated
//! 2. Individual statements are treated as separate semantic boundaries
//! 3. File modifications are treated as delete+add instead of modifications
//! 4. Class definitions are not properly grouped semantically

use diffviz_core::{
    ast_diff::SourceCode,
    common::{LanguageParser, ProgrammingLanguage},
    parsers::PythonParser,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

/// Test that Python semantic analysis produces correct semantic boundaries
///
/// This test reproduces the false positive where adding a class and enum to a basic
/// Python script results in 20 semantic boundaries instead of the expected ~5-6.
#[test]
#[ignore = "Bug #2: Python semantic analysis creates excessive false positive boundaries"]
fn bug_2_python_excessive_semantic_boundaries() {
    // Create old version: basic Python script with just main block
    let old_code = r#"#!/usr/bin/env python3
"""A simple todo list manager with priority support."""

if __name__ == "__main__":
    print("Todo Manager starting...")
"#;

    // Create new version: adds imports, enum, class with methods
    let new_code = r#"#!/usr/bin/env python3
"""A simple todo list manager with priority support."""

from datetime import datetime
from enum import Enum, auto
from typing import List, Optional


class Priority(Enum):
    LOW = auto()
    MEDIUM = auto()
    HIGH = auto()


class TodoItem:
    def __init__(self, title: str, description: str = "", priority: Priority = Priority.MEDIUM):
        self.title = title
        self.description = description
        self.priority = priority
        self.created_at = datetime.now()
        self.completed = False

    def mark_completed(self):
        self.completed = True

    def __str__(self):
        status = "✓" if self.completed else "○"
        return f"{status} [{self.priority.name}] {self.title}"


if __name__ == "__main__":
    print("Todo Manager starting...")
    todo = TodoItem("Test task", "This is a test")
    print(todo)
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    // Parse and analyze using Python parser
    let parser = PythonParser::new();
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
        ProgrammingLanguage::Python,
        &old_source,
        &new_source,
        &parser,
    );

    // EXPECTED BEHAVIOR: Should have ~5-6 semantic boundaries:
    // 1. Import additions (3 imports grouped or individual)
    // 2. Priority enum definition
    // 3. TodoItem class with methods
    // 4. Main block modifications
    //
    // ACTUAL BEHAVIOR: Creates 20+ boundaries with duplicates and over-decomposition

    // This should pass when the bug is fixed
    assert!(
        reviewable_diffs.len() <= 8,
        "Expected at most 8 semantic boundaries, got {}. Boundaries: {:#?}",
        reviewable_diffs.len(),
        reviewable_diffs
            .iter()
            .map(|d| &d.boundary.node_type)
            .collect::<Vec<_>>()
    );

    // Verify no duplicate boundaries
    let boundary_names: Vec<_> = reviewable_diffs
        .iter()
        .map(|d| &d.boundary.node_type)
        .collect();

    let mut unique_names = boundary_names.clone();
    unique_names.sort();
    unique_names.dedup();

    assert_eq!(
        boundary_names.len(),
        unique_names.len(),
        "Found duplicate semantic boundaries: {boundary_names:?}"
    );

    // Verify enum values are not treated as separate functions
    let function_like_boundaries: Vec<&_> = reviewable_diffs
        .iter()
        .filter(|d| d.boundary.node_type.contains("auto()"))
        .collect();

    assert!(
        function_like_boundaries.is_empty(),
        "Enum values should not be treated as separate boundaries: count={}",
        function_like_boundaries.len()
    );
}

/// Test case for simpler Python changes to verify granular issues
#[test]
#[ignore = "Bug #2: Python semantic analysis creates excessive false positive boundaries"]
fn bug_2_python_simple_function_addition() {
    let old_code = r#"def hello():
    print("Hello")
"#;

    let new_code = r#"def hello():
    print("Hello")

def world():
    print("World")
"#;

    let old_source = SourceCode::new(old_code);
    let new_source = SourceCode::new(new_code);

    let parser = PythonParser::new();
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
        ProgrammingLanguage::Python,
        &old_source,
        &new_source,
        &parser,
    );

    // EXPECTED: Should have exactly 1 boundary for the new function
    // ACTUAL: May create multiple boundaries or decompose incorrectly

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
        reviewable_diffs[0].boundary.node_type.contains("world"),
        "Expected boundary to be for 'world' function, got: {}",
        reviewable_diffs[0].boundary.node_type
    );
}
