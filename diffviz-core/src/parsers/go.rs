//! Go language parser implementation

use crate::common::{ASTError, LanguageParser, Result, SemanticNodeKind};
use crate::semantic_ast::{
    MetadataNode, MetadataPosition, SemanticError, SemanticNode, SemanticTree, SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Parser, Tree};

pub struct GoParser;

impl GoParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for GoParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_go::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set Go language: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse Go code".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_go::language()
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            "function_declaration" | "method_declaration" => SemanticNodeKind::Function,
            "struct_type" => SemanticNodeKind::Struct,
            "interface_type" => SemanticNodeKind::Interface,
            "package_clause" => SemanticNodeKind::Module,
            "import_declaration" => SemanticNodeKind::Import,
            "var_declaration" | "const_declaration" | "short_var_declaration" => {
                SemanticNodeKind::Variable
            }
            "expression_statement" | "assignment_statement" | "inc_statement" | "dec_statement" => {
                SemanticNodeKind::Statement
            }
            "call_expression"
            | "selector_expression"
            | "index_expression"
            | "binary_expression" => SemanticNodeKind::Expression,
            // Signature components
            "parameter_list" | "parameter_declaration" | "result" => {
                SemanticNodeKind::SignatureComponent
            }
            "comment" => SemanticNodeKind::Comment,
            "source_file" => SemanticNodeKind::SourceFile,
            _ => SemanticNodeKind::Other(node_kind.to_string()),
        }
    }

    fn build_semantic_tree<'a>(
        &self,
        ast: &'a Tree,
        source: &str,
    ) -> std::result::Result<SemanticTree<'a>, SemanticError> {
        let root_node = ast.root_node();
        let semantic_root = self.build_semantic_node(root_node, source, None, None)?;

        Ok(SemanticTree::new(
            semantic_root,
            crate::common::ProgrammingLanguage::Go,
        ))
    }
}

impl GoParser {
    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ";" | ":" | "." |
            "?" | "!" | "#" | "@" | "$" | "%" | "^" | "&" | "*" | "-" | 
            "=" | "+" | "|" | "\\" | "/" | "<" | ">" | ":=" |

            // Keywords that are part of larger constructs
            "func" | "struct" | "interface" | "type" | "const" | "var" |
            "package" | "import" | "if" | "else" | "for" | "range" | "switch" |
            "case" | "default" | "go" | "defer" | "return" | "break" | "continue" |
            "chan" | "map" | "select" |

            // Literals and identifiers
            "string_literal" | "int_literal" | "float_literal" | "identifier" |
            "field_identifier" | "type_identifier" | "package_identifier" |

            // Comments and whitespace
            "comment" | "line_comment" | "block_comment" |

            // Error nodes
            "ERROR" | "MISSING"
        )
    }

    /// Find preceding comment/build tag siblings for a given node (Go doesn't have decorators but has build tags)
    fn find_preceding_build_tags<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        parent: tree_sitter::Node<'a>,
    ) -> Vec<MetadataNode<'a>> {
        let mut metadata_nodes = Vec::new();
        let mut cursor = parent.walk();
        let mut position = -1;

        // Iterate through parent's children to find preceding siblings
        let children: Vec<tree_sitter::Node> = parent.children(&mut cursor).collect();
        let target_index = children.iter().position(|&n| n == node);

        if let Some(idx) = target_index {
            // Look backwards from the target node
            for i in (0..idx).rev() {
                let sibling = children[i];

                // Go build tags are comments with special format: // +build
                if sibling.kind() == "comment" {
                    metadata_nodes.push(MetadataNode {
                        node: sibling,
                        position: MetadataPosition::PrecedingSibling(position),
                    });
                    position -= 1;
                } else if !self.is_trivial_syntax_token(sibling.kind()) {
                    // Stop at the first non-comment, non-trivial node
                    break;
                }
            }
        }

        // Reverse to maintain proper order (closest to furthest)
        metadata_nodes.reverse();
        metadata_nodes
    }

    /// Build semantic node for Go AST node
    fn build_semantic_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "source_file" => self.build_source_file_node(node, source),
            "function_declaration" => {
                self.build_function_node(node, source, parent, parent_context)
            }
            "method_declaration" => self.build_method_node(node, source, parent, parent_context),
            "struct_type" => self.build_struct_node(node, source, parent),
            "interface_type" => self.build_interface_node(node, source, parent),
            "package_clause" => self.build_package_node(node, source),
            "import_declaration" => self.build_import_node(node, source),
            "var_declaration" | "const_declaration" => {
                self.build_variable_node(node, source, parent)
            }
            _ => {
                // Skip trivial syntax tokens
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
                    if !self.is_trivial_syntax_token(child.kind()) {
                        match self.build_semantic_node(child, source, Some(node), parent_context) {
                            Ok(child_node) => meaningful_children.push(child_node),
                            Err(_) => continue,
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

    /// Build semantic node for Go source file
    fn build_source_file_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let mut children = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if let Ok(child_node) = self.build_semantic_node(child, source, Some(node), None) {
                children.push(child_node);
            }
        }

        let mut root_node = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Module {
                module_type: crate::semantic_ast::ModuleType::File,
                is_public: true,
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        );
        root_node.children = children;

        Ok(root_node)
    }

    /// Build semantic node for Go function
    fn build_function_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        _parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");
        let result_node = node.child_by_field_name("result");

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            self.count_parameters(params)
        } else {
            0
        };

        // Extract return type
        let return_type = result_node.map(|result| {
            result
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string()
        });

        // Determine visibility from function name (capitalized = public)
        let visibility = if let Some(name_node) = name_node {
            self.get_visibility_from_name(name_node, source)
        } else {
            "private".to_string()
        };

        // Find metadata nodes (build tags/comments) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_build_tags(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic: false, // TODO: Detect generics in Go
                parameter_count,
                return_type,
                is_async: false, // Go doesn't have async functions like this
                visibility,
                is_method: false,
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for Go method
    fn build_method_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        _parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = node.child_by_field_name("name");
        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");
        let result_node = node.child_by_field_name("result");

        // Count parameters (excluding receiver)
        let parameter_count = if let Some(params) = parameters_node {
            self.count_parameters(params)
        } else {
            0
        };

        // Extract return type
        let return_type = result_node.map(|result| {
            result
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string()
        });

        // Determine visibility from method name (capitalized = public)
        let visibility = if let Some(name_node) = name_node {
            self.get_visibility_from_name(name_node, source)
        } else {
            "private".to_string()
        };

        // Find metadata nodes (build tags/comments) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_build_tags(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic: false,
                parameter_count,
                return_type,
                is_async: false,
                visibility,
                is_method: true, // This is a method
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for Go struct
    fn build_struct_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = None; // Struct types don't always have names in Go
        let body = node.child_by_field_name("body");

        // Count fields
        let field_count = body.map(|body_node| self.count_struct_fields(body_node));

        // Find metadata nodes (build tags/comments) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_build_tags(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: false, // TODO: Detect generics
                field_count,
                inheritance: vec![],
                visibility: "public".to_string(),
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for Go interface
    fn build_interface_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = None;
        let body = node.child_by_field_name("body");

        // Count methods in interface
        let field_count = body.map(|body_node| self.count_interface_methods(body_node));

        // Find metadata nodes (build tags/comments) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_build_tags(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: false,
                field_count,
                inheritance: vec![],
                visibility: "public".to_string(),
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for Go package clause
    fn build_package_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = node.child_by_field_name("name");

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Module {
                module_type: crate::semantic_ast::ModuleType::Namespace,
                is_public: true,
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        ))
    }

    /// Build semantic node for Go import
    fn build_import_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        Ok(SemanticNode::new(
            node,
            None,
            SemanticUnitType::Import {
                import_type: crate::semantic_ast::ImportType::Specific,
                source_module: "".to_string(), // TODO: Extract actual import path
                imported_items: vec![],
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        ))
    }

    /// Build semantic node for Go variable/constant
    fn build_variable_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        // Find metadata nodes (build tags/comments) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_build_tags(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            None,
            SemanticUnitType::Variable {
                is_const: node.kind() == "const_declaration",
                is_static: false, // TODO: Detect scope
                type_annotation: None,
                visibility: "public".to_string(),
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    // Helper methods

    fn count_parameters(&self, params_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                count += 1;
            }
        }
        count
    }

    fn count_struct_fields(&self, body_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                count += 1;
            }
        }
        count
    }

    fn count_interface_methods(&self, body_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "method_spec" {
                count += 1;
            }
        }
        count
    }

    /// Get visibility from Go naming convention (capitalized = public)
    fn get_visibility_from_name(&self, name_node: tree_sitter::Node, source: &str) -> String {
        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
            if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                "public".to_string()
            } else {
                "private".to_string()
            }
        } else {
            "private".to_string()
        }
    }
}
