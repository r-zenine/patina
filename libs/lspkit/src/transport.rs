//! Layer 0: LSP wire framing — `Content-Length` header + JSON body over a byte
//! stream (the base protocol all LSP messages ride on, independent of any
//! particular request/response semantics). See `lib.rs` for the
//! initialize/initialized handshake and request-id dispatch built on top.

use crate::Result;
use std::io::{BufRead, Write};

// `#[allow(dead_code)]`: these two functions have no non-test caller yet —
// wiring them into `LspClient::start`'s handshake and `native`'s request
// methods is this phase's implementation contribution. Test-design lands the
// framing contract's compile surface and its own unit tests (runtime red via
// `unimplemented!()`) only; the allow comes off once the implementation
// contribution wires a real call path.

/// Frame `value` as `Content-Length: <n>\r\n\r\n<json>` and write it to `writer`.
#[allow(dead_code)]
pub(crate) fn write_message<W: Write>(writer: &mut W, value: &serde_json::Value) -> Result<()> {
    let _ = (writer, value);
    unimplemented!("encode Content-Length header + JSON body, write to the stream")
}

/// Read one framed JSON-RPC message from `reader`, blocking until a full
/// message (headers + body) has arrived.
#[allow(dead_code)]
pub(crate) fn read_message<R: BufRead>(reader: &mut R) -> Result<serde_json::Value> {
    let _ = reader;
    unimplemented!("parse Content-Length header(s), read exactly that many body bytes, parse JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn write_message_emits_content_length_header_and_body() {
        let value = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"});
        let mut buf = Vec::new();

        write_message(&mut buf, &value).expect("encoding should succeed");

        let body = serde_json::to_vec(&value).unwrap();
        let expected_header = format!("Content-Length: {}\r\n\r\n", body.len());
        assert!(buf.starts_with(expected_header.as_bytes()));
        assert_eq!(&buf[expected_header.len()..], body.as_slice());
    }

    #[test]
    fn read_message_parses_a_well_formed_message() {
        let value = serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": null});
        let body = serde_json::to_vec(&value).unwrap();
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}",
            body.len(),
            String::from_utf8(body).unwrap()
        );
        let mut cursor = Cursor::new(framed.into_bytes());

        let parsed = read_message(&mut cursor).expect("well-formed message should parse");

        assert_eq!(parsed, value);
    }

    #[test]
    fn read_message_ignores_additional_headers() {
        let value = serde_json::json!({"jsonrpc": "2.0", "method": "initialized"});
        let body = serde_json::to_vec(&value).unwrap();
        let framed = format!(
            "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            String::from_utf8(body).unwrap()
        );
        let mut cursor = Cursor::new(framed.into_bytes());

        let parsed = read_message(&mut cursor).expect("extra headers should be tolerated");

        assert_eq!(parsed, value);
    }

    #[test]
    fn read_message_errors_when_content_length_header_is_missing() {
        let mut cursor = Cursor::new(b"\r\n{}".to_vec());

        let result = read_message(&mut cursor);

        assert!(matches!(result, Err(crate::Error::Protocol(_))));
    }

    #[test]
    fn write_then_read_round_trips_the_same_value() {
        let value = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "textDocument/references",
            "params": {"foo": "bar", "count": 3}
        });
        let mut buf = Vec::new();
        write_message(&mut buf, &value).expect("encoding should succeed");

        let mut cursor = Cursor::new(buf);
        let parsed = read_message(&mut cursor).expect("decoding should succeed");

        assert_eq!(parsed, value);
    }
}
