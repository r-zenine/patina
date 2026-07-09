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

    pub fn prepare_call_hierarchy(&self, at: &FileLocation) -> Result<Vec<CallHierarchyItem>> {
        let params = serde_json::json!({
            "textDocument": {"uri": to_file_uri(&at.path)},
            "position": to_lsp_position(at.position),
        });
        let result = self.call("textDocument/prepareCallHierarchy", params)?;
        match result {
            serde_json::Value::Null => Ok(Vec::new()),
            serde_json::Value::Array(items) => {
                items.iter().map(from_lsp_call_hierarchy_item).collect()
            }
            other => Err(Error::Protocol(format!(
                "textDocument/prepareCallHierarchy: expected an array or null result, got {other}"
            ))),
        }
    }

    pub fn incoming_calls(&self, item: &CallHierarchyItem) -> Result<Vec<CallSite>> {
        let params = serde_json::json!({"item": to_lsp_call_hierarchy_item(item)});
        let result = self.call("callHierarchy/incomingCalls", params)?;
        match result {
            serde_json::Value::Null => Ok(Vec::new()),
            serde_json::Value::Array(items) => items
                .iter()
                .map(|entry| from_lsp_call_site(entry, "from"))
                .collect(),
            other => Err(Error::Protocol(format!(
                "callHierarchy/incomingCalls: expected an array or null result, got {other}"
            ))),
        }
    }

    pub fn outgoing_calls(&self, item: &CallHierarchyItem) -> Result<Vec<CallSite>> {
        let params = serde_json::json!({"item": to_lsp_call_hierarchy_item(item)});
        let result = self.call("callHierarchy/outgoingCalls", params)?;
        match result {
            serde_json::Value::Null => Ok(Vec::new()),
            serde_json::Value::Array(items) => items
                .iter()
                .map(|entry| from_lsp_call_site(entry, "to"))
                .collect(),
            other => Err(Error::Protocol(format!(
                "callHierarchy/outgoingCalls: expected an array or null result, got {other}"
            ))),
        }
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

fn to_lsp_range(range: Range) -> serde_json::Value {
    serde_json::json!({
        "start": to_lsp_position(range.start),
        "end": to_lsp_position(range.end),
    })
}

/// LSP `SymbolKind` is a 1-based numeric enum; only the kinds this crate's
/// `SymbolKind` distinguishes are mapped, everything else collapses to `Other`.
fn from_lsp_symbol_kind(value: u64) -> crate::SymbolKind {
    use crate::SymbolKind;
    match value {
        2 => SymbolKind::Module,
        6 => SymbolKind::Method,
        8 => SymbolKind::Field,
        10 => SymbolKind::Enum,
        11 => SymbolKind::Interface,
        12 => SymbolKind::Function,
        19 => SymbolKind::Object,
        22 => SymbolKind::EnumMember,
        23 => SymbolKind::Struct,
        26 => SymbolKind::TypeParameter,
        _ => SymbolKind::Other,
    }
}

/// Inverse of `from_lsp_symbol_kind`, used when re-sending a `CallHierarchyItem`
/// this crate already parsed back to the server (`incoming_calls`/
/// `outgoing_calls` take the item, not just a position). `Other` has no single
/// correct LSP kind to round-trip to; `Function` is the safest default since
/// call hierarchy items are overwhelmingly functions/methods.
fn to_lsp_symbol_kind(kind: crate::SymbolKind) -> u64 {
    use crate::SymbolKind;
    match kind {
        SymbolKind::Module => 2,
        SymbolKind::Method => 6,
        SymbolKind::Field => 8,
        SymbolKind::Enum => 10,
        SymbolKind::Interface => 11,
        SymbolKind::Function => 12,
        SymbolKind::Object => 19,
        SymbolKind::EnumMember => 22,
        SymbolKind::Struct => 23,
        SymbolKind::TypeParameter => 26,
        SymbolKind::Other => 12,
    }
}

/// `CallHierarchyItem.range` spans the whole declaration (e.g. including
/// doc comments); `selectionRange` is just the name. This crate's
/// `CallHierarchyItem` keeps only one `Location`, so `selectionRange` is
/// used for both directions — it's the more useful of the two for display,
/// and it's still a valid position inside `range` when reconstructed for
/// `incoming_calls`/`outgoing_calls`.
fn from_lsp_call_hierarchy_item(value: &serde_json::Value) -> Result<CallHierarchyItem> {
    let name = value
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            Error::Protocol(format!("expected a call hierarchy item.name, got {value}"))
        })?
        .to_string();
    let kind = value
        .get("kind")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            Error::Protocol(format!("expected a call hierarchy item.kind, got {value}"))
        })?;
    let uri = value
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            Error::Protocol(format!("expected a call hierarchy item.uri, got {value}"))
        })?;
    let selection_range = value.get("selectionRange").ok_or_else(|| {
        Error::Protocol(format!(
            "expected a call hierarchy item.selectionRange, got {value}"
        ))
    })?;
    Ok(CallHierarchyItem {
        name,
        kind: from_lsp_symbol_kind(kind),
        location: Location {
            path: from_file_uri(uri)?,
            range: from_lsp_range(selection_range)?,
        },
    })
}

fn to_lsp_call_hierarchy_item(item: &CallHierarchyItem) -> serde_json::Value {
    serde_json::json!({
        "name": item.name,
        "kind": to_lsp_symbol_kind(item.kind),
        "uri": to_file_uri(&item.location.path),
        "range": to_lsp_range(item.location.range),
        "selectionRange": to_lsp_range(item.location.range),
    })
}

/// Parses a `CallHierarchyIncomingCall`/`CallHierarchyOutgoingCall` entry —
/// identical shape except the item field is named `from`/`to` respectively.
fn from_lsp_call_site(value: &serde_json::Value, item_field: &str) -> Result<CallSite> {
    let item = value.get(item_field).ok_or_else(|| {
        Error::Protocol(format!("expected a call site.{item_field}, got {value}"))
    })?;
    let ranges = value
        .get("fromRanges")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::Protocol(format!("expected a call site.fromRanges, got {value}")))?;
    Ok(CallSite {
        item: from_lsp_call_hierarchy_item(item)?,
        call_ranges: ranges.iter().map(from_lsp_range).collect::<Result<_>>()?,
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
