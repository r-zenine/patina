//! Integration test against a real `rust-analyzer` process. Requires
//! `rust-analyzer` on `PATH`; marked `#[ignore]` since CI may not have it
//! installed. Run explicitly with:
//!
//! ```sh
//! cargo test -p lspkit --test references_integration -- --ignored
//! ```

use lspkit::{FileLocation, Location, LspClient, Position};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references_fixture")
}

/// rust-analyzer answers `initialize` before it has finished loading the
/// crate graph, so the first `references` call after a fresh `start()` can
/// race indexing and come back with a transient "file not found" server
/// error. Retrying (test-only — production code stays fail-fast) is the
/// standard way LSP integration tests ride out this warm-up window.
fn references_with_retry(client: &LspClient, at: &FileLocation) -> Vec<Location> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.references(at, false);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(locations) if !locations.is_empty() => return locations,
            Ok(locations) if past_deadline => {
                panic!("references stayed empty until the deadline: {locations:?}")
            }
            Err(err) if past_deadline => {
                panic!("references never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

/// Regression test for the indexing-wait race: `start()` must block until
/// rust-analyzer reports `quiescent: true` (`experimental/serverStatus`), so
/// the very first `references` call — deliberately no retry here — already
/// sees cross-file references, including struct-field accesses. The earlier
/// `$/progress`-counting wait returned ~500ms after spawn (in the gap
/// between two progress streams), and every call answered from an empty
/// index: 0 references for everything.
#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn first_call_after_start_sees_cross_file_field_references() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    // `Item::weight` field declaration in src/types.rs, referenced only by
    // the `item.weight` field access in src/lib.rs's `total_weight`.
    let at = FileLocation {
        path: root.join("src/types.rs"),
        position: Position {
            line: 2,
            character: 9,
        },
    };

    let locations = client
        .references(&at, false)
        .expect("references should succeed on the first call after start()");
    assert_eq!(
        locations.len(),
        1,
        "expected the single cross-file field access, got {locations:?}"
    );
    assert!(
        locations[0].path.ends_with("src/lib.rs"),
        "expected the reference in src/lib.rs, got {locations:?}"
    );
}

/// Regression test for feature-gated references: `gated_target` is only
/// called from `gated_caller`, which sits behind the fixture's non-default
/// `extra` feature. Without `cargo.features = "all"` in the initialize
/// options, rust-analyzer cfg's that caller out of the crate graph and
/// silently reports 0 references — exactly how feature-gated test harnesses
/// and their integration tests were misreported as dead exports.
#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn references_include_call_sites_behind_non_default_features() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    let at = FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 55,
            character: 8,
        },
    };

    let locations = client
        .references(&at, false)
        .expect("references should succeed on the first call after start()");
    assert_eq!(
        locations.len(),
        1,
        "expected the feature-gated call site, got {locations:?}"
    );
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn references_finds_known_call_sites() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    let at = FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 1,
            character: 8,
        },
    };

    let mut locations = references_with_retry(&client, &at);
    locations.sort_by_key(|loc| loc.range.start.line);

    assert_eq!(
        locations.len(),
        2,
        "expected exactly 2 call sites, got {locations:?}"
    );
    assert_eq!(locations[0].range.start.line, 6);
    assert_eq!(locations[1].range.start.line, 10);
    for loc in &locations {
        assert_eq!(loc.range.start.character, 5);
    }
}
