//! LanguageDescriptor trait — data-driven language configuration for the generic semantic tree builder.
//!
//! Each language provides static kind tables and optional override methods.
//! `GenericSemanticTreeBuilder<D>` consumes any `D: LanguageDescriptor` to produce a
//! `SemanticTree` with the complete byte-coverage invariant.

use crate::common::{ProgrammingLanguage, SemanticNodeKind};
use crate::semantic_ast::{MetadataNode, MetadataPosition};
use tree_sitter::Node;

/// Language-specific configuration consumed by `GenericSemanticTreeBuilder`.
///
/// Implement this trait to add a new language. All methods with default
/// implementations cover the common case; override only what differs.
pub trait LanguageDescriptor: Send + Sync {
    // ── Identity ──────────────────────────────────────────────────────────

    /// Tree-sitter language handle for this language.
    fn ts_language(&self) -> tree_sitter::Language;

    /// Corresponding `ProgrammingLanguage` enum variant.
    fn programming_language(&self) -> ProgrammingLanguage;

    // ── Classification tables ─────────────────────────────────────────────

    /// Maps tree-sitter node kinds to `SemanticNodeKind`.
    ///
    /// Used by both `classify_node_kind` (full classification) and
    /// `GenericSemanticTreeBuilder` (semantic node creation).
    fn semantic_kind_map(&self) -> &[(&'static str, SemanticNodeKind)];

    /// Node kinds that are pure syntax tokens with no semantic value.
    ///
    /// Nodes matching these kinds are skipped during tree construction.
    /// They may still appear in `semantic_kind_map` for `classify_node_kind`.
    fn trivial_kinds(&self) -> &[&'static str];

    // ── Structural config ─────────────────────────────────────────────────

    /// For container node kinds (e.g. `"mod_item"`, `"impl_item"`), the
    /// tree-sitter field name whose child is the body to recurse into.
    ///
    /// Returns `None` for non-container nodes or nodes whose children are
    /// direct (e.g. `"source_file"`).
    fn container_body_field(&self, kind: &str) -> Option<&'static str>;

    /// Statement-wrapper node kinds whose children should be spliced directly into
    /// the enclosing container's children, rather than classified themselves.
    ///
    /// E.g. Python's `expression_statement` wraps `assignment` with no semantic
    /// value of its own; without this hook the classification-only `Statement`
    /// kind causes `build_typed_node` to drop the whole subtree, hiding
    /// module-level assignments from range lookups.
    ///
    /// Default: no wrapper kinds (most languages classify constructs directly).
    fn statement_wrapper_kinds(&self) -> &[&'static str] {
        &[]
    }

    /// The node kind that represents metadata / annotations
    /// (e.g. `"attribute_item"` in Rust, `"decorator"` in Python).
    ///
    /// Returns `None` if the language has no metadata annotation construct.
    fn metadata_kind(&self) -> Option<&'static str>;

    // ── Targeted overrides (with sensible defaults) ───────────────────────

    /// Extract the identifier/name for a node using grammar-specific field names.
    ///
    /// Default: try `child_by_field_name("name")` — covers function_item, struct_item,
    /// enum_item, const_item, static_item (Rust); function_declaration (Go);
    /// function_definition, class_definition (Python);
    /// function_declaration, interface_declaration (TypeScript).
    ///
    /// Override for nodes whose identifier lives under a different field or requires
    /// walking binding patterns (e.g. Rust let_declaration, Go short_var_declaration).
    #[expect(
        clippy::disallowed_methods,
        reason = "utf8_text on a tree-sitter node's own byte range is infallible"
    )]
    fn extract_identifier<'a>(&self, node: Node<'a>, source: &str) -> Option<String> {
        node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string())
    }

    /// Extract the visibility modifier text from a node.
    ///
    /// Default: look for a `"visibility_modifier"` child and return its text,
    /// falling back to `"private"`.
    fn extract_visibility<'a>(&self, node: Node<'a>, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier"
                && let Ok(text) = child.utf8_text(source.as_bytes())
            {
                return text.to_string();
            }
        }
        "private".to_string()
    }

    /// Collect metadata nodes (attributes, decorators) immediately preceding
    /// `node` among `parent`'s children.
    ///
    /// Default: scan backwards through `parent.children()` until a non-trivial,
    /// non-metadata sibling is found, collecting any nodes whose kind matches
    /// `metadata_kind()`.
    fn collect_metadata<'a>(&self, node: Node<'a>, parent: Node<'a>) -> Vec<MetadataNode<'a>> {
        let meta_kind = match self.metadata_kind() {
            Some(k) => k,
            None => return Vec::new(),
        };

        let mut cursor = parent.walk();
        let children: Vec<Node<'a>> = parent.children(&mut cursor).collect();

        let target_idx = match children.iter().position(|n| n.id() == node.id()) {
            Some(i) => i,
            None => return Vec::new(),
        };

        let mut metadata_nodes = Vec::new();
        let mut position = -1i32;

        for i in (0..target_idx).rev() {
            let sibling = children[i];
            if sibling.kind() == meta_kind {
                metadata_nodes.push(MetadataNode {
                    node: sibling,
                    position: MetadataPosition::PrecedingSibling(position),
                });
                position -= 1;
            } else if !self.trivial_kinds().contains(&sibling.kind()) {
                break;
            }
        }

        metadata_nodes.reverse();
        metadata_nodes
    }
}
