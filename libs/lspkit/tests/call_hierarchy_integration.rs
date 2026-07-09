//! Integration test against a real `rust-analyzer` process. Requires
//! `rust-analyzer` on `PATH`; marked `#[ignore]` since CI may not have it
//! installed. Run explicitly with:
//!
//! ```sh
//! cargo test -p lspkit --test call_hierarchy_integration -- --ignored
//! ```

use lspkit::{CallHierarchyItem, CallSite, FileLocation, LspClient, Position};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references_fixture")
}

/// Same rust-analyzer indexing race as `references_integration`'s retry
/// helper — the first call hierarchy request after a fresh `start()` can
/// come back empty or error out while indexing finishes.
fn prepare_call_hierarchy_with_retry(
    client: &LspClient,
    at: &FileLocation,
) -> Vec<CallHierarchyItem> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.prepare_call_hierarchy(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(items) if !items.is_empty() => return items,
            Ok(items) if past_deadline => {
                panic!("prepare_call_hierarchy stayed empty until the deadline: {items:?}")
            }
            Err(err) if past_deadline => {
                panic!("prepare_call_hierarchy never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn chain_b_item(client: &LspClient, root: &std::path::Path) -> CallHierarchyItem {
    let at = FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 17,
            character: 8,
        },
    };
    let mut items = prepare_call_hierarchy_with_retry(client, &at);
    assert_eq!(
        items.len(),
        1,
        "expected exactly one call hierarchy item for chain_b, got {items:?}"
    );
    items.remove(0)
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn prepare_call_hierarchy_resolves_the_symbol_at_the_position() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    let item = chain_b_item(&client, &root);

    assert_eq!(item.name, "chain_b");
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn incoming_calls_finds_the_known_caller() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");
    let item = chain_b_item(&client, &root);

    let callers: Vec<CallSite> = client
        .incoming_calls(&item)
        .expect("incoming_calls should succeed");

    assert_eq!(
        callers.len(),
        1,
        "expected exactly one caller of chain_b, got {callers:?}"
    );
    assert_eq!(callers[0].item.name, "chain_a");
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn outgoing_calls_finds_the_known_callee() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");
    let item = chain_b_item(&client, &root);

    let callees: Vec<CallSite> = client
        .outgoing_calls(&item)
        .expect("outgoing_calls should succeed");

    assert_eq!(
        callees.len(),
        1,
        "expected exactly one callee of chain_b, got {callees:?}"
    );
    assert_eq!(callees[0].item.name, "chain_c");
}
