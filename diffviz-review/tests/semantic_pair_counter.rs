use diffviz_review::review_engine_builder::ReviewEngineBuilder;
use diffviz_review::providers::mock_provider::MockDiffProvider;
use diffviz_review::entities::git_ref::DiffQuery;

#[test]
fn count_semantic_pairs_all_fixtures() {
    // Load fixtures
    let mock_provider = MockDiffProvider::from_review_fixtures()
        .expect("Failed to load fixtures");

    // Create builder
    let builder = ReviewEngineBuilder::new(
        Box::new(mock_provider),
        "test_author".to_string(),
    );

    // Build the review engine which runs the full pipeline
    let engine = builder
        .build(DiffQuery::head_to_unstaged())
        .expect("Failed to build review engine");

    // Get reviewable diffs which represent the semantic pairs
    let reviewable_diffs = &engine.state().reviewable_diffs;

    println!("\n=== Semantic Pair Analysis (All Fixtures) ===");
    println!("Total semantic pairs produced: {}\n", reviewable_diffs.len());

    // Group by file
    let mut files: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for (id, reviewable_diff) in reviewable_diffs.iter() {
        let file = id.file_path.clone().split('#').next().unwrap_or("unknown").to_string();
        files.entry(file).or_default().push((id, reviewable_diff));
    }

    // Print summary by file
    for (file, diffs) in files.iter() {
        println!("File: {} - {} semantic pairs", file, diffs.len());
        for (id, reviewable_diff) in diffs {
            let core_diff = &reviewable_diff.core_diff;
            let boundary_node = &core_diff.boundary;
            println!("  • {:?} ({:?}) @ L{}-{}",
                boundary_node.semantic_kind,
                boundary_node.change_status,
                id.line_range.start_line,
                id.line_range.end_line
            );
        }
        println!();
    }

    // Assert we got some pairs
    let total = reviewable_diffs.len();
    assert!(total > 0, "Should have produced at least one semantic pair");
    println!("✓ Test passed: {} total semantic pairs produced", total);
}

#[test]
fn count_semantic_pairs_typescript_react_component_only() {
    use diffviz_review::providers::mock_provider::ReviewFixture;
    use std::path::PathBuf;

    // Manually load just the TypeScript React component fixture
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("typescript_react_component.json");

    let fixture_json = std::fs::read_to_string(&fixture_path)
        .expect("Failed to read fixture file");

    let fixture: ReviewFixture = serde_json::from_str(&fixture_json)
        .expect("Failed to parse fixture JSON");

    // Create a mock provider with just this fixture
    // We use from_review_fixtures and filter to the specific fixture
    let all_fixtures = MockDiffProvider::from_review_fixtures()
        .expect("Failed to load fixtures");

    // Use a workaround: create a mock with the necessary state
    // For now, let's test by using from_review_fixtures but filtering results
    println!("\nNote: Using all fixtures approach, filtering to TypeScript React only...\n");
    let engine = {
        let builder = ReviewEngineBuilder::new(
            Box::new(all_fixtures),
            "test_author".to_string(),
        );
        builder.build(DiffQuery::head_to_unstaged())
            .expect("Failed to build review engine")
    };

    let all_diffs = &engine.state().reviewable_diffs;

    // Filter to TypeScript React component only
    let mut reviewable_diffs_filtered = std::collections::BTreeMap::new();
    for (id, diff) in all_diffs.iter() {
        if id.file_path.contains("Greeting.tsx") {
            reviewable_diffs_filtered.insert(id.clone(), diff.clone());
        }
    }
    let reviewable_diffs = &reviewable_diffs_filtered;

    println!("\n=== TypeScript React Component Semantic Pair Analysis ===");
    println!("Fixture: {}", fixture.name);
    println!("File: {}", fixture.file_path);
    println!("Language: {}", fixture.language);
    println!("Description: {}", fixture.description);
    println!("Complexity: {}", fixture.metadata.complexity_level);
    println!("Tags: {}\n", fixture.metadata.tags.join(", "));

    println!("Total semantic pairs produced: {}\n", reviewable_diffs.len());

    // Print details for each semantic pair
    for (id, reviewable_diff) in reviewable_diffs.iter() {
        let core_diff = &reviewable_diff.core_diff;
        let boundary_node = &core_diff.boundary;

        println!("Semantic Pair #{}", id.file_path.split('#').last().unwrap_or("?"));
        println!("  Semantic kind: {:?}", boundary_node.semantic_kind);
        println!("  Change status: {:?}", boundary_node.change_status);
        println!("  Relevance score: {}", boundary_node.relevance);
        println!("  Line range: L{}-{}", id.line_range.start_line, id.line_range.end_line);
        println!();
    }

    // Assert we got some pairs
    let total = reviewable_diffs.len();
    assert!(total > 0, "Should have produced at least one semantic pair");
    println!("✓ Test passed: {} semantic pairs produced for TypeScript React component", total);
}
