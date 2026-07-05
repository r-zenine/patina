//! Generic semantic tree builder — consumes any `LanguageDescriptor` to produce a `SemanticTree`.
//!
//! ## Byte-coverage invariant
//! Every byte in the source belongs to exactly one `SemanticNode` in the returned tree.
//! Nodes whose tree-sitter kind is not recognised and not trivial become
//! `SemanticUnitType::Unknown` wrappers, ensuring no gaps.
//!
//! ## Impl-block handling
//! `impl_item` (Rust) and equivalent container constructs are represented as
//! `SemanticUnitType::Module { module_type: Namespace }`.  This makes the
//! `create_reviewable_diff_from_range` decompose path kick in for impl-level ranges,
//! correctly resolving to individual methods rather than the full file.

use crate::common::{ASTError, LanguageParser, Result, SemanticNodeKind};
use crate::parsers::descriptor::LanguageDescriptor;
use crate::semantic_ast::{
    ImportType, ModuleType, SemanticError, SemanticNode, SemanticTree, SemanticUnitType,
};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser, Tree};

/// Build a container-qualified name: `"parent::name"` if a container path is known,
/// otherwise just `name`. Used to compute `SemanticNode::qualified_name`.
fn qualify(parent_context: Option<&str>, name: &str) -> String {
    match parent_context {
        Some(ctx) => format!("{ctx}::{name}"),
        None => name.to_string(),
    }
}

/// Generic semantic tree builder parametrised over any [`LanguageDescriptor`].
///
/// Wire it up by wrapping it in a language-specific newtype that also provides
/// language-specific `get_context_boundaries` (or use the default).
pub struct GenericSemanticTreeBuilder<D: LanguageDescriptor> {
    pub(crate) descriptor: D,
    kind_map: HashMap<&'static str, SemanticNodeKind>,
    trivial_set: HashSet<&'static str>,
}

impl<D: LanguageDescriptor> GenericSemanticTreeBuilder<D> {
    pub fn new(descriptor: D) -> Self {
        let kind_map = descriptor
            .semantic_kind_map()
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        let trivial_set = descriptor.trivial_kinds().iter().cloned().collect();
        Self {
            descriptor,
            kind_map,
            trivial_set,
        }
    }

    // ── Entry point ───────────────────────────────────────────────────────

    /// Build the semantic tree rooted at the `source_file` node.
    pub(crate) fn build_root<'a>(
        &self,
        node: Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let children = self.build_container_children(node, source, None);
        let mut root = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Module {
                module_type: ModuleType::File,
                is_public: true,
                metadata: HashMap::new(),
            },
            Vec::new(),
        );
        root.children = children;
        Ok(root)
    }

    // ── Container child collection ────────────────────────────────────────

    /// Collect semantic children from `container`'s direct children list.
    ///
    /// - Trivial tokens are skipped.
    /// - Metadata nodes (e.g. `attribute_item`) become `Unknown` children so
    ///   their byte ranges are covered (fixes the struct-range-expansion bug).
    /// - Recognised kinds produce typed `SemanticNode`s.
    /// - Everything else becomes an `Unknown` wrapper if it has semantic children,
    ///   otherwise it is skipped.
    fn build_container_children<'a>(
        &self,
        container: Node<'a>,
        source: &str,
        parent_context: Option<&str>,
    ) -> Vec<SemanticNode<'a>> {
        let mut children = Vec::new();
        let mut cursor = container.walk();

        for child in container.children(&mut cursor) {
            if !child.is_named() {
                continue;
            }
            let kind = child.kind();

            if self.trivial_set.contains(kind) {
                continue;
            }

            // Metadata / annotation nodes → Unknown child for byte coverage.
            if let Some(meta_kind) = self.descriptor.metadata_kind()
                && kind == meta_kind
            {
                children.push(self.make_unknown(child));
                continue;
            }

            if let Some(node) = self.build_node(child, source, Some(container), parent_context) {
                children.push(node);
            }
        }

        children
    }

    // ── Per-node dispatch ─────────────────────────────────────────────────

    fn build_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> Option<SemanticNode<'a>> {
        if !node.is_named() {
            return None;
        }
        let kind = node.kind();

        if self.trivial_set.contains(kind) {
            return None;
        }

        match self.kind_map.get(kind) {
            Some(semantic_kind) => {
                self.build_typed_node(node, source, parent, parent_context, semantic_kind.clone())
            }
            None => {
                // Unrecognised: create Unknown only if it has semantic children.
                let children = self.build_container_children(node, source, parent_context);
                if children.is_empty() {
                    None
                } else {
                    let mut unknown = self.make_unknown(node);
                    unknown.children = children;
                    Some(unknown)
                }
            }
        }
    }

    fn build_typed_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
        semantic_kind: SemanticNodeKind,
    ) -> Option<SemanticNode<'a>> {
        match semantic_kind {
            SemanticNodeKind::SourceFile => {
                // Shouldn't be encountered as a child, but handle gracefully.
                self.build_root(node, source).ok()
            }

            SemanticNodeKind::Function => {
                Some(self.build_callable(node, source, parent, parent_context))
            }

            SemanticNodeKind::Struct
            | SemanticNodeKind::Enum
            | SemanticNodeKind::Interface
            | SemanticNodeKind::Class => {
                Some(self.build_data_structure(node, source, parent, parent_context))
            }

            SemanticNodeKind::ImplBlock => {
                Some(self.build_impl_container(node, source, parent_context))
            }

            SemanticNodeKind::Module => {
                Some(self.build_module_container(node, source, parent_context))
            }

            SemanticNodeKind::Import => self.build_import(node, source).ok(),

            SemanticNodeKind::Variable => Some(self.build_variable(node, source, parent_context)),

            // Statement, Expression, Comment, SignatureComponent, TypeDefinition, Other:
            // These are classification-only kinds; skip them in tree construction.
            _ => None,
        }
    }

    // ── Typed builders ────────────────────────────────────────────────────

    fn build_callable<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> SemanticNode<'a> {
        let name_node = node.child_by_field_name("name");
        let parameters_node = node.child_by_field_name("parameters");
        let return_type_node = node.child_by_field_name("return_type");
        let type_parameters = node.child_by_field_name("type_parameters");

        let parameter_count = parameters_node.map(|p| p.named_child_count()).unwrap_or(0);

        let return_type = return_type_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        let is_async = Self::node_has_keyword(node, "async");
        let visibility = self.descriptor.extract_visibility(node, source);

        let metadata_nodes = parent
            .map(|p| self.descriptor.collect_metadata(node, p))
            .unwrap_or_default();

        let identifier = self.descriptor.extract_identifier(node, source);
        let mut semantic_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic: type_parameters.is_some(),
                parameter_count,
                return_type,
                is_async,
                visibility,
                is_method: parent_context.is_some(),
                signature_node: parameters_node,
            },
            metadata_nodes,
        );
        semantic_node.qualified_name = identifier
            .as_deref()
            .map(|name| qualify(parent_context, name));
        semantic_node.identifier = identifier;
        semantic_node
    }

    fn build_data_structure<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> SemanticNode<'a> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let body = node.child_by_field_name("body");
        let visibility = self.descriptor.extract_visibility(node, source);

        let metadata_nodes = parent
            .map(|p| self.descriptor.collect_metadata(node, p))
            .unwrap_or_default();

        let field_count = body.map(|b| {
            let mut cur = b.walk();
            b.children(&mut cur)
                .filter(|c| c.kind() == "field_declaration" || c.kind() == "enum_variant")
                .count()
        });

        let identifier = self.descriptor.extract_identifier(node, source);
        let mut semantic_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: type_parameters.is_some(),
                field_count,
                visibility,
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        );
        semantic_node.qualified_name = identifier
            .as_deref()
            .map(|name| qualify(parent_context, name));
        semantic_node.identifier = identifier;
        semantic_node
    }

    /// Build an impl block as a `Module { Namespace }` container.
    ///
    /// Representing impl blocks as Module triggers the decompose path in
    /// `create_reviewable_diff_from_range`, so byte ranges covering the impl
    /// header correctly resolve to the enclosed methods rather than the full file.
    fn build_impl_container<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent_context: Option<&str>,
    ) -> SemanticNode<'a> {
        let target_type = node
            .child_by_field_name("type")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("Unknown");
        let own_path = qualify(parent_context, target_type);

        let children = node
            .child_by_field_name("body")
            .map(|body| self.build_container_children(body, source, Some(&own_path)))
            .unwrap_or_default();

        let mut module_node = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Module {
                module_type: ModuleType::Namespace,
                is_public: false,
                metadata: HashMap::new(),
            },
            Vec::new(),
        );
        module_node.children = children;
        module_node
    }

    fn build_module_container<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent_context: Option<&str>,
    ) -> SemanticNode<'a> {
        let name_node = node.child_by_field_name("name");
        let visibility = self.descriptor.extract_visibility(node, source);
        let module_name = name_node.and_then(|n| n.utf8_text(source.as_bytes()).ok());
        let own_path = module_name.map(|name| qualify(parent_context, name));

        let children = node
            .child_by_field_name("body")
            .map(|body| self.build_container_children(body, source, own_path.as_deref()))
            .unwrap_or_default();

        let mut module_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Module {
                module_type: ModuleType::Module,
                is_public: visibility.starts_with("pub"),
                metadata: HashMap::new(),
            },
            Vec::new(),
        );
        module_node.children = children;
        module_node
    }

    fn build_import<'a>(
        &self,
        node: Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let (import_type, source_module, imported_items) =
            self.parse_use_declaration(node, source)?;

        Ok(SemanticNode::new(
            node,
            None,
            SemanticUnitType::Import {
                import_type,
                source_module,
                imported_items,
                metadata: HashMap::new(),
            },
            Vec::new(),
        ))
    }

    fn build_variable<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent_context: Option<&str>,
    ) -> SemanticNode<'a> {
        let name_node = node.child_by_field_name("name");
        let type_node = node.child_by_field_name("type");
        let visibility = self.descriptor.extract_visibility(node, source);

        let type_annotation = type_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        let identifier = self.descriptor.extract_identifier(node, source);
        let mut semantic_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Variable {
                is_const: node.kind() == "const_item",
                is_static: node.kind() == "static_item",
                type_annotation,
                visibility,
                metadata: HashMap::new(),
            },
            Vec::new(),
        );
        semantic_node.qualified_name = identifier
            .as_deref()
            .map(|name| qualify(parent_context, name));
        semantic_node.identifier = identifier;
        semantic_node
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn make_unknown<'a>(&self, node: Node<'a>) -> SemanticNode<'a> {
        let mut metadata = HashMap::new();
        metadata.insert("original_kind".to_string(), node.kind().to_string());
        SemanticNode::new(
            node,
            None,
            SemanticUnitType::Unknown {
                node_kind: node.kind().to_string(),
                metadata,
            },
            Vec::new(),
        )
    }

    fn node_has_keyword(node: Node<'_>, keyword: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == keyword {
                return true;
            }
            if child.kind() == "function_modifiers" && Self::node_has_keyword(child, keyword) {
                return true;
            }
        }
        false
    }

    fn parse_use_declaration(
        &self,
        node: Node<'_>,
        source: &str,
    ) -> std::result::Result<(ImportType, String, Vec<String>), SemanticError> {
        let use_text = node
            .utf8_text(source.as_bytes())
            .map_err(|_| SemanticError::TreeBuildError("Failed to extract use text".into()))?;

        let use_text = use_text
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .trim();

        if use_text.contains('*') {
            Ok((
                ImportType::Wildcard,
                use_text.replace("::*", ""),
                Vec::new(),
            ))
        } else if use_text.contains('{') {
            let parts: Vec<&str> = use_text.split("::").collect();
            let module = parts[..parts.len() - 1].join("::");
            let items_part = parts.last().unwrap_or(&"");
            let items = items_part
                .trim_matches(|c| c == '{' || c == '}')
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok((ImportType::Specific, module, items))
        } else {
            Ok((ImportType::Default, use_text.to_string(), Vec::new()))
        }
    }
}

// ── LanguageParser impl ───────────────────────────────────────────────────────

impl<D: LanguageDescriptor> LanguageParser for GenericSemanticTreeBuilder<D> {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&self.descriptor.ts_language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set language: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Parsing returned None".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        self.descriptor.ts_language()
    }

    fn build_semantic_tree<'a>(
        &self,
        ast: &'a Tree,
        source: &str,
    ) -> std::result::Result<SemanticTree<'a>, SemanticError> {
        let semantic_root = self.build_root(ast.root_node(), source)?;
        Ok(SemanticTree::new(
            semantic_root,
            self.descriptor.programming_language(),
        ))
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        self.kind_map
            .get(node_kind)
            .cloned()
            .unwrap_or_else(|| SemanticNodeKind::Other(node_kind.to_string()))
    }

    fn extract_identifier(&self, node: Node, source: &str) -> Option<String> {
        self.descriptor.extract_identifier(node, source)
    }
}

// ── Invariant assertion ───────────────────────────────────────────────────────

/// Assert that every child's byte range is contained within its parent's range.
/// Panics with a descriptive message if the invariant is violated.
#[allow(dead_code)]
pub fn assert_byte_coverage_invariant(node: &SemanticNode<'_>, source_len: usize) {
    let parent_range = node.tree_sitter_node.byte_range();
    assert!(
        parent_range.end <= source_len,
        "Node '{}' end_byte {} exceeds source length {}",
        node.tree_sitter_node.kind(),
        parent_range.end,
        source_len
    );
    for child in &node.children {
        let child_range = child.tree_sitter_node.byte_range();
        assert!(
            child_range.start >= parent_range.start && child_range.end <= parent_range.end,
            "Child '{}' range [{}, {}) is outside parent '{}' range [{}, {})",
            child.tree_sitter_node.kind(),
            child_range.start,
            child_range.end,
            node.tree_sitter_node.kind(),
            parent_range.start,
            parent_range.end,
        );
        assert_byte_coverage_invariant(child, source_len);
    }
}
