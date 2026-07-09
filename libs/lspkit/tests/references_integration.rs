//! Integration test against a real `rust-analyzer` process. Requires
//! `rust-analyzer` on `PATH`; marked `#[ignore]` since CI may not have it
//! installed. Run explicitly with:
//!
//! ```sh
//! cargo test -p lspkit --test references_integration -- --ignored
//! ```

use lspkit::{FileLocation, LspClient, Position};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references_fixture")
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

    let mut locations = client
        .references(&at, false)
        .expect("references should resolve for a real symbol");
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
