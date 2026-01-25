//! C language parser implementation

use crate::common::{ASTError, LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::semantic_ast::{
    ImportType, ModuleType, SemanticError, SemanticNode, SemanticSimilarity, SemanticTree,
    SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Tree};

pub struct CParser;

impl CParser {
    pub fn new() -> Self {
        Self
    }

    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators (excluding semicolon which we handle explicitly)
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ":" | "::" |
            "." | "->" | "?" | "!" | "#" | "@" | "$" | "%" |
            "^" | "&" | "*" | "-" | "=" | "+" | "|" | "\\" | "/" | "<" | ">" |

            // Keywords that are part of larger constructs
            "if" | "else" | "for" | "while" | "do" | "switch" | "case" | "default" |
            "break" | "continue" | "return" | "goto" | "sizeof" | "typedef" |
            "struct" | "union" | "enum" | "static" | "extern" | "auto" | "register" |
            "const" | "volatile" | "inline" | "restrict" |

            // Type specifiers
            "void" | "char" | "short" | "int" | "long" | "float" | "double" |
            "signed" | "unsigned" |

            // Literals and identifiers
            "string_literal" | "char_literal" | "number_literal" |
            "identifier" | "field_identifier" | "type_identifier" |

            // Comments and whitespace  
            "comment" | "preproc_directive" |

            // Granular statement-level constructs that should not be semantic nodes
            "expression_statement" | "assignment_expression" | "binary_expression" |
            "call_expression" | "field_expression" | "parenthesized_expression" |
            "unary_expression" | "conditional_expression" | "compound_statement" |
            "if_statement" | "return_statement" | "init_declarator" |
            "field_declaration" | "parameter_declaration" | "pointer_declarator" |
            "function_declarator" | "field_declaration_list" | "parameter_list" |
            "enumerator_list" | "enumerator" | "system_lib_string" |

            // Error nodes
            "ERROR" | "MISSING"
        )
    }

    /// Build semantic node for C AST node with context awareness
    fn build_semantic_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "translation_unit" => self.build_translation_unit_node(node, source),
            "function_definition" => self.build_function_node(node, source, parent, parent_context),
            "struct_specifier" => self.build_struct_node(node, source, parent),
            "enum_specifier" => self.build_enum_node(node, source, parent),
            "preproc_include" | "preproc_def" | "preproc_ifdef" | "preproc_if" => {
                self.build_preprocessor_node(node, source, parent)
            }
            "declaration" => {
                // Skip simple variable declarations as standalone semantic nodes
                // Only complex declarations should become semantic nodes
                Err(SemanticError::TreeBuildError(
                    "Simple declarations should not be standalone semantic nodes".to_string(),
                ))
            }
            "typedef_declaration" => self.build_typedef_node(node, source, parent),
            // Handle semicolon-terminated declarations properly
            ";" => {
                // Skip standalone semicolons - they should be handled by their parent declarations
                Err(SemanticError::TreeBuildError(
                    "Standalone semicolon skipped".to_string(),
                ))
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

    /// Build translation unit (root) node
    fn build_translation_unit_node<'a>(
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

    /// Build function definition node
    fn build_function_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
        _parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let (name, return_type, params, storage_class) = self.extract_function_info(node, source);

        // Build children from function body
        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "compound_statement" {
                // Process function body content
                let mut body_cursor = child.walk();
                for body_child in child.children(&mut body_cursor) {
                    if !self.is_trivial_syntax_token(body_child.kind()) {
                        match self.build_semantic_node(
                            body_child,
                            source,
                            Some(child),
                            Some("function_body"),
                        ) {
                            Ok(child_node) => children.push(child_node),
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        let mut metadata = HashMap::new();
        if let Some(ref ret_type) = return_type {
            metadata.insert("return_type".to_string(), ret_type.clone());
        }
        if let Some(ref storage) = storage_class {
            metadata.insert("storage_class".to_string(), storage.clone());
        }
        metadata.insert("parameter_count".to_string(), params.to_string());

        let mut semantic_node = SemanticNode::new(
            node,
            name.and_then(|_n| {
                // Find the identifier node for the function name
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "function_declarator" {
                        let mut decl_cursor = child.walk();
                        for decl_child in child.children(&mut decl_cursor) {
                            if decl_child.kind() == "identifier" {
                                return Some(decl_child);
                            }
                        }
                    }
                }
                None
            }),
            SemanticUnitType::Callable {
                is_generic: false,
                parameter_count: params,
                return_type,
                is_async: false,
                visibility: storage_class.unwrap_or_else(|| "local".to_string()),
                is_method: false,
                signature_node: None,
                body_node: None,
                metadata,
            },
            Vec::new(),
        );
        semantic_node.children = children;
        Ok(semantic_node)
    }

    /// Build struct specifier node
    fn build_struct_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name = self.extract_struct_name(node, source);
        let field_count = self.count_struct_fields(node);

        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut field_cursor = child.walk();
                for field in child.children(&mut field_cursor) {
                    if !self.is_trivial_syntax_token(field.kind()) {
                        match self.build_semantic_node(field, source, Some(child), Some("struct")) {
                            Ok(field_node) => children.push(field_node),
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("field_count".to_string(), field_count.to_string());
        metadata.insert("type".to_string(), "struct".to_string());

        let mut semantic_node = SemanticNode::new(
            node,
            name.and_then(|_| {
                // Find the type_identifier node for the struct name
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .find(|&child| child.kind() == "type_identifier")
            }),
            SemanticUnitType::DataStructure {
                is_generic: false,
                field_count: Some(field_count),
                inheritance: Vec::new(),
                visibility: "public".to_string(),
                signature_node: None,
                metadata,
            },
            Vec::new(),
        );
        semantic_node.children = children;
        Ok(semantic_node)
    }

    /// Build enum specifier node
    fn build_enum_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name = self.extract_enum_name(node, source);
        let variant_count = self.count_enum_variants(node);

        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "enumerator_list" {
                let mut enum_cursor = child.walk();
                for variant in child.children(&mut enum_cursor) {
                    if !self.is_trivial_syntax_token(variant.kind()) {
                        match self.build_semantic_node(variant, source, Some(child), Some("enum")) {
                            Ok(variant_node) => children.push(variant_node),
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("variant_count".to_string(), variant_count.to_string());
        metadata.insert("type".to_string(), "enum".to_string());

        let mut semantic_node = SemanticNode::new(
            node,
            name.and_then(|_| {
                // Find the type_identifier node for the enum name
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .find(|&child| child.kind() == "type_identifier")
            }),
            SemanticUnitType::DataStructure {
                is_generic: false,
                field_count: Some(variant_count),
                inheritance: Vec::new(),
                visibility: "public".to_string(),
                signature_node: None,
                metadata,
            },
            Vec::new(),
        );
        semantic_node.children = children;
        Ok(semantic_node)
    }

    /// Build preprocessor directive node
    #[allow(dead_code)]
    fn build_preprocessor_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let directive_text = source[node.start_byte()..node.end_byte()].trim();
        let (import_type, name) = if directive_text.starts_with("#include") {
            if directive_text.contains('<') {
                (
                    ImportType::Specific,
                    self.extract_include_path(directive_text),
                )
            } else {
                (
                    ImportType::Default,
                    self.extract_include_path(directive_text),
                )
            }
        } else {
            (ImportType::Namespace, directive_text.to_string())
        };

        let mut metadata = HashMap::new();
        metadata.insert("directive".to_string(), directive_text.to_string());

        Ok(SemanticNode::new(
            node,
            None, // Preprocessor directives don't have a specific name node
            SemanticUnitType::Import {
                import_type,
                source_module: name,
                imported_items: Vec::new(),
                metadata,
            },
            Vec::new(),
        ))
    }

    /// Build variable declaration node
    #[allow(dead_code)]
    fn build_variable_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let (name, var_type, storage_class) = self.extract_variable_info(node, source);

        let mut metadata = HashMap::new();
        if let Some(ref v_type) = var_type {
            metadata.insert("type".to_string(), v_type.clone());
        }
        if let Some(ref storage) = storage_class {
            metadata.insert("storage_class".to_string(), storage.clone());
        }

        Ok(SemanticNode::new(
            node,
            name.and_then(|_| {
                // Find the identifier node for the variable name
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "init_declarator" {
                        let mut init_cursor = child.walk();
                        for init_child in child.children(&mut init_cursor) {
                            if init_child.kind() == "identifier" {
                                return Some(init_child);
                            }
                        }
                    }
                }
                None
            }),
            SemanticUnitType::Variable {
                is_const: false,
                is_static: storage_class.as_ref().is_some_and(|s| s == "static"),
                type_annotation: var_type,
                visibility: storage_class.unwrap_or_else(|| "local".to_string()),
                metadata,
            },
            Vec::new(),
        ))
    }

    /// Build typedef declaration node
    fn build_typedef_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let alias_name = self.extract_typedef_name(node, source);

        let mut metadata = HashMap::new();
        metadata.insert("kind".to_string(), "typedef".to_string());

        let alias_name_clone = alias_name.clone();
        Ok(SemanticNode::new(
            node,
            alias_name.and_then(|_| {
                // Find the type_identifier node for the typedef name
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .find(|&child| child.kind() == "type_identifier")
            }),
            SemanticUnitType::Variable {
                is_const: true,
                is_static: false,
                type_annotation: alias_name_clone,
                visibility: "global".to_string(),
                metadata,
            },
            Vec::new(),
        ))
    }

    /// Extract function information from function_definition node
    fn extract_function_info(
        &self,
        node: Node,
        source: &str,
    ) -> (Option<String>, Option<String>, usize, Option<String>) {
        let mut name = None;
        let mut return_type = None;
        let mut param_count = 0;
        let mut storage_class = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declarator" => {
                    let mut decl_cursor = child.walk();
                    for decl_child in child.children(&mut decl_cursor) {
                        if decl_child.kind() == "identifier" {
                            name = Some(
                                source[decl_child.start_byte()..decl_child.end_byte()].to_string(),
                            );
                        } else if decl_child.kind() == "parameter_list" {
                            param_count = self.count_parameters(decl_child);
                        }
                    }
                }
                "primitive_type" | "type_identifier" => {
                    return_type = Some(source[child.start_byte()..child.end_byte()].to_string());
                }
                "storage_class_specifier" => {
                    storage_class = Some(source[child.start_byte()..child.end_byte()].to_string());
                }
                _ => {}
            }
        }

        (name, return_type, param_count, storage_class)
    }

    /// Count parameters in a parameter_list node
    fn count_parameters(&self, node: Node) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                count += 1;
            }
        }
        count
    }

    /// Extract struct name from struct_specifier node
    fn extract_struct_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Some(source[child.start_byte()..child.end_byte()].to_string());
            }
        }
        None
    }

    /// Count fields in a struct
    fn count_struct_fields(&self, node: Node) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut field_cursor = child.walk();
                for field in child.children(&mut field_cursor) {
                    if field.kind() == "field_declaration" {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Extract enum name from enum_specifier node
    fn extract_enum_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Some(source[child.start_byte()..child.end_byte()].to_string());
            }
        }
        None
    }

    /// Count enum variants
    fn count_enum_variants(&self, node: Node) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "enumerator_list" {
                let mut enum_cursor = child.walk();
                for variant in child.children(&mut enum_cursor) {
                    if variant.kind() == "enumerator" {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Extract include path from preprocessor directive
    #[allow(dead_code)]
    fn extract_include_path(&self, directive: &str) -> String {
        if let Some(start) = directive.find('<') {
            if let Some(end) = directive.find('>') {
                return directive[start + 1..end].to_string();
            }
        }
        if let Some(start) = directive.find('"') {
            if let Some(end) = directive.rfind('"') {
                if end > start {
                    return directive[start + 1..end].to_string();
                }
            }
        }
        directive.to_string()
    }

    /// Extract variable information from declaration node
    #[allow(dead_code)]
    fn extract_variable_info(
        &self,
        node: Node,
        source: &str,
    ) -> (Option<String>, Option<String>, Option<String>) {
        let mut name = None;
        let mut var_type = None;
        let mut storage_class = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "init_declarator" => {
                    let mut init_cursor = child.walk();
                    for init_child in child.children(&mut init_cursor) {
                        if init_child.kind() == "identifier" {
                            name = Some(
                                source[init_child.start_byte()..init_child.end_byte()].to_string(),
                            );
                        }
                    }
                }
                "primitive_type" | "type_identifier" => {
                    var_type = Some(source[child.start_byte()..child.end_byte()].to_string());
                }
                "storage_class_specifier" => {
                    storage_class = Some(source[child.start_byte()..child.end_byte()].to_string());
                }
                _ => {}
            }
        }

        (name, var_type, storage_class)
    }

    /// Extract typedef name
    fn extract_typedef_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Some(source[child.start_byte()..child.end_byte()].to_string());
            }
        }
        None
    }
}

impl Default for CParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for CParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_c::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("Failed to set C language: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse C code".to_string(),
            })
    }

    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_c::language()
    }

    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            "function_definition" => SemanticNodeKind::Function,
            "struct_specifier" => SemanticNodeKind::Struct,
            "enum_specifier" => SemanticNodeKind::Enum,
            "preproc_include" | "preproc_def" => SemanticNodeKind::Import,
            "declaration" | "init_declarator" => SemanticNodeKind::Variable,
            "expression_statement" | "assignment_expression" => SemanticNodeKind::Statement,
            "call_expression" | "field_expression" | "binary_expression" => {
                SemanticNodeKind::Expression
            }
            // Signature components
            "parameter_list" | "parameter_declaration" | "pointer_declarator" => {
                SemanticNodeKind::SignatureComponent
            }
            "typedef_declaration" => SemanticNodeKind::TypeDefinition,
            "comment" => SemanticNodeKind::Comment,
            "translation_unit" => SemanticNodeKind::SourceFile,
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

        Ok(SemanticTree::new(semantic_root, ProgrammingLanguage::C))
    }

    fn compare_semantic_units(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        // Basic structural comparison for C units
        match (&old.unit_type, &new.unit_type) {
            // Compare functions
            (SemanticUnitType::Callable { .. }, SemanticUnitType::Callable { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));
                if old_name == new_name {
                    // Same function name - check if signatures match
                    let old_text = old_source
                        .node_text(&old.tree_sitter_node)
                        .unwrap_or_default();
                    let new_text = new_source
                        .node_text(&new.tree_sitter_node)
                        .unwrap_or_default();

                    if old_text == new_text {
                        SemanticSimilarity::identical()
                    } else {
                        SemanticSimilarity::signature_change(true, 0.5)
                    }
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Compare structs and enums
            (SemanticUnitType::DataStructure { .. }, SemanticUnitType::DataStructure { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));
                if old_name == new_name {
                    SemanticSimilarity::structural_refactor(0.8)
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Compare variables
            (SemanticUnitType::Variable { .. }, SemanticUnitType::Variable { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));
                if old_name == new_name {
                    SemanticSimilarity::body_change()
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Compare preprocessor directives
            (
                SemanticUnitType::Import {
                    source_module: old_module,
                    ..
                },
                SemanticUnitType::Import {
                    source_module: new_module,
                    ..
                },
            ) => {
                if old_module == new_module {
                    SemanticSimilarity::identical()
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Different types are unrelated
            _ => SemanticSimilarity::unrelated(),
        }
    }
}
