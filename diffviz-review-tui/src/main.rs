//! Test-only TUI binary for predictable manual testing
//!
//! This standalone binary always uses curated fixtures from diffviz-review
//! to provide predictable, reproducible content for manual TUI validation.
//! Never requires a git repository - always works with the same test data.

use anyhow::Result;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::{
    ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange, DiffQuery, GitRef,
    ReviewDecisions, ReviewEngineBuilder,
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

    // Standard ReviewEngine creation using ReviewEngineBuilder
    let review_engine_builder =
        ReviewEngineBuilder::new(Box::new(mock_provider), "dev-user".to_string());
    let diff_query = DiffQuery::new(GitRef::Head, GitRef::Unstaged);
    let mut review_engine = review_engine_builder
        .build(diff_query)
        .map_err(|e| anyhow::anyhow!("Failed to build ReviewEngine: {}", e))?;

    // Phase 1: Add hardcoded decision data for TUI testing
    // Use set_decisions_with_index() to automatically build decision index by detecting
    // overlaps between CodeImpact line ranges and actual ReviewableDiffs in the review state
    let decisions = create_hardcoded_decisions();
    review_engine.set_decisions_with_index(decisions);

    Ok(review_engine)
}

/// Create hardcoded decisions for Phase 1 TUI testing
/// These demonstrate decision-based review structure without requiring JSON loading
fn create_hardcoded_decisions() -> ReviewDecisions {
    let mut decisions = ReviewDecisions::new();

    // Decision 1: Refactor user model module
    decisions.add_decision(Decision {
        number: 1,
        title: "Refactor user model module".to_string(),
        summary: "Extract user model logic into separate, testable module".to_string(),
        decision_log_line: Some(15),
        code_impacts: vec![
            CodeImpact {
                file: "src/models/user.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 21 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "User model structure refactoring".to_string(),
            },
            CodeImpact {
                file: "src/config/reader.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 1, end: 7 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "Configuration reader updates".to_string(),
            },
        ],
    });

    // Decision 2: Improve network client error handling
    decisions.add_decision(Decision {
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
                file: "src/models/user.rs".to_string(),
                line_ranges: vec![DecisionLineRange { start: 10, end: 21 }],
                change_type: ChangeType::Modification,
                confidence: Confidence::High,
                reasoning: "User model error handling enhancements".to_string(),
            },
        ],
    });

    // Decision 3: Add logging infrastructure
    decisions.add_decision(Decision {
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
                line_ranges: vec![DecisionLineRange { start: 1, end: 17 }],
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
        ],
    });

    decisions
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
