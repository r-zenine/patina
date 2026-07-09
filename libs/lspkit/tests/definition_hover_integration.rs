//! Integration test against a real `rust-analyzer` process. Requires
//! `rust-analyzer` on `PATH`; marked `#[ignore]` since CI may not have it
//! installed. Run explicitly with:
//!
//! ```sh
//! cargo test -p lspkit --test definition_hover_integration -- --ignored
//! ```

use lspkit::{FileLocation, Hover, Location, LspClient, Position};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references_fixture")
}

/// Position of the `Shape` type annotation on `describe`'s parameter — the
/// scrutinee's type at the match site Phase 13 will resolve.
fn shape_usage_site(root: &std::path::Path) -> FileLocation {
    FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 30,
            character: 26,
        },
    }
}

/// Same rust-analyzer indexing race as `references_integration`'s retry
/// helper — the first request after a fresh `start()` can come back empty
/// or error out while indexing finishes.
fn definition_with_retry(client: &LspClient, at: &FileLocation) -> Vec<Location> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.definition(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(locations) if !locations.is_empty() => return locations,
            Ok(locations) if past_deadline => {
                panic!("definition stayed empty until the deadline: {locations:?}")
            }
            Err(err) if past_deadline => {
                panic!("definition never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn hover_with_retry(client: &LspClient, at: &FileLocation) -> Hover {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.hover(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(Some(hover)) => return hover,
            Ok(None) if past_deadline => panic!("hover stayed empty until the deadline"),
            Err(err) if past_deadline => panic!("hover never resolved before the deadline: {err}"),
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn definition_resolves_match_scrutinee_type_to_the_enum_declaration() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");
    let at = shape_usage_site(&root);

    let locations = definition_with_retry(&client, &at);

    assert_eq!(
        locations.len(),
        1,
        "expected exactly one definition site for Shape, got {locations:?}"
    );
    assert_eq!(locations[0].range.start.line, 25);
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn hover_returns_the_enum_signature() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");
    let at = shape_usage_site(&root);

    let hover = hover_with_retry(&client, &at);

    assert!(
        hover.signature.contains("Shape"),
        "expected hover signature to mention Shape, got {:?}",
        hover.signature
    );
}
