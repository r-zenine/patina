//! Layer 1: thin, ~1:1 wrappers over LSP protocol requests. No cross-request
//! orchestration lives here — see `composite` for that.

use crate::{
    CallHierarchyItem, CallSite, DocumentSymbol, FileLocation, Hover, Location, LspClient, Result,
};
use std::path::Path;

impl LspClient {
    pub fn hover(&self, _at: &FileLocation) -> Result<Option<Hover>> {
        todo!("wire textDocument/hover over the JSON-RPC transport")
    }

    pub fn definition(&self, _at: &FileLocation) -> Result<Vec<Location>> {
        todo!("wire textDocument/definition over the JSON-RPC transport")
    }

    pub fn references(
        &self,
        _at: &FileLocation,
        _include_declaration: bool,
    ) -> Result<Vec<Location>> {
        todo!("wire textDocument/references over the JSON-RPC transport")
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
