//! Test-only TUI binary for predictable manual testing
//!
//! This standalone binary always uses curated fixtures from diffviz-review
//! to provide predictable, reproducible content for manual TUI validation.
//! Never requires a git repository - always works with the same test data.

use anyhow::Result;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewEngineBuilder,
};
use diffviz_review_tui::ReviewTuiApp;
use std::env;

fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    // Parse command line arguments for test modes (feature-gated)
    #[cfg_attr(not(feature = "test-harness"), allow(unused_variables))]
    let args: Vec<String> = env::args().collect();

    #[cfg(feature = "test-harness")]
    {
        if args.len() > 1 {
            match args[1].as_str() {
                "--test-input" => {
                    if args.len() < 3 {
                        eprintln!("Usage: {} --test-input <input_sequence>", args[0]);
                        eprintln!("Example: {} --test-input \"jjk<Enter>\"", args[0]);
                        std::process::exit(1);
                    }
                    return run_input_test(&args[2]);
                }
                "--test-render" => {
                    if args.len() < 3 {
                        eprintln!("Usage: {} --test-render <state.json>", args[0]);
                        eprintln!("Note: Not yet implemented - render from input sequence instead");
                        std::process::exit(1);
                    }
                    return run_render_test(&args[2]);
                }
                "--test-full" => {
                    if args.len() < 3 {
                        eprintln!("Usage: {} --test-full <input_sequence>", args[0]);
                        eprintln!("Example: {} --test-full \"jjk<Enter>\"", args[0]);
                        std::process::exit(1);
                    }
                    return run_combined_test(&args[2]);
                }
                _ => {
                    // Fall through to normal TUI mode
                }
            }
        }
    }

    // Normal interactive mode
    let review_engine = create_test_review_engine()?;
    let mut app = ReviewTuiApp::new(review_engine)?;
    app.run()?;

    Ok(())
}

/// Create a test ReviewEngine using curated fixtures
fn create_test_review_engine() -> Result<diffviz_review::engines::ReviewEngine> {
    // Use curated fixtures from diffviz-review crate
    let mock_provider = MockDiffProvider::from_review_fixtures()
        .map_err(|e| anyhow::anyhow!("Failed to load test fixtures: {}", e))?;

    // Use decision-based ReviewEngine creation
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "dev-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);

    // Create hardcoded decisions for TUI testing
    let decisions = create_hardcoded_decisions_vec();

    // Build ReviewEngine using the decision-based pipeline
    let review_engine = review_engine_builder
        .build_from_decisions(decisions, diff_query)
        .map_err(|e| anyhow::anyhow!("Failed to build ReviewEngine from decisions: {}", e))?;

    Ok(review_engine)
}

/// Create hardcoded decisions for TUI testing
/// Returns Vec<Decision> for use with build_from_decisions()
fn create_hardcoded_decisions_vec() -> Vec<Decision> {
    vec![
        // Decision 1: Refactor calculator model module
        Decision {
            number: 1,
            title: "Refactor calculator model module".to_string(),
            summary: "Extract calculator logic into separate, testable module".to_string(),
            decision_log_line: Some(15),
            code_impacts: vec![
                CodeImpact {
                    file: "src/models/calculator.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 72 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Calculator model structure refactoring".to_string(),
                },
                CodeImpact {
                    file: "src/config/reader.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 7 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Configuration reader updates".to_string(),
                },
            ],
        },
        // Decision 2: Improve network client error handling
        Decision {
            number: 2,
            title: "Improve error handling in network client".to_string(),
            summary: "Standardize error types and add context to error messages".to_string(),
            decision_log_line: Some(28),
            code_impacts: vec![
                CodeImpact {
                    file: "src/network/client.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 6 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::Medium,
                    reasoning: "Network error handling improvements".to_string(),
                },
                CodeImpact {
                    file: "src/models/calculator.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 20, end: 40 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Calculator model error handling enhancements".to_string(),
                },
            ],
        },
        // Decision 3: Add logging infrastructure
        Decision {
            number: 3,
            title: "Add structured logging throughout application".to_string(),
            summary: "Architectural decision: use tracing crate for observability".to_string(),
            decision_log_line: Some(42),
            code_impacts: vec![
                CodeImpact {
                    file: "src/data/fetcher.py".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 5 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Add logging to async data fetching operations".to_string(),
                },
                CodeImpact {
                    file: "src/components/Greeting.tsx".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 50 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::Medium,
                    reasoning: "Add component lifecycle logging".to_string(),
                },
                CodeImpact {
                    file: "src/types/api.ts".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 9 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Add API type validation logging".to_string(),
                },
                // Phase 6: Enhanced calculator fixture for context folding validation
                CodeImpact {
                    file: "src/models/calculator.rs".to_string(),
                    line_ranges: vec![DecisionLineRange { start: 1, end: 72 }],
                    change_type: ChangeType::Modification,
                    confidence: Confidence::High,
                    reasoning: "Calculator module with extensive context for folding test".to_string(),
                },
            ],
        },
    ]
}

/// Run input sequence test mode (feature-gated)
#[cfg(feature = "test-harness")]
fn run_input_test(input_sequence: &str) -> Result<()> {
    use diffviz_review_tui::test_harness::InputTestHarness;

    let review_engine = create_test_review_engine()?;
    let mut harness = InputTestHarness::new(review_engine);

    match harness.run_sequence_final_state(input_sequence) {
        Ok(snapshot) => {
            let json = snapshot.to_json()?;
            println!("{}", json);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error running input test: {}", e);
            Err(e)
        }
    }
}

/// Run render test mode (feature-gated)
#[cfg(feature = "test-harness")]
fn run_render_test(state_json: &str) -> Result<()> {
    use diffviz_review_tui::test_harness::StateSnapshot;
    use std::fs;

    // Try to read as file first, then as inline JSON
    let json_str = if let Ok(content) = fs::read_to_string(state_json) {
        content
    } else {
        state_json.to_string()
    };

    let _snapshot: StateSnapshot = serde_json::from_str(&json_str)?;

    eprintln!("Render test mode requires state reconstruction from JSON.");
    eprintln!("This is not yet fully implemented.");
    eprintln!("Use --test-full instead to capture state and visual output together.");

    Ok(())
}

/// Run combined test mode (feature-gated)
#[cfg(feature = "test-harness")]
fn run_combined_test(input_sequence: &str) -> Result<()> {
    use diffviz_review_tui::test_harness::CombinedTestHarness;

    let review_engine = create_test_review_engine()?;
    let mut harness = CombinedTestHarness::new(review_engine);

    match harness.run_sequence_with_renders(input_sequence) {
        Ok(results) => {
            for (i, result) in results.iter().enumerate() {
                println!("=== Step {} ===", i);
                println!("State:");
                let json = result.state.to_json()?;
                println!("{}", json);
                println!("\nVisual:");
                println!("{}", result.visual);
                println!();
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error running combined test: {}", e);
            Err(e)
        }
    }
}
