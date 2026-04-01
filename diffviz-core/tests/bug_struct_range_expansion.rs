//! Regression test for struct range expansion bug
//!
//! Bug: When specifying a code impact range that only covers a struct declaration
//! (e.g., lines 34-35 for the #[derive] + struct keyword), create_reviewable_diff_from_range()
//! fails to expand to the full struct definition including all fields and their attributes.
//!
//! Expected behavior: The range should expand to cover the entire struct boundary,
//! including all fields, decorators, and the closing brace.

use diffviz_core::ast_diff::SourceCode;
use diffviz_core::common::ProgrammingLanguage;
use diffviz_core::decision_based_diff::create_reviewable_diff_from_range;
use diffviz_core::parsers::RustParser;

#[test]
fn test_struct_declaration_range_should_expand_to_full_struct() {
    // Rust code with a struct that has field-level attributes
    // Lines are 1-based for the user-facing API
    let old_source = r#"pub struct CodeImpact {
    pub file: String,
    pub line_ranges: Vec<DecisionLineRange>,
    pub reasoning: String,
}
"#;

    let new_source = r#"#[derive(Debug, Clone, SchemaTemplate)]
pub struct CodeImpact {
    #[schema(example = "...", comment = "...")]
    pub reasoning: String,

    #[schema(example = "...", comment = "...")]
    pub file: String,

    pub line_ranges: Vec<DecisionLineRange>,
}
"#;

    let old_provider = SourceCode::new(old_source.to_string());
    let new_provider = SourceCode::new(new_source.to_string());

    let parser = RustParser::new();

    // User specifies only lines 1-2 (the #[derive] and struct keyword)
    // The system should automatically expand this to include the full struct (lines 1-10)
    let start_line = 1;
    let end_line = 2;

    let result = create_reviewable_diff_from_range(
        "test.rs",
        start_line,
        end_line,
        Some(&old_provider),
        &new_provider,
        ProgrammingLanguage::Rust,
        &parser,
    );

    // The bug: this returns an error or empty vec because the range is too narrow
    // Expected: should return at least one ReviewableDiff representing the struct change
    assert!(
        result.is_ok(),
        "Range expansion should succeed for struct declaration. Error: {:?}",
        result.err()
    );

    let diffs = result.unwrap();
    assert!(
        !diffs.is_empty(),
        "Should produce at least one diff for modified struct with expanded range"
    );

    // The diff should represent the entire struct, not just the declaration line
    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];

    // Verify it captured the struct modification
    assert_eq!(diff.language, ProgrammingLanguage::Rust);
}

#[test]
fn test_decision_log_scenario_struct_range_should_expand() {
    // This reproduces the actual bug from decision-log.yaml:
    // Decision 2 specifies lines 34-35 for CodeImpact struct in diffviz-review/src/entities/decision.rs
    // But the actual changes span lines 34-52 (full struct definition with all field attributes)

    let old_source = r#"#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeImpact {
    /// Path to the source file relative to repository root. Example: "src/auth/middleware.rs"
    pub file: String,

    /// Line ranges within this file affected by the decision
    pub line_ranges: Vec<DecisionLineRange>,

    /// Explanation of why this decision impacts these specific lines.
    /// Example: "Middleware validates JWT tokens and injects user context"
    pub reasoning: String,
}
"#;

    let new_source = r#"#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaTemplate)]
pub struct CodeImpact {
    ////// Explanation of why this decision impacts these specific lines
    #[schema(
        example = "Middleware validates JWT tokens and injects user context",
        comment = "Why this file is affected by this decision"
    )]
    pub reasoning: String,

    /// Path to the source file relative to repository root
    #[schema(
        example = "src/auth/middleware.rs",
        comment = "Path to the source file relative to repository root"
    )]
    pub file: String,

    /// Line ranges within this file affected by the decision
    pub line_ranges: Vec<DecisionLineRange>,
}
"#;

    let old_provider = SourceCode::new(old_source.to_string());
    let new_provider = SourceCode::new(new_source.to_string());

    let parser = RustParser::new();

    // User specifies only lines 1-2 (corresponding to actual decision-log lines 34-35)
    // which is just the #[derive] attribute and struct keyword
    let start_line = 1;
    let end_line = 2;

    let result = create_reviewable_diff_from_range(
        "decision.rs",
        start_line,
        end_line,
        Some(&old_provider),
        &new_provider,
        ProgrammingLanguage::Rust,
        &parser,
    );

    // BUG: This returns an empty vec or error because the narrow range doesn't capture enough context
    // EXPECTED: Should expand the range to the full struct boundary and return a diff
    assert!(
        result.is_ok(),
        "Should successfully expand struct range. Error: {:?}",
        result.err()
    );

    let diffs = result.unwrap();
    assert!(
        !diffs.is_empty(),
        "Range should expand to full struct - CodeImpact struct spans 19 lines with 5 attributes,\
         but range was only 1-2. System should automatically expand to capture full semantic unit."
    );
}
