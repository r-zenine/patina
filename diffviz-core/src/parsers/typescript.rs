//! TypeScript language parser implementation

use crate::common::{ASTError, LanguageParser, Result, SemanticNodeKind};
use crate::semantic_ast::{
    MetadataNode, MetadataPosition, SemanticError, SemanticNode, SemanticTree, SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Parser, Tree};

pub struct TypeScriptParser;

impl TypeScriptParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for TypeScriptParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_typescript::language_typescript())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set TypeScript language: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse TypeScript code".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_typescript::language_typescript()
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            "function_declaration" | "method_definition" | "arrow_function" => {
                SemanticNodeKind::Function
            }
            "class_declaration" => SemanticNodeKind::Class,
            "interface_declaration" => SemanticNodeKind::Interface,
            "enum_declaration" => SemanticNodeKind::Enum,
            "import_statement" | "export_statement" => SemanticNodeKind::Import,
            "variable_declaration" | "lexical_declaration" => SemanticNodeKind::Variable,
            "expression_statement" | "assignment_expression" => SemanticNodeKind::Statement,
            "call_expression" | "member_expression" | "binary_expression" => {
                SemanticNodeKind::Expression
            }
            "type_alias_declaration" => SemanticNodeKind::TypeDefinition,
            // Signature components
            "formal_parameters" | "rest_parameter" | "type_parameters" | "type_parameter"
            | "type_annotation" | "return_type" => SemanticNodeKind::SignatureComponent,
            "comment" => SemanticNodeKind::Comment,
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

        Ok(SemanticTree::new(
            semantic_root,
            crate::common::ProgrammingLanguage::TypeScript,
        ))
    }
}

impl TypeScriptParser {
    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ";" | ":" | "." |
            "?" | "!" | "#" | "@" | "$" | "%" | "^" | "&" | "*" | "-" | 
            "=" | "+" | "|" | "\\" | "/" | "<" | ">" | "=>" |

            // Keywords that are part of larger constructs
            "function" | "class" | "interface" | "enum" | "type" | "import" | 
            "export" | "const" | "let" | "var" | "if" | "else" | "for" | "while" |
            "do" | "switch" | "case" | "default" | "try" | "catch" | "finally" |
            "return" | "yield" | "break" | "continue" | "async" | "await" |
            "public" | "private" | "protected" | "static" | "abstract" | "readonly" |

            // Literals and identifiers
            "string" | "number" | "boolean" | "null" | "undefined" | "identifier" |
            "property_identifier" | "type_identifier" |

            // Comments and whitespace
            "comment" | "line_comment" | "block_comment" |

            // Error nodes
            "ERROR" | "MISSING"
        )
    }

    /// Find preceding decorator siblings for a given node
    fn find_preceding_decorators<'a>(
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

                if sibling.kind() == "decorator" {
                    metadata_nodes.push(MetadataNode {
                        node: sibling,
                        position: MetadataPosition::PrecedingSibling(position),
                    });
                    position -= 1;
                } else if !self.is_trivial_syntax_token(sibling.kind()) {
                    // Stop at the first non-decorator, non-trivial node
                    break;
                }
            }
        }

        // Reverse to maintain proper order (closest to furthest)
        metadata_nodes.reverse();
        metadata_nodes
    }

    /// Build semantic node for TypeScript AST node
    fn build_semantic_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "program" => self.build_program_node(node, source),
            "function_declaration" => {
                self.build_function_node(node, source, parent, parent_context)
            }
            "method_definition" => self.build_method_node(node, source, parent, parent_context),
            "arrow_function" => {
                self.build_arrow_function_node(node, source, parent, parent_context)
            }
            "class_declaration" => self.build_class_node(node, source, parent),
            "interface_declaration" => self.build_interface_node(node, source, parent),
            "enum_declaration" => self.build_enum_node(node, source, parent),
            "type_alias_declaration" => self.build_type_alias_node(node, source, parent),
            "import_statement" | "export_statement" => self.build_import_node(node, source),
            "variable_declaration" | "lexical_declaration" => {
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

    /// Build semantic node for TypeScript program (root)
    fn build_program_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let mut children = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match self.build_semantic_node(child, source, Some(node), None) {
                Ok(child_node) => children.push(child_node),
                Err(_) => continue,
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

    /// Build semantic node for TypeScript function declaration
    fn build_function_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");
        let return_type_node = node.child_by_field_name("return_type");

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            self.count_parameters(params)
        } else {
            0
        };

        // Extract return type annotation
        let return_type = return_type_node.map(|return_type| {
            return_type
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string()
        });

        // Check if async
        let is_async = self.is_async_function(node, source);

        // Determine if this is a method or standalone function
        let is_method = parent_context == Some("class");

        // Determine visibility from modifiers
        let visibility = if is_method {
            self.get_method_visibility(node, source)
        } else {
            "public".to_string()
        };

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic: self.has_type_parameters(node),
                parameter_count,
                return_type,
                is_async,
                visibility,
                is_method,
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for TypeScript method
    fn build_method_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        _parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");
        let return_type_node = node.child_by_field_name("return_type");

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            self.count_parameters(params)
        } else {
            0
        };

        // Extract return type annotation
        let return_type = return_type_node.map(|return_type| {
            return_type
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string()
        });

        // Determine visibility from modifiers
        let visibility = self.get_method_visibility(node, source);
        let is_async = self.is_async_function(node, source);

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::Callable {
                is_generic: self.has_type_parameters(node),
                parameter_count,
                return_type,
                is_async,
                visibility,
                is_method: true,
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for TypeScript arrow function
    fn build_arrow_function_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let parameters_node = node.child_by_field_name("parameters");
        let body_node = node.child_by_field_name("body");
        let return_type_node = node.child_by_field_name("return_type");

        // Count parameters
        let parameter_count = if let Some(params) = parameters_node {
            self.count_parameters(params)
        } else {
            0
        };

        // Extract return type annotation
        let return_type = return_type_node.map(|return_type| {
            return_type
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string()
        });

        let is_async = self.is_async_function(node, source);

        // Check if this is a method inside a class (arrow functions can be class properties)
        let is_method = parent_context == Some("class");

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            None, // Arrow functions might not have explicit names
            SemanticUnitType::Callable {
                is_generic: self.has_type_parameters(node),
                parameter_count,
                return_type,
                is_async,
                visibility: "public".to_string(),
                is_method,
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for TypeScript class
    fn build_class_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let superclass_node = node.child_by_field_name("superclass");
        let body_node = node.child_by_field_name("body");

        // Extract inheritance
        let inheritance = if let Some(superclass) = superclass_node {
            vec![
                superclass
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .to_string(),
            ]
        } else {
            vec![]
        };

        // Count class members
        let field_count = body_node.map(|body| self.count_class_members(body));

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        // Build children with class context
        let mut children = Vec::new();
        if let Some(body) = body_node {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if self.is_trivial_syntax_token(child.kind()) || child.kind() == "decorator" {
                    continue;
                }

                match self.build_semantic_node(child, source, Some(body), Some("class")) {
                    Ok(child_node) => children.push(child_node),
                    Err(_) => continue,
                }
            }
        }

        let mut class_node = SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: self.has_type_parameters(node),
                field_count,
                inheritance,
                visibility: "public".to_string(),
                signature_node: superclass_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        );
        class_node.children = children;

        Ok(class_node)
    }

    /// Build semantic node for TypeScript interface
    fn build_interface_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = node.child_by_field_name("name");
        let body_node = node.child_by_field_name("body");

        // Count interface members
        let field_count = body_node.map(|body| self.count_interface_members(body));

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: self.has_type_parameters(node),
                field_count,
                inheritance: vec![], // TODO: Extract extends clauses
                visibility: "public".to_string(),
                signature_node: body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for TypeScript enum
    fn build_enum_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = node.child_by_field_name("name");
        let body_node = node.child_by_field_name("body");

        // Count enum members
        let field_count = body_node.map(|body| self.count_enum_members(body));

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: false, // Enums can't be generic in TypeScript
                field_count,
                inheritance: vec![],
                visibility: "public".to_string(),
                signature_node: body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for TypeScript type alias
    fn build_type_alias_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        let name_node = node.child_by_field_name("name");
        let type_node = node.child_by_field_name("value");

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            name_node,
            SemanticUnitType::DataStructure {
                is_generic: self.has_type_parameters(node),
                field_count: None,
                inheritance: vec![],
                visibility: "public".to_string(),
                signature_node: type_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for TypeScript import/export
    fn build_import_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        // Extract import information (simplified)
        let (module_path, alias, is_wildcard) = self.extract_import_info(node, source);

        Ok(SemanticNode::new(
            node,
            None,
            SemanticUnitType::Import {
                import_type: if is_wildcard {
                    crate::semantic_ast::ImportType::Wildcard
                } else if alias.is_some() {
                    crate::semantic_ast::ImportType::Namespace
                } else {
                    crate::semantic_ast::ImportType::Specific
                },
                source_module: module_path,
                imported_items: if let Some(alias_name) = alias {
                    vec![alias_name]
                } else {
                    vec![]
                },
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        ))
    }

    /// Build semantic node for TypeScript variable declaration
    fn build_variable_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        use crate::semantic_ast::{SemanticNode, SemanticUnitType};
        use std::collections::HashMap;

        // Determine if it's const, let, or var
        let is_mutable = !self.is_const_declaration(node);

        // Find metadata nodes (decorators) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_decorators(node, parent_node)
        } else {
            Vec::new()
        };

        Ok(SemanticNode::new(
            node,
            None,
            SemanticUnitType::Variable {
                is_const: !is_mutable,
                is_static: false,      // TODO: Detect scope
                type_annotation: None, // TODO: Extract type annotations
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
            if matches!(
                child.kind(),
                "required_parameter" | "optional_parameter" | "rest_parameter"
            ) {
                count += 1;
            }
        }
        count
    }

    fn count_class_members(&self, body_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if matches!(
                child.kind(),
                "method_definition" | "property_definition" | "constructor"
            ) {
                count += 1;
            }
        }
        count
    }

    fn count_interface_members(&self, body_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if matches!(child.kind(), "property_signature" | "method_signature") {
                count += 1;
            }
        }
        count
    }

    fn count_enum_members(&self, body_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "property_identifier" {
                count += 1;
            }
        }
        count
    }

    fn has_type_parameters(&self, node: tree_sitter::Node) -> bool {
        node.child_by_field_name("type_parameters").is_some()
    }

    fn is_async_function(&self, node: tree_sitter::Node, source: &str) -> bool {
        // Check for async keyword before the function
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        text.trim_start().starts_with("async ")
    }

    fn get_method_visibility(&self, node: tree_sitter::Node, source: &str) -> String {
        // Look for public/private/protected modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "accessibility_modifier" {
                if let Ok(modifier) = child.utf8_text(source.as_bytes()) {
                    return modifier.to_string();
                }
            }
        }
        "public".to_string() // Default visibility
    }

    fn is_const_declaration(&self, node: tree_sitter::Node) -> bool {
        // Check if this is a const declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const" {
                return true;
            }
        }
        false
    }

    fn extract_import_info(
        &self,
        node: tree_sitter::Node,
        source: &str,
    ) -> (String, Option<String>, bool) {
        // Simplified import extraction
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");

        // Check for wildcard import
        let is_wildcard = text.contains("* from") || text.contains("* as");

        // Extract module path (simplified)
        let module_path = if let Some(start) = text.find("from \"") {
            let start = start + 6; // Skip 'from "'
            if let Some(end) = text[start..].find('"') {
                text[start..start + end].to_string()
            } else {
                "".to_string()
            }
        } else if let Some(start) = text.find("from '") {
            let start = start + 6; // Skip "from '"
            if let Some(end) = text[start..].find('\'') {
                text[start..start + end].to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // Extract alias (simplified)
        let alias = if text.contains(" as ") {
            // Find the alias after " as "
            if let Some(start) = text.find(" as ") {
                let start = start + 4; // Skip " as "
                text[start..]
                    .split_whitespace()
                    .next()
                    .map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        (module_path, alias, is_wildcard)
    }
}
