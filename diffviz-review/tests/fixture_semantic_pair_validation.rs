use diffviz_review::entities::git_ref::DiffQuery;
use diffviz_review::providers::mock_provider::{MockDiffProvider, ReviewFixture};
use diffviz_review::review_engine_builder::ReviewEngineBuilder;
use std::collections::HashMap;
use std::path::PathBuf;

/// Loads all fixtures from the fixtures directory
fn load_all_fixtures() -> Vec<(String, ReviewFixture)> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let mut fixtures = Vec::new();

    for entry in std::fs::read_dir(&fixtures_dir).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let fixture_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let fixture_json = std::fs::read_to_string(&path)
                .expect(&format!("Failed to read fixture file: {:?}", path));

            let fixture: ReviewFixture = serde_json::from_str(&fixture_json)
                .expect(&format!("Failed to parse fixture JSON: {:?}", path));

            fixtures.push((fixture_name, fixture));
        }
    }

    fixtures.sort_by(|a, b| a.0.cmp(&b.0));
    fixtures
}

#[test]
#[ignore = "Bug: typescript_react_component fixture fails to produce semantic pairs"]
fn test_each_fixture_produces_semantic_pairs() {
    // Load all fixtures
    let fixtures = load_all_fixtures();
    println!(
        "\n=== Testing {} Fixtures for Semantic Pair Production ===\n",
        fixtures.len()
    );

    // Build a single review engine with all fixtures
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load fixtures");
    let builder = ReviewEngineBuilder::new(Box::new(mock_provider), "test_author".to_string());
    let engine = builder
        .build(DiffQuery::head_to_unstaged())
        .expect("Failed to build review engine");

    let all_diffs = &engine.state().reviewable_diffs;

    // Group semantic pairs by file path
    let mut pairs_by_file: HashMap<String, Vec<_>> = HashMap::new();
    for (id, diff) in all_diffs.iter() {
        let file_path = id.file_path.split('#').next().unwrap_or("unknown").to_string();
        pairs_by_file
            .entry(file_path)
            .or_default()
            .push((id, diff));
    }

    // Track results
    let mut passing_fixtures = Vec::new();
    let mut failing_fixtures = Vec::new();

    // Check each fixture
    for (fixture_name, fixture) in &fixtures {
        let pairs = pairs_by_file.get(&fixture.file_path).map(|v| v.len()).unwrap_or(0);

        if pairs > 0 {
            println!(
                "✓ {} - {} semantic pairs produced",
                fixture_name, pairs
            );
            println!("  File: {}", fixture.file_path);
            println!("  Language: {}", fixture.language);

            if let Some(pairs_list) = pairs_by_file.get(&fixture.file_path) {
                for (id, reviewable_diff) in pairs_list {
                    let core_diff = &reviewable_diff.core_diff;
                    let boundary_node = &core_diff.boundary;
                    println!(
                        "    • {:?} ({:?}) @ L{}-{}",
                        boundary_node.semantic_kind,
                        boundary_node.change_status,
                        id.line_range.start_line,
                        id.line_range.end_line
                    );
                }
            }
            println!();
            passing_fixtures.push(fixture_name.clone());
        } else {
            println!("✗ {} - NO SEMANTIC PAIRS PRODUCED", fixture_name);
            println!("  File: {}", fixture.file_path);
            println!("  Language: {}", fixture.language);
            println!("  Description: {}", fixture.description);
            println!("  Expected changes: +{} -{}",
                fixture.expected_line_stats.additions,
                fixture.expected_line_stats.deletions
            );
            println!();
            failing_fixtures.push(fixture_name.clone());
        }
    }

    // Print summary
    println!("\n=== Summary ===");
    println!("Total fixtures: {}", fixtures.len());
    println!("Passing fixtures: {} ({}%)",
        passing_fixtures.len(),
        (passing_fixtures.len() * 100) / fixtures.len()
    );
    println!("Failing fixtures: {} ({}%)",
        failing_fixtures.len(),
        (failing_fixtures.len() * 100) / fixtures.len()
    );

    if !failing_fixtures.is_empty() {
        println!("\nFixtures that failed to produce semantic pairs:");
        for name in &failing_fixtures {
            println!("  - {}", name);
        }
    }

    // Assert that all fixtures produced at least one semantic pair
    assert!(
        failing_fixtures.is_empty(),
        "The following fixtures failed to produce semantic pairs: {:?}",
        failing_fixtures
    );
}

#[test]
fn test_overall_semantic_pair_count() {
    let mock_provider =
        MockDiffProvider::from_review_fixtures().expect("Failed to load fixtures");
    let builder = ReviewEngineBuilder::new(Box::new(mock_provider), "test_author".to_string());
    let engine = builder
        .build(DiffQuery::head_to_unstaged())
        .expect("Failed to build review engine");

    let total_pairs = engine.state().reviewable_diffs.len();

    println!("\nTotal semantic pairs produced across all fixtures: {}", total_pairs);

    // Assert we got a reasonable number of pairs
    assert!(
        total_pairs >= 8,
        "Expected at least 8 semantic pairs (one per fixture), got {}",
        total_pairs
    );
}
