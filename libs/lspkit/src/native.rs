//! Layer 1: thin, ~1:1 wrappers over LSP protocol requests. No cross-request
//! orchestration lives here — see `composite` for that.

use crate::{
    CallHierarchyItem, CallSite, DocumentSymbol, Error, FileLocation, Hover, Location, LspClient,
    Position, Range, Result,
};
use std::path::{Path, PathBuf};

impl LspClient {
    pub fn hover(&self, _at: &FileLocation) -> Result<Option<Hover>> {
        todo!("wire textDocument/hover over the JSON-RPC transport")
    }

    pub fn definition(&self, _at: &FileLocation) -> Result<Vec<Location>> {
        todo!("wire textDocument/definition over the JSON-RPC transport")
    }

    pub fn references(
        &self,
        at: &FileLocation,
        include_declaration: bool,
    ) -> Result<Vec<Location>> {
        let params = serde_json::json!({
            "textDocument": {"uri": to_file_uri(&at.path)},
            "position": to_lsp_position(at.position),
            "context": {"includeDeclaration": include_declaration},
        });
        let result = self.call("textDocument/references", params)?;
        match result {
            serde_json::Value::Null => Ok(Vec::new()),
            serde_json::Value::Array(items) => items.iter().map(from_lsp_location).collect(),
            other => Err(Error::Protocol(format!(
                "textDocument/references: expected an array or null result, got {other}"
            ))),
        }
    }

    pub fn implementations(&self, _at: &FileLocation) -> Result<Vec<Location>> {
        todo!("wire textDocument/implementation over the JSON-RPC transport")
    }

    pub fn document_symbols(&self, _path: &Path) -> Result<Vec<DocumentSymbol>> {
        todo!("wire textDocument/documentSymbol over the JSON-RPC transport")
    }

    pub fn workspace_symbols(&self, _query: &str) -> Result<Vec<DocumentSymbol>> {
        todo!("wire workspace/symbol over the JSON-RPC transport")
    }

    pub fn prepare_call_hierarchy(&self, _at: &FileLocation) -> Result<Vec<CallHierarchyItem>> {
        todo!("wire textDocument/prepareCallHierarchy over the JSON-RPC transport")
    }

    pub fn incoming_calls(&self, _item: &CallHierarchyItem) -> Result<Vec<CallSite>> {
        todo!("wire callHierarchy/incomingCalls over the JSON-RPC transport")
    }

    pub fn outgoing_calls(&self, _item: &CallHierarchyItem) -> Result<Vec<CallSite>> {
        todo!("wire callHierarchy/outgoingCalls over the JSON-RPC transport")
    }
}

// ---- LSP <-> this crate's type conversions ----
//
// LSP positions are 0-based; this crate's `Position` is 1-based (editor
// convention, see lib.rs). Convert at this boundary only.

fn to_lsp_position(position: Position) -> serde_json::Value {
    serde_json::json!({
        "line": position.line.saturating_sub(1),
        "character": position.character.saturating_sub(1),
    })
}

fn from_lsp_position(value: &serde_json::Value) -> Result<Position> {
    let line = value
        .get("line")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| Error::Protocol(format!("expected a position.line, got {value}")))?;
    let character = value
        .get("character")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| Error::Protocol(format!("expected a position.character, got {value}")))?;
    Ok(Position {
        line: line as u32 + 1,
        character: character as u32 + 1,
    })
}

fn from_lsp_range(value: &serde_json::Value) -> Result<Range> {
    let start = value
        .get("start")
        .ok_or_else(|| Error::Protocol(format!("expected a range.start, got {value}")))?;
    let end = value
        .get("end")
        .ok_or_else(|| Error::Protocol(format!("expected a range.end, got {value}")))?;
    Ok(Range {
        start: from_lsp_position(start)?,
        end: from_lsp_position(end)?,
    })
}

fn from_lsp_location(value: &serde_json::Value) -> Result<Location> {
    let uri = value
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::Protocol(format!("expected a location.uri, got {value}")))?;
    let range = value
        .get("range")
        .ok_or_else(|| Error::Protocol(format!("expected a location.range, got {value}")))?;
    Ok(Location {
        path: from_file_uri(uri)?,
        range: from_lsp_range(range)?,
    })
}

fn to_file_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn from_file_uri(uri: &str) -> Result<PathBuf> {
    uri.strip_prefix("file://")
        .map(PathBuf::from)
        .ok_or_else(|| Error::Protocol(format!("expected a file:// URI, got {uri}")))
}
