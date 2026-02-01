//! Python language parser implementation
//!
//! This module provides Python-specific parsing using TreeSitter and implements
//! the simplified LanguageParser trait for AST analysis.

use crate::common::{ASTError, LanguageParser, Result, SemanticNodeKind};
use crate::semantic_ast::{
    ImportType, MetadataNode, MetadataPosition, SemanticError, SemanticNode, SemanticTree,
    SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Parser, Tree};

/// Python language parser using TreeSitter
pub struct PythonParser;

impl PythonParser {
    /// Create a new PythonParser
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for PythonParser {
    /// Parse Python content into TreeSitter AST
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_python::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set Python language: {e}"),
            })?;

        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse Python code".to_string(),
            })
    }

    /// Get the TreeSitter language for Python
    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_python::language()
    }

    /// Classify TreeSitter node kinds into semantic categories for Python
    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            // Core language constructs
            "function_definition" => SemanticNodeKind::Function,
            "class_definition" => SemanticNodeKind::Class,

            // Module and imports
            "module" => SemanticNodeKind::Module,
            "import_statement" | "import_from_statement" => SemanticNodeKind::Import,

            // Variables and assignments
            "assignment" | "augmented_assignment" => SemanticNodeKind::Variable,

            // Statements and expressions
            "expression_statement"
            | "assert_statement"
            | "return_statement"
            | "pass_statement"
            | "break_statement"
            | "continue_statement" => SemanticNodeKind::Statement,

            "call"
            | "attribute"
            | "subscript"
            | "binary_operator"
            | "unary_operator"
            | "comparison_operator" => SemanticNodeKind::Expression,

            // Signature components
            "parameters" | "parameter" | "default_parameter" | "typed_parameter" => {
                SemanticNodeKind::SignatureComponent
            }

            // Comments
            "comment" => SemanticNodeKind::Comment,

            // Everything else
            _ => SemanticNodeKind::Other(node_kind.to_string()),
        }
    }

    /// Get Python-specific context boundaries for different change types
    fn get_context_boundaries(
        &self,
        change_type: &crate::ast_diff::ASTChangeType,
        change_node_kind: &SemanticNodeKind,
    ) -> Vec<SemanticNodeKind> {
        match (change_type, change_node_kind) {
            // Function changes should look for enclosing classes or modules
            (_, SemanticNodeKind::Function) => vec![
                SemanticNodeKind::Class,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],

            // Class changes should look for enclosing modules
            (_, SemanticNodeKind::Class) => {
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

            // Variable/Statement/Expression changes should look for enclosing function or class
            (
                _,
                SemanticNodeKind::Variable
                | SemanticNodeKind::Statement
                | SemanticNodeKind::Expression,
            ) => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::Class,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],

            // For other types, provide general boundaries
            _ => vec![
                SemanticNodeKind::Function,
                SemanticNodeKind::Class,
                SemanticNodeKind::Module,
                SemanticNodeKind::SourceFile,
            ],
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
            crate::common::ProgrammingLanguage::Python,
        ))
    }
}

impl PythonParser {
    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ";" | ":" | "." |
            "?" | "!" | "#" | "@" | "$" | "%" | "^" | "&" | "*" | "-" | 
            "=" | "+" | "|" | "\\" | "/" | "<" | ">" |

            // Keywords that are part of larger constructs
            "def" | "class" | "import" | "from" | "if" | "else" | "elif" |
            "for" | "while" | "with" | "try" | "except" | "finally" |
            "return" | "yield" | "pass" | "break" | "continue" | "lambda" |
            "and" | "or" | "not" | "in" | "is" | "async" | "await" |

            // Literals and identifiers
            "string" | "integer" | "float" | "boolean" | "none" | "identifier" |

            // Comments and whitespace  
            "comment" | "line_comment" |

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

    /// Build semantic node for Python AST node
    fn build_semantic_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "module" => self.build_module_node(node, source),
            "function_definition" => self.build_function_node(node, source, parent, parent_context),
            "async_function_definition" => {
                self.build_async_function_node(node, source, parent, parent_context)
            }
            "class_definition" => self.build_class_node(node, source, parent),
            "import_statement" | "import_from_statement" => self.build_import_node(node, source),
            "assignment" | "augmented_assignment" => self.build_variable_node(node, source, parent),
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

    /// Build semantic node for Python module
    fn build_module_node<'a>(
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
                module_type: crate::semantic_ast::ModuleType::Module,
                is_public: true,
                metadata: HashMap::new(),
            },
            Vec::new(), // metadata_nodes
        );
        root_node.children = children;

        Ok(root_node)
    }

    /// Build semantic node for Python function
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

        // Determine visibility and method status
        let visibility = if let Some(name) = name_node {
            let name_text = name.utf8_text(source.as_bytes()).unwrap_or("");
            if name_text.starts_with("__") && name_text.ends_with("__") {
                "magic".to_string()
            } else if name_text.starts_with("_") {
                "private".to_string()
            } else {
                "public".to_string()
            }
        } else {
            "public".to_string()
        };

        // Check if this is a method inside a class
        let is_method = parent_context == Some("class");

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
                is_generic: false, // Python doesn't have generics in the same way
                parameter_count,
                return_type,
                is_async: false,
                visibility,
                is_method,
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for Python async function
    fn build_async_function_node<'a>(
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

        // Determine visibility and method status
        let visibility = if let Some(name) = name_node {
            let name_text = name.utf8_text(source.as_bytes()).unwrap_or("");
            if name_text.starts_with("__") && name_text.ends_with("__") {
                "magic".to_string()
            } else if name_text.starts_with("_") {
                "private".to_string()
            } else {
                "public".to_string()
            }
        } else {
            "public".to_string()
        };

        // Check if this is a method inside a class
        let is_method = parent_context == Some("class");

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
                is_generic: false,
                parameter_count,
                return_type,
                is_async: true, // This is async
                visibility,
                is_method,
                signature_node: parameters_node,
                body_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Build semantic node for Python class
    fn build_class_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name_node = node.child_by_field_name("name");
        let superclasses_node = node.child_by_field_name("superclasses");
        let body_node = node.child_by_field_name("body");

        // Extract inheritance
        let inheritance = if let Some(superclasses) = superclasses_node {
            self.extract_superclasses(superclasses, source)
        } else {
            vec![]
        };

        // Count class methods/attributes
        let field_count = body_node.map(|body| self.count_class_members(body));

        // Determine visibility
        let visibility = if let Some(name) = name_node {
            let name_text = name.utf8_text(source.as_bytes()).unwrap_or("");
            if name_text.starts_with("_") {
                "private".to_string()
            } else {
                "public".to_string()
            }
        } else {
            "public".to_string()
        };

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
                is_generic: false,
                field_count,
                inheritance,
                visibility,
                signature_node: superclasses_node,
                metadata: HashMap::new(),
            },
            metadata_nodes,
        );
        class_node.children = children;

        Ok(class_node)
    }

    /// Build semantic node for Python import
    fn build_import_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let (import_type, source_module, imported_items) =
            self.parse_python_import(node, source)?;

        Ok(SemanticNode::new(
            node,
            None,
            SemanticUnitType::Import {
                import_type,
                source_module,
                imported_items,
                metadata: HashMap::new(),
            },
            Vec::new(), // No decorators on imports
        ))
    }

    /// Build semantic node for Python variable/assignment
    fn build_variable_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        _source: &str,
        parent: Option<tree_sitter::Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
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
                is_const: false,       // Python variables are not const by default
                is_static: false,      // TODO: Detect scope
                type_annotation: None, // TODO: Extract type annotations
                visibility: "public".to_string(),
                metadata: HashMap::new(),
            },
            metadata_nodes,
        ))
    }

    /// Parse Python import statement with better handling
    fn parse_python_import(
        &self,
        node: tree_sitter::Node<'_>,
        source: &str,
    ) -> std::result::Result<(ImportType, String, Vec<String>), SemanticError> {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");

        if text.contains("import *") {
            // from module import *
            let module = if let Some(_start) = text.find("from ") {
                text.split_whitespace().nth(1).unwrap_or("").to_string()
            } else {
                "".to_string()
            };
            Ok((ImportType::Wildcard, module, Vec::new()))
        } else if text.starts_with("from ") {
            // from module import item1, item2
            let parts: Vec<&str> = text.split(" import ").collect();
            let module = parts[0].trim_start_matches("from ").trim().to_string();
            let items: Vec<String> = if parts.len() > 1 {
                parts[1]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                Vec::new()
            };
            Ok((ImportType::Specific, module, items))
        } else if text.starts_with("import ") {
            // import module1, module2
            let items: Vec<String> = text
                .trim_start_matches("import ")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok((ImportType::Default, "".to_string(), items))
        } else {
            Err(SemanticError::TreeBuildError(
                "Failed to parse Python import statement".to_string(),
            ))
        }
    }

    // Helper methods

    fn count_parameters(&self, params_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "identifier"
                || child.kind() == "typed_parameter"
                || child.kind() == "default_parameter"
            {
                count += 1;
            }
        }
        count
    }

    fn extract_superclasses(
        &self,
        superclasses_node: tree_sitter::Node,
        source: &str,
    ) -> Vec<String> {
        let mut inheritance = Vec::new();
        let mut cursor = superclasses_node.walk();
        for child in superclasses_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(name) = child.utf8_text(source.as_bytes()) {
                    inheritance.push(name.to_string());
                }
            }
        }
        inheritance
    }

    fn count_class_members(&self, body_node: tree_sitter::Node) -> usize {
        let mut count = 0;
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if matches!(
                child.kind(),
                "function_definition" | "async_function_definition" | "assignment"
            ) {
                count += 1;
            }
        }
        count
    }

    #[allow(dead_code)]
    fn extract_import_info(
        &self,
        node: tree_sitter::Node,
        source: &str,
    ) -> (String, Option<String>, bool) {
        // Simple extraction - could be improved
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");

        // Check for wildcard import
        let is_wildcard = text.contains("import *");

        // Extract module path (simplified)
        let module_path = if text.starts_with("from ") {
            // from module import something
            text.split_whitespace().nth(1).unwrap_or("").to_string()
        } else if text.starts_with("import ") {
            // import module
            text.split_whitespace().nth(1).unwrap_or("").to_string()
        } else {
            "".to_string()
        };

        // Extract alias (simplified)
        let alias = if text.contains(" as ") {
            text.split(" as ").nth(1).map(|s| s.trim().to_string())
        } else {
            None
        };

        (module_path, alias, is_wildcard)
    }
}
