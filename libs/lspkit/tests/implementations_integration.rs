//! Integration test against a real `rust-analyzer` process. Requires
//! `rust-analyzer` on `PATH`; marked `#[ignore]` since CI may not have it
//! installed. Run explicitly with:
//!
//! ```sh
//! cargo test -p lspkit --test implementations_integration -- --ignored
//! ```

use lspkit::{FileLocation, Location, LspClient, Position};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references_fixture")
}

/// Position of the `Greeter` trait name in its declaration.
fn greeter_trait_site(root: &std::path::Path) -> FileLocation {
    FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 37,
            character: 11,
        },
    }
}

/// Same rust-analyzer indexing race as `references_integration`'s retry
/// helper — the first request after a fresh `start()` can come back empty
/// or error out while indexing finishes.
fn implementations_with_retry(client: &LspClient, at: &FileLocation) -> Vec<Location> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.implementations(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(locations) if !locations.is_empty() => return locations,
            Ok(locations) if past_deadline => {
                panic!("implementations stayed empty until the deadline: {locations:?}")
            }
            Err(err) if past_deadline => {
                panic!("implementations never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn implementations_returns_the_single_implementor() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");
    let at = greeter_trait_site(&root);

    let locations = implementations_with_retry(&client, &at);

    assert_eq!(
        locations.len(),
        1,
        "expected exactly one implementor of Greeter, got {locations:?}"
    );
    assert_eq!(locations[0].range.start.line, 43);
}
