//! Layer 2: composite queries built by orchestrating several `native` calls
//! and reshaping the results into one domain-shaped answer. Each of these
//! issues more than one LSP request, so they're strictly more expensive than
//! a `native` call.

use crate::{BlastRadius, DocumentSymbol, FileLocation, LspClient, PeekResult, Result};

impl LspClient {
    /// `definition` + `hover` at the resolved location, in one call.
    pub fn peek_definition(&self, _at: &FileLocation) -> Result<Option<PeekResult>> {
        todo!("definition() then hover() at the resolved location")
    }

    /// `prepare_call_hierarchy` + `incoming_calls` + `outgoing_calls` + `references`,
    /// merged into one impact report.
    pub fn blast_radius(&self, _at: &FileLocation) -> Result<BlastRadius> {
        todo!("prepare_call_hierarchy() then incoming_calls()/outgoing_calls()/references()")
    }

    /// All other methods/associated items sharing the same enclosing impl/type as `at`.
    ///
    /// Local case: find the enclosing `impl` in `document_symbols` for the same file.
    /// Full case: also `workspace_symbols`/`implementations` for other `impl` blocks
    /// of the same type across files, merging their children in.
    pub fn sibling_methods(&self, _at: &FileLocation) -> Result<Vec<DocumentSymbol>> {
        todo!("document_symbols() on the enclosing file, locate containing impl, return siblings")
    }
}
