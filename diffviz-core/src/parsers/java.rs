//! Java language parser implementation
//!
//! This module provides Java-specific parsing using TreeSitter and implements
//! the LanguageParser trait for semantic AST analysis. It handles Java's
//! object-oriented features including classes, interfaces, generics, annotations,
//! inheritance, and package system.

use crate::common::{ASTError, LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::semantic_ast::{
    ImportType, MetadataNode, MetadataPosition, ModuleType, SemanticError, SemanticNode,
    SemanticSimilarity, SemanticTree, SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Tree};

pub struct JavaParser;

impl JavaParser {
    pub fn new() -> Self {
        Self
    }

    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ";" | "." | ":" | "?" | "!" |
            "@" | "#" | "$" | "%" | "^" | "&" | "*" | "-" | "=" | "+" | "|" | "\\" |
            "/" | "<" | ">" | "<<" | ">>" | ">>>" | "++" | "--" | "==" | "!=" |
            "<=" | ">=" | "&&" | "||" | "->" | "::" |

            // Java keywords that are part of larger constructs
            "if" | "else" | "for" | "while" | "do" | "switch" | "case" | "default" |
            "break" | "continue" | "return" | "throw" | "try" | "catch" | "finally" |
            "synchronized" | "volatile" | "transient" | "native" | "strictfp" |
            "abstract" | "final" | "static" | "public" | "private" | "protected" |
            "class" | "interface" | "enum" | "extends" | "implements" | "super" | "this" |
            "new" | "instanceof" | "assert" | "package" | "import" |

            // Type keywords
            "void" | "boolean" | "byte" | "short" | "int" | "long" | "float" | "double" |
            "char" | "var" |

            // Literals and identifiers
            "string_literal" | "character_literal" | "decimal_integer_literal" |
            "hex_integer_literal" | "octal_integer_literal" | "binary_integer_literal" |
            "decimal_floating_point_literal" | "hex_floating_point_literal" |
            "true" | "false" | "null" |
            "identifier" | "type_identifier" |

            // Comments
            "comment" | "line_comment" | "block_comment" |

            // Statement and expression constructs that should not be semantic nodes
            "expression_statement" | "assignment_expression" | "binary_expression" |
            "method_invocation" | "field_access" | "array_access" | "parenthesized_expression" |
            "unary_expression" | "conditional_expression" | "block" | "if_statement" |
            "return_statement" | "throw_statement" | "local_variable_declaration" |
            "enhanced_for_statement" | "for_statement" | "while_statement" | "do_statement" |

            // Type and generic constructs
            "type_arguments" | "type_parameters" | "wildcard" | "generic_type" |
            "array_type" | "integral_type" | "floating_point_type" | "boolean_type" |

            // Annotation elements (handled as metadata)
            "annotation" | "marker_annotation" | "annotation_argument_list"
        )
    }
}

impl JavaParser {
    /// Build semantic node from tree-sitter AST node
    fn build_semantic_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "program" => self.build_program_node(node, source),
            "class_declaration" => self.build_class_node(node, source, parent),
            "interface_declaration" => self.build_interface_node(node, source, parent),
            "enum_declaration" => self.build_enum_node(node, source, parent),
            "method_declaration" => self.build_method_node(node, source, parent, parent_context),
            "constructor_declaration" => {
                self.build_constructor_node(node, source, parent, parent_context)
            }
            "field_declaration" => self.build_field_node(node, source, parent),
            "import_declaration" => self.build_import_node(node, source, parent),
            "package_declaration" => self.build_package_node(node, source, parent),
            _ => {
                // Skip trivial syntax tokens
                if self.is_trivial_syntax_token(node.kind()) {
                    return Err(SemanticError::TreeBuildError(format!(
                        "Skipping trivial syntax token: {}",
                        node.kind()
                    )));
                }

                // For non-trivial unknown nodes, try to process children
                let mut children = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !self.is_trivial_syntax_token(child.kind()) {
                        match self.build_semantic_node(child, source, Some(node), parent_context) {
                            Ok(child_node) => children.push(child_node),
                            Err(_) => continue,
                        }
                    }
                }

                // If we found meaningful children, create a container node
                if !children.is_empty() {
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
                    container_node.children = children;
                    Ok(container_node)
                } else {
                    Err(SemanticError::TreeBuildError(format!(
                        "Unknown node with no meaningful children: {}",
                        node.kind()
                    )))
                }
            }
        }
    }

    /// Build program (root) node
    fn build_program_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let mut children = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if !self.is_trivial_syntax_token(child.kind()) {
                match self.build_semantic_node(child, source, Some(node), Some("file")) {
                    Ok(child_node) => children.push(child_node),
                    Err(_) => continue,
                }
            }
        }

        let mut semantic_node = SemanticNode::new(
            node,
            None,
            SemanticUnitType::Module {
                module_type: ModuleType::File,
                is_public: true,
                metadata: HashMap::new(),
            },
            Vec::new(),
        );
        semantic_node.children = children;
        Ok(semantic_node)
    }

    /// Build class node (Java class declaration)
    fn build_class_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let superclass = node.child_by_field_name("superclass");
        let interfaces = node.child_by_field_name("interfaces");
        let body = node.child_by_field_name("body");

        // Extract inheritance information
        let mut inheritance = Vec::new();
        if let Some(superclass_node) = superclass {
            if let Ok(superclass_name) = superclass_node.utf8_text(source.as_bytes()) {
                inheritance.push(superclass_name.to_string());
            }
        }
        if let Some(interfaces_node) = interfaces {
            // Parse interface list
            let mut cursor = interfaces_node.walk();
            for child in interfaces_node.children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    if let Ok(interface_name) = child.utf8_text(source.as_bytes()) {
                        inheritance.push(interface_name.to_string());
                    }
                }
            }
        }

        // Extract visibility and other modifiers
        let visibility = self.extract_visibility(node, source);
        let is_generic = type_parameters.is_some();

        // Count fields (approximate)
        let field_count = if let Some(body_node) = body {
            let mut count = 0;
            let mut cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    count += 1;
                }
            }
            Some(count)
        } else {
            None
        };

        // Find annotations if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_annotations(node, parent_node)
        } else {
            Vec::new()
        };

        // Extract class name for context
        let class_name = name_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("Unknown");

        // Process class body contents
        let mut class_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic,
                field_count,
                inheritance,
                visibility,
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        );

        // Add child semantic units from class body
        if let Some(body_node) = body {
            let mut children = Vec::new();
            let mut cursor = body_node.walk();

            for child in body_node.children(&mut cursor) {
                if !self.is_trivial_syntax_token(child.kind()) {
                    match self.build_semantic_node(child, source, Some(body_node), Some(class_name))
                    {
                        Ok(child_node) => children.push(child_node),
                        Err(_) => continue,
                    }
                }
            }

            class_node.children = children;
        }

        Ok(class_node)
    }

    /// Build interface node (Java interface declaration)
    fn build_interface_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let interfaces = node.child_by_field_name("extends");
        let body = node.child_by_field_name("body");

        // Extract inheritance information (extended interfaces)
        let mut inheritance = Vec::new();
        if let Some(interfaces_node) = interfaces {
            let mut cursor = interfaces_node.walk();
            for child in interfaces_node.children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    if let Ok(interface_name) = child.utf8_text(source.as_bytes()) {
                        inheritance.push(interface_name.to_string());
                    }
                }
            }
        }

        let visibility = self.extract_visibility(node, source);
        let is_generic = type_parameters.is_some();

        // Count methods in interface
        let field_count = if let Some(body_node) = body {
            let mut count = 0;
            let mut cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                if matches!(child.kind(), "method_declaration" | "constant_declaration") {
                    count += 1;
                }
            }
            Some(count)
        } else {
            None
        };

        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_annotations(node, parent_node)
        } else {
            Vec::new()
        };

        // Extract interface name for context
        let interface_name = name_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("Unknown");

        // Process interface body contents
        let mut interface_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic,
                field_count,
                inheritance,
                visibility,
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        );

        // Add child semantic units from interface body
        if let Some(body_node) = body {
            let mut children = Vec::new();
            let mut cursor = body_node.walk();

            for child in body_node.children(&mut cursor) {
                if !self.is_trivial_syntax_token(child.kind()) {
                    match self.build_semantic_node(
                        child,
                        source,
                        Some(body_node),
                        Some(interface_name),
                    ) {
                        Ok(child_node) => children.push(child_node),
                        Err(_) => continue,
                    }
                }
            }

            interface_node.children = children;
        }

        Ok(interface_node)
    }

    /// Build enum node (Java enum declaration)
    fn build_enum_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let interfaces = node.child_by_field_name("interfaces");
        let body = node.child_by_field_name("body");

        // Extract implemented interfaces
        let mut inheritance = Vec::new();
        if let Some(interfaces_node) = interfaces {
            let mut cursor = interfaces_node.walk();
            for child in interfaces_node.children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    if let Ok(interface_name) = child.utf8_text(source.as_bytes()) {
                        inheritance.push(interface_name.to_string());
                    }
                }
            }
        }

        let visibility = self.extract_visibility(node, source);

        // Count enum constants
        let field_count = if let Some(body_node) = body {
            let mut count = 0;
            let mut cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                if child.kind() == "enum_constant" {
                    count += 1;
                }
            }
            Some(count)
        } else {
            None
        };

        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_annotations(node, parent_node)
        } else {
            Vec::new()
        };

        // Process enum body contents
        let enum_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: false, // Java enums can't be generic
                field_count,
                inheritance,
                visibility,
                signature_node: body,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        );

        // For enums, don't process children separately to avoid duplication
        // The enum body change will be detected through the main enum comparison
        // This prevents enum fields/methods from appearing as separate changes
        // when the overall enum is transforming

        Ok(enum_node)
    }

    /// Helper methods for extracting Java-specific information
    fn extract_visibility(&self, node: Node<'_>, _source: &str) -> String {
        // Look for visibility modifiers in the node or its siblings
        if let Some(parent) = node.parent() {
            let mut cursor = parent.walk();
            for child in parent.children(&mut cursor) {
                if child == node {
                    break;
                }
                match child.kind() {
                    "public" => return "public".to_string(),
                    "private" => return "private".to_string(),
                    "protected" => return "protected".to_string(),
                    _ => continue,
                }
            }
        }

        // Also check modifiers within the node itself
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "modifiers" => {
                    let mut mod_cursor = child.walk();
                    for modifier in child.children(&mut mod_cursor) {
                        match modifier.kind() {
                            "public" => return "public".to_string(),
                            "private" => return "private".to_string(),
                            "protected" => return "protected".to_string(),
                            _ => continue,
                        }
                    }
                }
                _ => continue,
            }
        }

        "package".to_string() // Default package visibility in Java
    }

    /// Build method node (Java method declaration)
    fn build_method_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");
        let return_type_node = node.child_by_field_name("type");

        // Extract metadata
        let mut metadata = HashMap::new();
        if let Some(parent) = parent_context {
            metadata.insert("parent_class".to_string(), parent.to_string());
        }

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            let mut count = 0;
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "formal_parameter" {
                    count += 1;
                }
            }
            count
        } else {
            0
        };

        // Extract return type
        let return_type = return_type_node
            .and_then(|n| n.utf8_text(source.as_bytes()).ok().map(|s| s.to_string()));

        // Check if this is a static method, abstract, etc.
        let visibility = self.extract_visibility(node, source);
        let is_static = self.has_modifier(node, "static");
        let is_generic = type_parameters.is_some();

        // Determine if it's a method (has instance context) vs static function
        let is_method = !is_static && parent_context.is_some();

        // Find annotations
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_annotations(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic,
                parameter_count,
                return_type,
                is_async: false, // Java doesn't have native async/await
                visibility,
                is_method,
                signature_node: parameters_node,
                body_node,
                metadata,
            },
            metadata_nodes,
        ))
    }

    /// Build constructor node (Java constructor declaration)
    fn build_constructor_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let type_parameters = node.child_by_field_name("type_parameters");
        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");

        // Extract metadata
        let mut metadata = HashMap::new();
        metadata.insert("is_constructor".to_string(), "true".to_string());
        if let Some(parent) = parent_context {
            metadata.insert("parent_class".to_string(), parent.to_string());
        }

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            let mut count = 0;
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "formal_parameter" {
                    count += 1;
                }
            }
            count
        } else {
            0
        };

        let visibility = self.extract_visibility(node, source);
        let is_generic = type_parameters.is_some();

        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_annotations(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic,
                parameter_count,
                return_type: None, // Constructors don't have return types
                is_async: false,
                visibility,
                is_method: true, // Constructors are always instance-related
                signature_node: parameters_node,
                body_node,
                metadata,
            },
            metadata_nodes,
        ))
    }

    /// Build field node (Java field declaration)
    fn build_field_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node
            .child_by_field_name("declarator")
            .and_then(|decl| decl.child_by_field_name("name"));
        let type_node = node.child_by_field_name("type");

        // Extract type annotation
        let type_annotation =
            type_node.and_then(|n| n.utf8_text(source.as_bytes()).ok().map(|s| s.to_string()));

        let visibility = self.extract_visibility(node, source);
        let is_static = self.has_modifier(node, "static");
        let is_final = self.has_modifier(node, "final");

        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_annotations(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Variable {
                is_const: is_final,
                is_static,
                type_annotation,
                visibility,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build import node (Java import declaration)
    fn build_import_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let import_text = node.utf8_text(source.as_bytes()).unwrap_or("");
        let is_static = import_text.contains("import static");

        // Find the scoped_identifier node (the import path)
        let name_node = {
            let mut cursor = node.walk();
            let mut name_node = None;
            for child in node.children(&mut cursor) {
                if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                    name_node = Some(child);
                    break;
                }
            }
            name_node
        };

        // Parse import statement
        let (import_type, source_module, imported_items) = if import_text.contains("*") {
            // Wildcard import: import java.util.*; or import static java.util.Collections.*;
            let module = if let Some(start) = import_text.find("import") {
                let after_import = &import_text[start + 6..].trim();
                let after_static = if let Some(stripped) = after_import.strip_prefix("static") {
                    stripped.trim()
                } else {
                    after_import
                };
                after_static
                    .trim_end_matches(".*;")
                    .trim_end_matches(".*")
                    .to_string()
            } else {
                "".to_string()
            };
            (ImportType::Wildcard, module, Vec::new())
        } else {
            // Regular import: import java.util.List;
            let module = if let Some(start) = import_text.find("import") {
                let after_import = &import_text[start + 6..].trim();
                let after_static = if let Some(stripped) = after_import.strip_prefix("static") {
                    stripped.trim()
                } else {
                    after_import
                };
                after_static.trim_end_matches(";").to_string()
            } else {
                "".to_string()
            };

            // For regular imports, the full path is both module and imported item
            let imported_items = if !module.is_empty() {
                vec![module.split('.').next_back().unwrap_or(&module).to_string()]
            } else {
                Vec::new()
            };

            let import_type = if is_static {
                ImportType::Namespace
            } else {
                ImportType::Specific
            };

            (import_type, module, imported_items)
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Import {
                import_type,
                source_module,
                imported_items,
                metadata: HashMap::new(),
            },
            Vec::new(),
        ))
    }

    /// Build package node (Java package declaration)
    fn build_package_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let package_text = node.utf8_text(source.as_bytes()).unwrap_or("");

        // Find the scoped_identifier node (the package path)
        let name_node = {
            let mut cursor = node.walk();
            let mut name_node = None;
            for child in node.children(&mut cursor) {
                if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                    name_node = Some(child);
                    break;
                }
            }
            name_node
        };

        let package_name = if let Some(start) = package_text.find("package") {
            package_text[start + 7..]
                .trim_end_matches(";")
                .trim()
                .to_string()
        } else {
            "".to_string()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Module {
                module_type: ModuleType::Namespace,
                is_public: true,
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("package_name".to_string(), package_name);
                    map
                },
            },
            Vec::new(),
        ))
    }

    /// Check if a node has a specific modifier
    fn has_modifier(&self, node: Node<'_>, modifier: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let mut mod_cursor = child.walk();
                for mod_child in child.children(&mut mod_cursor) {
                    if mod_child.kind() == modifier {
                        return true;
                    }
                }
            } else if child.kind() == modifier {
                return true;
            }
        }
        false
    }

    /// Find preceding annotations (Java's equivalent of Rust attributes)
    fn find_preceding_annotations<'a>(
        &self,
        node: Node<'a>,
        parent: Node<'a>,
    ) -> Vec<MetadataNode<'a>> {
        let mut metadata_nodes = Vec::new();
        let mut cursor = parent.walk();
        let mut position = -1;

        // Collect all children
        let children: Vec<Node> = parent.children(&mut cursor).collect();

        // Find our target node and look for preceding annotations
        if let Some(target_index) = children.iter().position(|&child| child == node) {
            // Look backwards for annotations
            for i in (0..target_index).rev() {
                let sibling = children[i];
                if matches!(sibling.kind(), "annotation" | "marker_annotation") {
                    metadata_nodes.push(MetadataNode {
                        node: sibling,
                        position: MetadataPosition::PrecedingSibling(position),
                    });
                    position -= 1;
                } else if !self.is_trivial_syntax_token(sibling.kind()) {
                    // Stop at first non-annotation, non-trivial node
                    break;
                }
            }
        }

        metadata_nodes.reverse(); // Maintain proper order
        metadata_nodes
    }

    /// Java-specific comparison for data structures (classes, interfaces, enums)
    fn compare_java_data_structures(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        if let (
            SemanticUnitType::DataStructure {
                is_generic: old_generic,
                inheritance: old_inheritance,
                field_count: old_fields,
                ..
            },
            SemanticUnitType::DataStructure {
                is_generic: new_generic,
                inheritance: new_inheritance,
                field_count: new_fields,
                ..
            },
        ) = (&old.unit_type, &new.unit_type)
        {
            // Check for structural changes
            let generics_changed = old_generic != new_generic;
            let inheritance_changed = old_inheritance != new_inheritance;
            let fields_changed = old_fields != new_fields;
            let annotations_changed = self.compare_annotations(old, new, old_source, new_source);

            if generics_changed || inheritance_changed {
                // Major structural changes (regular class -> generic class, inheritance changes)
                SemanticSimilarity::structural_refactor(0.8)
            } else if fields_changed || annotations_changed {
                // Minor structural changes (field count, annotations)
                SemanticSimilarity::structural_refactor(0.9)
            } else {
                // Same structure, check for body changes
                SemanticSimilarity::body_change()
            }
        } else {
            SemanticSimilarity::unrelated()
        }
    }

    /// Java-specific comparison for callables (methods, constructors)
    fn compare_java_callables(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        if let (
            SemanticUnitType::Callable {
                is_generic: old_generic,
                parameter_count: old_params,
                return_type: old_return,
                ..
            },
            SemanticUnitType::Callable {
                is_generic: new_generic,
                parameter_count: new_params,
                return_type: new_return,
                ..
            },
        ) = (&old.unit_type, &new.unit_type)
        {
            // Check for signature changes
            let generics_changed = old_generic != new_generic;
            let params_changed = old_params != new_params;
            let return_changed = old_return != new_return;
            let annotations_changed = self.compare_annotations(old, new, old_source, new_source);

            if generics_changed || params_changed || return_changed {
                // Signature changes
                SemanticSimilarity::signature_change(true, 0.7)
            } else if annotations_changed {
                // Same signature, different annotations (e.g., added @Override)
                SemanticSimilarity::structural_refactor(0.95)
            } else {
                // Same signature, check for actual body changes
                let body_changed = self.check_method_body_changed(old, new, old_source, new_source);
                if body_changed {
                    SemanticSimilarity::body_change()
                } else {
                    SemanticSimilarity::identical()
                }
            }
        } else {
            SemanticSimilarity::unrelated()
        }
    }

    /// Check if method body has actually changed by comparing the method body nodes
    fn check_method_body_changed(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        // Find method body nodes in both old and new method declarations
        let old_body = self.find_method_body(old.tree_sitter_node);
        let new_body = self.find_method_body(new.tree_sitter_node);

        match (old_body, new_body) {
            (Some(old_body_node), Some(new_body_node)) => {
                // Compare body content by extracting text and normalizing whitespace
                let old_body_text = old_source.node_text(&old_body_node).unwrap_or("");
                let new_body_text = new_source.node_text(&new_body_node).unwrap_or("");

                // Normalize whitespace for comparison
                let old_normalized = old_body_text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                let new_normalized = new_body_text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                old_normalized != new_normalized
            }
            (None, None) => false, // Both have no body (interface methods) - no change
            _ => true,             // One has body, other doesn't - definite change
        }
    }

    /// Find the method body node (block) within a method declaration
    fn find_method_body<'a>(
        &self,
        method_node: tree_sitter::Node<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut cursor = method_node.walk();
        method_node
            .children(&mut cursor)
            .find(|&child| child.kind() == "block")
    }

    /// Compare annotations between two semantic units
    fn compare_annotations(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> bool {
        // If different number of annotations, they're different
        if old.metadata_nodes.len() != new.metadata_nodes.len() {
            return true;
        }

        // Compare each annotation's text content
        for (old_meta, new_meta) in old.metadata_nodes.iter().zip(new.metadata_nodes.iter()) {
            let old_text = old_source.node_text(&old_meta.node).unwrap_or_default();
            let new_text = new_source.node_text(&new_meta.node).unwrap_or_default();
            if old_text != new_text {
                return true;
            }
        }

        false
    }

    /// Calculate similarity between two names, handling Java-specific transformations
    fn calculate_name_similarity(&self, name1: &str, name2: &str) -> SemanticSimilarity {
        if name1 == name2 {
            return SemanticSimilarity::identical();
        }

        // Handle generic instantiation vs base name matching
        // e.g., "List" should match "List<String>"
        let base1 = name1.split('<').next().unwrap_or(name1);
        let base2 = name2.split('<').next().unwrap_or(name2);

        if base1 == base2 {
            SemanticSimilarity::structural_refactor(0.8) // Good match for base name
        } else if base1.contains(base2) || base2.contains(base1) {
            SemanticSimilarity::name_refactor(0.6) // Partial match
        } else {
            SemanticSimilarity::unrelated()
        }
    }
}

impl Default for JavaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for JavaParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_java::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set Java language: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse Java code".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_java::language()
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            "method_declaration" | "constructor_declaration" => SemanticNodeKind::Function,
            "class_declaration" => SemanticNodeKind::Class,
            "interface_declaration" => SemanticNodeKind::Interface,
            "enum_declaration" => SemanticNodeKind::Enum,
            "import_declaration" => SemanticNodeKind::Import,
            "variable_declaration" | "field_declaration" => SemanticNodeKind::Variable,
            "expression_statement" | "assignment_expression" => SemanticNodeKind::Statement,
            "method_invocation" | "field_access" | "binary_expression" => {
                SemanticNodeKind::Expression
            }
            "comment" | "line_comment" | "block_comment" => SemanticNodeKind::Comment,
            "program" => SemanticNodeKind::SourceFile,
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
        Ok(SemanticTree::new(semantic_root, ProgrammingLanguage::Java))
    }

    fn compare_semantic_units(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        // Enhanced semantic matching for Java constructs
        use SemanticUnitType::*;

        match (&old.unit_type, &new.unit_type) {
            // Class-to-class matching (including generic transformations)
            (DataStructure { .. }, DataStructure { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));

                if old_name == new_name {
                    self.compare_java_data_structures(old, new, old_source, new_source)
                } else if let (Some(old), Some(new)) = (&old_name, &new_name) {
                    self.calculate_name_similarity(old, new)
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Method-to-method matching
            (Callable { .. }, Callable { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));

                if old_name == new_name {
                    self.compare_java_callables(old, new, old_source, new_source)
                } else if let (Some(old), Some(new)) = (&old_name, &new_name) {
                    self.calculate_name_similarity(old, new)
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Import-to-import matching
            (
                Import {
                    source_module: old_module,
                    ..
                },
                Import {
                    source_module: new_module,
                    ..
                },
            ) => {
                if old_module == new_module {
                    SemanticSimilarity::identical()
                } else {
                    // Check for package refactoring scenarios
                    let old_parts: Vec<&str> = old_module.split('.').collect();
                    let new_parts: Vec<&str> = new_module.split('.').collect();

                    if let (Some(old_class), Some(new_class)) = (old_parts.last(), new_parts.last())
                    {
                        if old_class == new_class {
                            SemanticSimilarity::structural_refactor(0.8) // Same class, different package
                        } else {
                            SemanticSimilarity::unrelated()
                        }
                    } else {
                        SemanticSimilarity::unrelated()
                    }
                }
            }

            // Cross-type matching for structural transformations
            _ => SemanticSimilarity::unrelated(),
        }
    }
}
