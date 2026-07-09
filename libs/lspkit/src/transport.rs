//! Layer 0: LSP wire framing — `Content-Length` header + JSON body over a byte
//! stream (the base protocol all LSP messages ride on, independent of any
//! particular request/response semantics). See `lib.rs` for the
//! initialize/initialized handshake and request-id dispatch built on top.

use crate::{Error, Result};
use std::io::{BufRead, Write};

/// Frame `value` as `Content-Length: <n>\r\n\r\n<json>` and write it to `writer`.
pub(crate) fn write_message<W: Write>(writer: &mut W, value: &serde_json::Value) -> Result<()> {
    let body = serde_json::to_vec(value).map_err(|e| Error::Protocol(e.to_string()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())
        .map_err(|e| Error::Protocol(e.to_string()))?;
    writer
        .write_all(&body)
        .map_err(|e| Error::Protocol(e.to_string()))?;
    writer.flush().map_err(|e| Error::Protocol(e.to_string()))
}

/// Read one framed JSON-RPC message from `reader`, blocking until a full
/// message (headers + body) has arrived.
pub(crate) fn read_message<R: BufRead>(reader: &mut R) -> Result<serde_json::Value> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| Error::Protocol(e.to_string()))?;
        if bytes_read == 0 {
            return Err(Error::ServerExited);
        }
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some(value) = header
            .strip_prefix("Content-Length:")
            .or_else(|| header.strip_prefix("Content-Length :"))
        {
            let value = value.trim();
            content_length = Some(value.parse::<usize>().map_err(|_| {
                Error::Protocol(format!("invalid Content-Length header: {value:?}"))
            })?);
        }
        // Other headers (e.g. Content-Type) are part of the base protocol but
        // carry no information this crate needs — ignored, not rejected.
    }

    let content_length =
        content_length.ok_or_else(|| Error::Protocol("missing Content-Length header".into()))?;
    let mut body = vec![0u8; content_length];
    reader
        .read_exact(&mut body)
        .map_err(|e| Error::Protocol(e.to_string()))?;
    serde_json::from_slice(&body).map_err(|e| Error::Protocol(e.to_string()))
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
