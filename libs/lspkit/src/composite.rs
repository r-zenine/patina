//! Layer 2: composite queries built by orchestrating several `native` calls
//! and reshaping the results into one domain-shaped answer. Each of these
//! issues more than one LSP request, so they're strictly more expensive than
//! a `native` call.

use crate::{BlastRadius, DocumentSymbol, FileLocation, Location, LspClient, PeekResult, Result};

impl LspClient {
    /// `definition` + `hover` at the resolved location, in one call.
    ///
    /// When `definition` resolves to more than one location (macro expansions,
    /// partial impls), only the first is peeked — this query answers "what is
    /// this", which a single representative location already does.
    pub fn peek_definition(&self, at: &FileLocation) -> Result<Option<PeekResult>> {
        let Some(location) = self.definition(at)?.into_iter().next() else {
            return Ok(None);
        };
        let hover_at = FileLocation {
            path: location.path.clone(),
            position: location.range.start,
        };
        let hover = self.hover(&hover_at)?;
        Ok(Some(PeekResult {
            location,
            signature: hover
                .as_ref()
                .map_or_else(String::new, |h| h.signature.clone()),
            docs: hover.and_then(|h| h.docs),
        }))
    }

    /// `prepare_call_hierarchy` + `incoming_calls` + `outgoing_calls` + `references`,
    /// merged into one impact report.
    ///
    /// When `at` resolves to more than one call hierarchy item, only the first
    /// is used — same rationale as `peek_definition`.
    pub fn blast_radius(&self, at: &FileLocation) -> Result<BlastRadius> {
        let Some(item) = self.prepare_call_hierarchy(at)?.into_iter().next() else {
            return Ok(BlastRadius {
                callers: Vec::new(),
                callees: Vec::new(),
                other_references: Vec::new(),
                total_impact: 0,
            });
        };
        let callers = self.incoming_calls(&item)?;
        let callees = self.outgoing_calls(&item)?;
        // `references` includes the call sites already captured by the call
        // hierarchy plus non-call references (struct literals, imports, doc
        // links) — excluding the declaration itself, since call/callee sites
        // already speak to usage and the declaration isn't an impact.
        let call_site_paths: std::collections::HashSet<_> = callers
            .iter()
            .chain(callees.iter())
            .map(|site| site.item.location.path.clone())
            .collect();
        let references_at = FileLocation {
            path: item.location.path.clone(),
            position: item.location.range.start,
        };
        let other_references: Vec<Location> = self
            .references(&references_at, false)?
            .into_iter()
            .filter(|reference| !call_site_paths.contains(&reference.path))
            .collect();
        let total_impact = callers.len() + callees.len() + other_references.len();
        Ok(BlastRadius {
            callers,
            callees,
            other_references,
            total_impact,
        })
    }

    /// All other methods/associated items sharing the same enclosing impl/type as `at`.
    ///
    /// Local case only: finds the enclosing `impl`/`trait` block in
    /// `document_symbols` for `at`'s own file and returns its other children.
    /// Cross-file impls of the same type (via `workspace_symbols`/
    /// `implementations`) are not merged in — rust-analyzer's `workspace/symbol`
    /// takes a fuzzy text query, not a type handle, so reliably resolving "other
    /// impl blocks of this exact type" needs the type name threaded through from
    /// the caller; deferred until a caller actually needs cross-file siblings.
    pub fn sibling_methods(&self, at: &FileLocation) -> Result<Vec<DocumentSymbol>> {
        let symbols = self.document_symbols(&at.path)?;
        let Some(enclosing) = find_enclosing_container(&symbols, at.position) else {
            return Ok(Vec::new());
        };
        Ok(enclosing
            .children
            .iter()
            .filter(|child| !contains(&child.range, at.position))
            .cloned()
            .collect())
    }
}

/// Depth-first search for the innermost symbol whose `range` contains
/// `position` and that itself has children (an `impl`/`trait` block, not a
/// leaf method) — that symbol's `children` are the siblings.
fn find_enclosing_container(
    symbols: &[DocumentSymbol],
    position: crate::Position,
) -> Option<&DocumentSymbol> {
    for symbol in symbols {
        if !contains(&symbol.range, position) {
            continue;
        }
        if let Some(nested) = find_enclosing_container(&symbol.children, position) {
            return Some(nested);
        }
        if !symbol.children.is_empty() {
            return Some(symbol);
        }
    }
    None
}

fn contains(range: &crate::Range, position: crate::Position) -> bool {
    (range.start.line, range.start.character) <= (position.line, position.character)
        && (position.line, position.character) <= (range.end.line, range.end.character)
}
