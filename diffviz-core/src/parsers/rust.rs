//! Rust language parser implementation
//!
//! This module provides Rust-specific parsing using TreeSitter and implements
//! the simplified LanguageParser trait for AST analysis.

use crate::common::{ASTError, LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::semantic_ast::{
    ImportType, MetadataNode, MetadataPosition, ModuleType, SemanticError, SemanticNode,
    SemanticSimilarity, SemanticTree, SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Tree};

/// Rust language parser using TreeSitter
pub struct RustParser;

impl RustParser {
    /// Create a new RustParser
    pub fn new() -> Self {
        Self
    }

    /// Build a semantic node from a TreeSitter node
    /// Only creates semantic nodes for meaningful constructs, skips trivial syntax tokens
    fn build_semantic_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "source_file" => self.build_source_file_node(node, source),
            "function_item" => self.build_function_node(node, source, parent, parent_context),
            "struct_item" => self.build_struct_node(node, source, parent),
            "enum_item" => self.build_enum_node(node, source, parent),
            "mod_item" => self.build_module_node(node, source, parent),
            "use_declaration" => self.build_import_node(node, source, parent),
            "const_item" | "static_item" => self.build_variable_node(node, source, parent),
            "impl_item" => {
                // Skip impl, return error since this should be handled at source file level
                Err(SemanticError::TreeBuildError(
                    "Impl items should be handled at parent level".to_string(),
                ))
            }
            _ => {
                // Skip trivial syntax tokens - only process meaningful semantic constructs
                if self.is_trivial_syntax_token(node.kind()) {
                    return Err(SemanticError::TreeBuildError(format!(
                        "Skipping trivial syntax token: {}",
                        node.kind()
                    )));
                }

                // For non-trivial unknown node types, look for meaningful children
                let mut meaningful_children = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    // Only try to build semantic nodes for potentially meaningful children
                    if !self.is_trivial_syntax_token(child.kind()) {
                        match self.build_semantic_node(child, source, Some(node), parent_context) {
                            Ok(child_node) => meaningful_children.push(child_node),
                            Err(_) => {
                                // Skip children that failed to parse - no need to create Unknown nodes
                                continue;
                            }
                        }
                    }
                }

                // If we found meaningful children, create a container node
                if !meaningful_children.is_empty() {
                    let mut metadata = HashMap::new();
                    metadata.insert("original_kind".to_string(), node.kind().to_string());

                    let mut container_node = SemanticNode::new(
                        node,
                        None,
                        SemanticUnitType::Unknown {
                            node_kind: node.kind().to_string(),
                            metadata,
                        },
                        Vec::new(),
                    );
                    container_node.children = meaningful_children;

                    Ok(container_node)
                } else {
                    // No meaningful children found, skip this node
                    Err(SemanticError::TreeBuildError(format!(
                        "No meaningful children found for node: {}",
                        node.kind()
                    )))
                }
            }
        }
    }

    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ";" | ":" | "::" |
            "." | ".." | "..." | "?" | "!" | "#" | "@" | "$" | "%" |
            "^" | "&" | "*" | "-" | "=" | "+" | "|" | "\\" | "/" | "<" | ">" |

            // Keywords that are part of larger constructs
            "fn" | "struct" | "enum" | "impl" | "trait" | "mod" | "use" |
            "pub" | "const" | "static" | "let" | "mut" | "if" | "else" |
            "match" | "for" | "while" | "loop" | "break" | "continue" |
            "return" | "async" | "await" | "move" | "where" | "as" | "in" |

            // Derives (these should be part of their parent construct, but not the attribute container)
            "derive" | "Debug" | "Clone" | "Copy" | "PartialEq" |
            "Eq" | "Hash" | "Default" | "Serialize" | "Deserialize" |

            // Visibility and modifiers
            "visibility_modifier" | "function_modifiers" |

            // Literals and identifiers (these should be part of their parent construct)
            "string_literal" | "raw_string_literal" | "integer_literal" |
            "float_literal" | "boolean_literal" | "char_literal" |
            "identifier" | "field_identifier" | "type_identifier" |

            // Type-related tokens
            "primitive_type" | "generic_type" | "reference_type" |
            "pointer_type" | "array_type" | "tuple_type" |

            // Expression components (should be part of larger expressions)
            "binary_operator" | "unary_operator" | "assignment_operator" |
            "compound_assignment_expr" | "range_expression" |

            // Comments and whitespace
            "line_comment" | "block_comment" | "doc_comment" |

            // Error nodes
            "ERROR" | "MISSING"
        )
    }

    /// Extract attribute information from an attribute_item node
    #[allow(dead_code)]
    fn extract_attribute_info(&self, node: Node<'_>, source: &str) -> String {
        // Find the attribute node inside the attribute_item
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "attribute" {
                return child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            }
        }
        // Fallback: return the full text
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    /// Build semantic node with attributes, delegating to the existing build_semantic_node
    #[allow(dead_code)]
    fn build_semantic_node_with_attributes<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent_context: Option<&str>,
        attributes: &[String],
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let mut semantic_node = self.build_semantic_node(node, source, None, parent_context)?;

        // Add attributes to the node's metadata if there are any
        if !attributes.is_empty() {
            match &mut semantic_node.unit_type {
                SemanticUnitType::Callable { metadata, .. }
                | SemanticUnitType::DataStructure { metadata, .. }
                | SemanticUnitType::Variable { metadata, .. } => {
                    metadata.insert("attributes".to_string(), attributes.join(", "));
                }
                _ => {
                    // For other types, we could extend this as needed
                }
            }
        }

        Ok(semantic_node)
    }

    /// Build semantic node for source file (root)
    fn build_source_file_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let mut children = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            // Skip trivial syntax tokens at the root level
            if self.is_trivial_syntax_token(child.kind()) {
                continue;
            }

            // Skip attribute items - they will be picked up by their target nodes
            if child.kind() == "attribute_item" {
                continue;
            }

            if child.kind() == "impl_item" {
                // Special handling for impl blocks - extract the methods
                match self.build_impl_items(child, source, Some(node)) {
                    Ok(methods) => children.extend(methods),
                    Err(_) => {
                        // Skip impl blocks that failed to parse - no Unknown nodes
                        continue;
                    }
                }
            } else {
                // Pass the source_file node as parent so children can find their attributes
                match self.build_semantic_node(child, source, Some(node), None) {
                    Ok(child_node) => children.push(child_node),
                    Err(_) => {
                        // Skip children that failed to parse - no Unknown nodes for trivial tokens
                        continue;
                    }
                }
            }
        }

        let mut root_node = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Module {
                module_type: ModuleType::File,
                is_public: true,
                metadata: HashMap::new(),
            },
            Vec::new(), // Source file doesn't have metadata nodes
        );
        root_node.children = children;

        Ok(root_node)
    }

    /// Build semantic node for function
    fn build_function_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let parameters_node = node.child_by_field_name("parameters");
        let return_type_node = node.child_by_field_name("return_type");
        let body_node = node.child_by_field_name("body");
        let type_parameters = node.child_by_field_name("type_parameters");

        // Extract metadata
        let mut metadata = HashMap::new();
        if let Some(parent) = parent_context {
            metadata.insert("parent_impl".to_string(), parent.to_string());
        }

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            params.child_count().saturating_sub(2) // Subtract parentheses
        } else {
            0
        };

        // Extract return type
        let return_type = return_type_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        // Check for async
        let is_async = Self::node_has_keyword(node, "async");

        // Extract visibility
        let visibility = self.extract_visibility(node, source);

        // Find metadata nodes (attributes) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_attributes(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
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
                body_node,
                metadata,
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for struct
    fn build_struct_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let body = node.child_by_field_name("body");

        // Count fields
        let field_count = body.map(|body_node| self.count_struct_fields(body_node));

        let visibility = self.extract_visibility(node, source);

        // Find metadata nodes (attributes) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_attributes(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: type_parameters.is_some(),
                field_count,
                inheritance: Vec::new(), // Rust doesn't have inheritance
                visibility,
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for enum
    fn build_enum_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let body = node.child_by_field_name("body");

        // Count variants
        let field_count = body.map(|body_node| self.count_enum_variants(body_node));

        let visibility = self.extract_visibility(node, source);

        // Find metadata nodes (attributes) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_attributes(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: type_parameters.is_some(),
                field_count,
                inheritance: Vec::new(),
                visibility,
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for module
    fn build_module_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let visibility = self.extract_visibility(node, source);

        let mut children = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                // Skip trivial syntax tokens in modules
                if self.is_trivial_syntax_token(child.kind()) {
                    continue;
                }

                if let Ok(child_node) = self.build_semantic_node(child, source, Some(body), None) {
                    children.push(child_node);
                }
            }
        }

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

        Ok(module_node)
    }

    /// Build semantic node for import/use declaration
    fn build_import_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        // Extract import information
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

    /// Build semantic node for const/static variable
    fn build_variable_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_node = node.child_by_field_name("type");
        let visibility = self.extract_visibility(node, source);

        let type_annotation = type_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());

        let is_const = node.kind() == "const_item";
        let is_static = node.kind() == "static_item";

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Variable {
                is_const,
                is_static,
                type_annotation,
                visibility,
                metadata: HashMap::new(),
            },
            Vec::new(),
        ))
    }

    /// Build semantic nodes for impl block contents (skip the impl itself)
    fn build_impl_items<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<Vec<SemanticNode<'a>>, SemanticError> {
        // Extract the target type for context
        let target_type = node
            .child_by_field_name("type")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("Unknown");

        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_item" {
                    if let Ok(method_node) =
                        self.build_function_node(child, source, parent, Some(target_type))
                    {
                        methods.push(method_node);
                    }
                }
            }
        }

        Ok(methods)
    }

    /// Helper methods for metadata extraction
    fn extract_visibility(&self, node: Node<'_>, source: &str) -> String {
        // Look for visibility modifier as previous sibling
        if let Some(parent) = node.parent() {
            let mut cursor = parent.walk();
            for child in parent.children(&mut cursor) {
                if child.id() == node.id() {
                    break;
                }
                if child.kind() == "visibility_modifier" {
                    if let Ok(vis_text) = child.utf8_text(source.as_bytes()) {
                        return vis_text.to_string();
                    }
                }
            }
        }
        "private".to_string()
    }

    fn node_has_keyword(node: Node<'_>, keyword: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Check if this child is the keyword we're looking for
            if child.kind() == keyword {
                return true;
            }

            // Special handling for function_modifiers which contains async/pub/const etc.
            if child.kind() == "function_modifiers" && Self::node_has_keyword(child, keyword) {
                return true;
            }

            // For other container nodes, we could add recursive search here if needed
            // but for now, let's be specific about function_modifiers
        }
        false
    }

    fn count_struct_fields(&self, body_node: Node<'_>) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                count += 1;
            }
        }
        count
    }

    fn count_enum_variants(&self, body_node: Node<'_>) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "enum_variant" {
                count += 1;
            }
        }
        count
    }

    fn parse_use_declaration(
        &self,
        node: Node<'_>,
        source: &str,
    ) -> std::result::Result<(ImportType, String, Vec<String>), SemanticError> {
        // Simple implementation - extract the use path
        if let Ok(use_text) = node.utf8_text(source.as_bytes()) {
            let use_text = use_text
                .trim_start_matches("use ")
                .trim_end_matches(';')
                .trim();

            if use_text.contains("*") {
                Ok((
                    ImportType::Wildcard,
                    use_text.replace("::*", ""),
                    Vec::new(),
                ))
            } else if use_text.contains("{") {
                // Parse specific imports
                let parts: Vec<&str> = use_text.split("::").collect();
                let module = parts[..parts.len() - 1].join("::");
                let items_part = parts.last().map_or("", |v| v);
                let items: Vec<String> = items_part
                    .trim_matches(|c| c == '{' || c == '}')
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                Ok((ImportType::Specific, module, items))
            } else {
                Ok((ImportType::Default, use_text.to_string(), Vec::new()))
            }
        } else {
            Err(SemanticError::TreeBuildError(
                "Failed to extract use declaration text".to_string(),
            ))
        }
    }

    /// Rust-specific similarity comparison methods
    fn compare_rust_callables(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        _old_source: &dyn crate::ast_diff::SourceProvider,
        _new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        if let (
            SemanticUnitType::Callable {
                parameter_count: old_params,
                return_type: old_ret,
                ..
            },
            SemanticUnitType::Callable {
                parameter_count: new_params,
                return_type: new_ret,
                ..
            },
        ) = (&old.unit_type, &new.unit_type)
        {
            if old_params == new_params && old_ret == new_ret {
                SemanticSimilarity::body_change()
            } else {
                let param_similarity = if old_params == new_params { 0.5 } else { 0.0 };
                let return_similarity = if old_ret == new_ret { 0.5 } else { 0.0 };

                SemanticSimilarity::signature_change(true, param_similarity + return_similarity)
            }
        } else {
            SemanticSimilarity::unrelated()
        }
    }

    fn compare_rust_data_structures(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        if let (
            SemanticUnitType::DataStructure {
                field_count: old_fields,
                ..
            },
            SemanticUnitType::DataStructure {
                field_count: new_fields,
                ..
            },
        ) = (&old.unit_type, &new.unit_type)
        {
            // Check for structural changes: field count or attributes
            let fields_changed = old_fields != new_fields;
            let attributes_changed = self.compare_metadata_nodes(old, new, old_source, new_source);

            if fields_changed || attributes_changed {
                SemanticSimilarity::structural_refactor(0.8)
            } else {
                SemanticSimilarity::body_change()
            }
        } else {
            SemanticSimilarity::unrelated()
        }
    }

    /// Compare metadata nodes (attributes) between old and new semantic units
    fn compare_metadata_nodes(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        // If different number of metadata nodes, they're different
        if old.metadata_nodes.len() != new.metadata_nodes.len() {
            return true;
        }

        // Compare each metadata node's text content
        for (old_meta, new_meta) in old.metadata_nodes.iter().zip(new.metadata_nodes.iter()) {
            let old_text = old_source.node_text(&old_meta.node).unwrap_or_default();
            let new_text = new_source.node_text(&new_meta.node).unwrap_or_default();

            if old_text != new_text {
                return true;
            }
        }

        false // No differences found
    }

    fn analyze_potential_rename(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        _old_source: &dyn crate::ast_diff::SourceProvider,
        _new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        // For renames, compare structural similarity
        match (&old.unit_type, &new.unit_type) {
            (
                SemanticUnitType::Callable {
                    parameter_count: old_params,
                    return_type: old_ret,
                    ..
                },
                SemanticUnitType::Callable {
                    parameter_count: new_params,
                    return_type: new_ret,
                    ..
                },
            ) => {
                if old_params == new_params && old_ret == new_ret {
                    SemanticSimilarity::name_refactor(0.9)
                } else {
                    SemanticSimilarity::structural_refactor(0.6)
                }
            }
            _ => SemanticSimilarity::structural_refactor(0.5),
        }
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RustParser {
    /// Find preceding attribute siblings for a given node
    fn find_preceding_attributes<'a>(
        &self,
        node: Node<'a>,
        parent: Node<'a>,
    ) -> Vec<MetadataNode<'a>> {
        let mut metadata_nodes = Vec::new();
        let mut cursor = parent.walk();
        let mut position = -1;

        // Iterate through parent's children in reverse to find preceding siblings
        let children: Vec<Node> = parent.children(&mut cursor).collect();
        let target_index = children.iter().position(|&n| n == node);

        if let Some(idx) = target_index {
            // Look backwards from the target node
            for i in (0..idx).rev() {
                let sibling = children[i];

                if sibling.kind() == "attribute_item" {
                    metadata_nodes.push(MetadataNode {
                        node: sibling,
                        position: MetadataPosition::PrecedingSibling(position),
                    });
                    position -= 1;
                } else if !self.is_trivial_syntax_token(sibling.kind()) {
                    // Stop at the first non-attribute, non-trivial node
                    break;
                }
            }
        }

        // Reverse to maintain proper order (closest to furthest)
        metadata_nodes.reverse();
        metadata_nodes
    }
}

impl LanguageParser for RustParser {
    /// Parse Rust content into TreeSitter AST
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set Rust language: {e}"),
            })?;

        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse Rust code".to_string(),
            })
    }

    /// Get the TreeSitter language for Rust
    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_rust::language()
    }

    /// Build semantic AST tree from Rust TreeSitter AST
    fn build_semantic_tree<'a>(
        &self,
        ast: &'a Tree,
        source: &str,
    ) -> std::result::Result<SemanticTree<'a>, SemanticError> {
        let root_node = ast.root_node();
        let semantic_root = self.build_semantic_node(root_node, source, None, None)?;

        Ok(SemanticTree::new(semantic_root, ProgrammingLanguage::Rust))
    }

    /// Compare two Rust semantic units for similarity
    fn compare_semantic_units(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        // Must be same unit type to be comparable
        if old.unit_type_name() != new.unit_type_name() {
            return SemanticSimilarity::unrelated();
        }

        let old_name = old
            .name_node
            .and_then(|node| old_source.node_text(&node).ok())
            .map(|s| s.to_string());
        let new_name = new
            .name_node
            .and_then(|node| new_source.node_text(&node).ok())
            .map(|s| s.to_string());

        match (&old_name, &new_name) {
            (Some(old_n), Some(new_n)) if old_n == new_n => {
                // Same name - check for body changes based on unit type
                match (&old.unit_type, &new.unit_type) {
                    (SemanticUnitType::Callable { .. }, SemanticUnitType::Callable { .. }) => {
                        self.compare_rust_callables(old, new, old_source, new_source)
                    }
                    (
                        SemanticUnitType::DataStructure { .. },
                        SemanticUnitType::DataStructure { .. },
                    ) => self.compare_rust_data_structures(old, new, old_source, new_source),
                    _ => {
                        // For other unit types, compare the full node content
                        let old_text = old_source.node_text(&old.tree_sitter_node).unwrap_or("");
                        let new_text = new_source.node_text(&new.tree_sitter_node).unwrap_or("");

                        if old_text == new_text {
                            SemanticSimilarity::identical()
                        } else {
                            SemanticSimilarity::body_change()
                        }
                    }
                }
            }
            (Some(_), Some(_)) => {
                // Different names - could be rename or structural change
                self.analyze_potential_rename(old, new, old_source, new_source)
            }
            _ => SemanticSimilarity::unrelated(), // One or both have no name
        }
    }

    /// Classify TreeSitter node kinds into semantic categories for Rust
    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            // Core language constructs
            "function_item" => SemanticNodeKind::Function,
            "struct_item" => SemanticNodeKind::Struct,
            "enum_item" => SemanticNodeKind::Enum,
            "impl_item" => SemanticNodeKind::ImplBlock,
            "trait_item" => SemanticNodeKind::Interface,

            // Module and imports
            "mod_item" => SemanticNodeKind::Module,
            "use_declaration" => SemanticNodeKind::Import,

            // Variables and declarations
            "let_declaration" | "const_item" | "static_item" => SemanticNodeKind::Variable,

            // Statements and expressions
            "expression_statement" | "assignment_expression" => SemanticNodeKind::Statement,
            "call_expression"
            | "method_call_expression"
            | "binary_expression"
            | "unary_expression"
            | "field_expression"
            | "index_expression" => SemanticNodeKind::Expression,

            // Type definitions
            "type_alias" => SemanticNodeKind::TypeDefinition,

            // Comments
            "line_comment" | "block_comment" => SemanticNodeKind::Comment,

            // Source file root
            "source_file" => SemanticNodeKind::SourceFile,

            // Everything else
            _ => SemanticNodeKind::Other(node_kind.to_string()),
        }
    }

    /// Get Rust-specific context boundaries for different change types
    fn get_context_boundaries(
        &self,
        change_type: &crate::ast_diff::ASTChangeType,
        change_node_kind: &SemanticNodeKind,
    ) -> Vec<SemanticNodeKind> {
        match (change_type, change_node_kind) {
            // Function changes should look for enclosing impl blocks, structs, or modules
            (_, SemanticNodeKind::Function) => vec![
                SemanticNodeKind::ImplBlock,
                SemanticNodeKind::Struct,
                SemanticNodeKind::Enum,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],

            // Struct/Enum changes should look for enclosing modules
            (_, SemanticNodeKind::Struct | SemanticNodeKind::Enum) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }

            // Impl block changes should include the struct/enum being implemented
            (_, SemanticNodeKind::ImplBlock) => vec![
                SemanticNodeKind::Struct,
                SemanticNodeKind::Enum,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],

            // Trait changes should look for module context
            (_, SemanticNodeKind::Interface) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }

            // Import changes should look for appropriate boundaries depending on context
            (_, SemanticNodeKind::Import) => {
                vec![
                    SemanticNodeKind::Function,
                    SemanticNodeKind::Module,
                    SemanticNodeKind::SourceFile,
                ]
            }

            // Variable/Statement/Expression changes should look for enclosing function
            (
                _,
                SemanticNodeKind::Variable
                | SemanticNodeKind::Statement
                | SemanticNodeKind::Expression,
            ) => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::ImplBlock,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],

            // Module changes should look for parent modules
            (_, SemanticNodeKind::Module) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }

            // Type definitions should look for module context
            (_, SemanticNodeKind::TypeDefinition) => {
                vec![SemanticNodeKind::Module, SemanticNodeKind::SourceFile]
            }

            // For other types, provide general boundaries
            _ => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::ImplBlock,
                SemanticNodeKind::Struct,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ASTChangeType;

    #[test]
    fn test_rust_parser_creation() {
        let parser = RustParser::new();
        let _language = parser.get_language();
        // Basic creation test
    }

    #[test]
    fn test_classify_node_kinds() {
        let parser = RustParser::new();

        assert!(matches!(
            parser.classify_node_kind("function_item"),
            SemanticNodeKind::Function
        ));
        assert!(matches!(
            parser.classify_node_kind("struct_item"),
            SemanticNodeKind::Struct
        ));
        assert!(matches!(
            parser.classify_node_kind("enum_item"),
            SemanticNodeKind::Enum
        ));
        assert!(matches!(
            parser.classify_node_kind("impl_item"),
            SemanticNodeKind::ImplBlock
        ));
        assert!(matches!(
            parser.classify_node_kind("unknown"),
            SemanticNodeKind::Other(_)
        ));
    }

    #[test]
    fn test_parse_simple_rust() {
        let parser = RustParser::new();
        let code = "fn main() { println!(\"hello\"); }";

        let tree = parser.try_parse(code).unwrap();
        let root = tree.root_node();

        assert_eq!(root.kind(), "source_file");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn test_context_boundaries() {
        let parser = RustParser::new();

        // Test function boundaries
        let boundaries =
            parser.get_context_boundaries(&ASTChangeType::Content, &SemanticNodeKind::Function);

        assert!(boundaries.contains(&SemanticNodeKind::ImplBlock));
        assert!(boundaries.contains(&SemanticNodeKind::Module));

        // Test struct boundaries
        let struct_boundaries =
            parser.get_context_boundaries(&ASTChangeType::Structural, &SemanticNodeKind::Struct);

        assert!(struct_boundaries.contains(&SemanticNodeKind::Module));
        assert!(struct_boundaries.contains(&SemanticNodeKind::SourceFile));
    }
}
