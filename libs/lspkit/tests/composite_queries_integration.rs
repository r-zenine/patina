//! Integration test against a real `rust-analyzer` process. Requires
//! `rust-analyzer` on `PATH`; marked `#[ignore]` since CI may not have it
//! installed. Run explicitly with:
//!
//! ```sh
//! cargo test -p lspkit --test composite_queries_integration -- --ignored
//! ```

use lspkit::{DocumentSymbol, FileLocation, LspClient, PeekResult, Position};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references_fixture")
}

/// Same rust-analyzer indexing race every other integration test in this
/// crate rides out — the first request after a fresh `start()` can come back
/// empty while indexing finishes.
fn peek_definition_with_retry(client: &LspClient, at: &FileLocation) -> PeekResult {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.peek_definition(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(Some(result)) => return result,
            Ok(None) if past_deadline => panic!("peek_definition stayed empty until the deadline"),
            Err(err) if past_deadline => {
                panic!("peek_definition never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn blast_radius_with_retry(client: &LspClient, at: &FileLocation) -> lspkit::BlastRadius {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.blast_radius(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(radius) if radius.total_impact > 0 => return radius,
            Ok(radius) if past_deadline => {
                panic!("blast_radius stayed empty until the deadline: {radius:?}")
            }
            Err(err) if past_deadline => {
                panic!("blast_radius never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn sibling_methods_with_retry(client: &LspClient, at: &FileLocation) -> Vec<DocumentSymbol> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = client.sibling_methods(at);
        let past_deadline = Instant::now() >= deadline;
        match outcome {
            Ok(symbols) if !symbols.is_empty() => return symbols,
            Ok(symbols) if past_deadline => {
                panic!("sibling_methods stayed empty until the deadline: {symbols:?}")
            }
            Err(err) if past_deadline => {
                panic!("sibling_methods never resolved before the deadline: {err}")
            }
            _ => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn peek_definition_resolves_location_and_signature_in_one_call() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    // Same `Shape` usage site as definition_hover_integration.
    let at = FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 30,
            character: 26,
        },
    };

    let peek = peek_definition_with_retry(&client, &at);

    assert_eq!(peek.location.range.start.line, 25);
    assert!(
        peek.signature.contains("Shape"),
        "expected peeked signature to mention Shape, got {:?}",
        peek.signature
    );
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn blast_radius_reports_the_known_caller_and_callee_of_chain_b() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    // Same `chain_b` site as call_hierarchy_integration.
    let at = FileLocation {
        path: root.join("src/lib.rs"),
        position: Position {
            line: 17,
            character: 8,
        },
    };

    let radius = blast_radius_with_retry(&client, &at);

    assert_eq!(
        radius.callers.len(),
        1,
        "expected exactly one caller of chain_b, got {:?}",
        radius.callers
    );
    assert_eq!(radius.callers[0].item.name, "chain_a");
    assert_eq!(
        radius.callees.len(),
        1,
        "expected exactly one callee of chain_b, got {:?}",
        radius.callees
    );
    assert_eq!(radius.callees[0].item.name, "chain_c");
}

#[test]
#[ignore = "requires rust-analyzer on PATH; run with `cargo test -- --ignored`"]
fn sibling_methods_finds_the_other_method_in_the_same_impl_block() {
    let root = fixture_root();
    let client = LspClient::start(&root).expect("initialize handshake should succeed");

    // Inside `Item::describe` in src/types.rs; `Item::kind` is the only
    // other method in the same `impl Item` block.
    let at = FileLocation {
        path: root.join("src/types.rs"),
        position: Position {
            line: 6,
            character: 15,
        },
    };

    let siblings = sibling_methods_with_retry(&client, &at);

    assert_eq!(
        siblings.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        vec!["kind"],
        "expected describe's only sibling to be kind, got {siblings:?}"
    );
}
