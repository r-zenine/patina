use crate::common::{ASTError, LanguageParser, ProgrammingLanguage, Result, SemanticNodeKind};
use crate::semantic_ast::{
    ImportType, MetadataNode, MetadataPosition, ModuleType, SemanticError, SemanticNode,
    SemanticSimilarity, SemanticTree, SemanticUnitType,
};
use std::collections::HashMap;
use tree_sitter::{Node, Parser, Tree};

pub struct CppParser;
impl CppParser {
    pub fn new() -> Self {
        Self
    }

    /// Check if a node kind represents trivial syntax that should not become a semantic node
    fn is_trivial_syntax_token(&self, node_kind: &str) -> bool {
        matches!(
            node_kind,
            // Punctuation and operators
            "(" | ")" | "[" | "]" | "{" | "}" | "," | ":" | "::" | ";" |
            "." | "->" | "?" | "!" | "#" | "@" | "$" | "%" |
            "^" | "&" | "*" | "-" | "=" | "+" | "|" | "\\" | "/" | "<" | ">" |
            "<<" | ">>" | "++" | "--" | "==" | "!=" | "<=" | ">=" | "&&" | "||" |

            // C++ keywords that are part of larger constructs
            "if" | "else" | "for" | "while" | "do" | "switch" | "case" | "default" |
            "break" | "continue" | "return" | "goto" | "sizeof" | "typedef" |
            "struct" | "class" | "union" | "enum" | "static" | "extern" | "auto" | "register" |
            "const" | "volatile" | "inline" | "constexpr" | "consteval" | "constinit" |
            "mutable" | "virtual" | "override" | "final" | "explicit" | "friend" |
            "public" | "private" | "protected" | "namespace" | "using" |
            "template" | "typename" | "concept" | "requires" |

            // Type specifiers
            "void" | "char" | "short" | "int" | "long" | "float" | "double" |
            "signed" | "unsigned" | "bool" | "wchar_t" | "char8_t" | "char16_t" | "char32_t" |

            // Literals and identifiers
            "string_literal" | "char_literal" | "number_literal" | "raw_string_literal" |
            "identifier" | "field_identifier" | "type_identifier" | "namespace_identifier" |
            "template_type" | "template_function" | "primitive_type" |

            // Comments and preprocessor
            "comment" | "preproc_directive" |

            // Statement-level constructs that should not be semantic nodes
            "expression_statement" | "assignment_expression" | "binary_expression" |
            "call_expression" | "field_expression" | "parenthesized_expression" |
            "unary_expression" | "conditional_expression" | "compound_statement" |
            "if_statement" | "return_statement" | "declaration" |
            "parameter_declaration" | "pointer_declarator" | "reference_declarator" |
            "function_declarator" | "parameter_list" | "argument_list" |
            "field_declaration_list" | "enumerator_list" | "enumerator" |
            "access_specifier" | "base_class_clause" | "initializer_list" |
            "template_parameter_list" | "template_argument_list" |

            // Error nodes
            "ERROR" | "MISSING"
        )
    }

    /// Build semantic node for C++ AST node with context awareness
    fn build_semantic_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        match node.kind() {
            "translation_unit" => self.build_translation_unit_node(node, source),
            "class_specifier" => self.build_class_node(node, source, parent),
            "struct_specifier" => self.build_struct_node(node, source, parent),
            "union_specifier" => self.build_union_node(node, source, parent),
            "enum_specifier" => self.build_enum_node(node, source, parent),
            "function_definition" => self.build_function_node(node, source, parent, parent_context),
            "template_declaration" => self.build_template_node(node, source, parent),
            "namespace_definition" => self.build_namespace_node(node, source, parent),
            "using_declaration" => self.build_using_node(node, source, parent),
            "typedef_declaration" => self.build_typedef_node(node, source, parent),
            "preproc_include" | "preproc_def" | "preproc_ifdef" | "preproc_if" => {
                self.build_preprocessor_node(node, source, parent)
            }
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
                        Vec::new(), // metadata_nodes
                    );
                    container_node.children = children;

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
            Vec::new(), // metadata_nodes
        );
        semantic_node.children = children;
        Ok(semantic_node)
    }

    /// Build class node (C++ class)
    fn build_class_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name = self.extract_class_name(node, source);
        let field_count = self.count_class_members(node);
        let inheritance = self.extract_base_classes(node, source);
        let is_template = self.is_template_specialization(node, parent);

        // Find metadata nodes (attributes) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_attributes(node, parent_node)
        } else {
            Vec::new()
        };

        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "class".to_string());
        if is_template {
            metadata.insert("template".to_string(), "true".to_string());
        }

        let mut semantic_node = SemanticNode::new(
            node,
            name.and_then(|_| {
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .find(|&child| child.kind() == "type_identifier")
            }),
            SemanticUnitType::DataStructure {
                is_generic: is_template,
                field_count: Some(field_count),
                inheritance,
                visibility: "public".to_string(),
                signature_node: None,
                metadata,
            },
            metadata_nodes,
        );

        // Build children from class body
        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut body_cursor = child.walk();
                for body_child in child.children(&mut body_cursor) {
                    if !self.is_trivial_syntax_token(body_child.kind()) {
                        match self.build_semantic_node(
                            body_child,
                            source,
                            Some(child),
                            Some("class"),
                        ) {
                            Ok(child_node) => children.push(child_node),
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        semantic_node.children = children;
        Ok(semantic_node)
    }

    /// Build struct node (C++ struct, similar to class but public by default)
    fn build_struct_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        // Structs are essentially classes with public default access
        let mut result = self.build_class_node(node, source, parent)?;

        // Update metadata to indicate this is a struct
        if let SemanticUnitType::DataStructure { metadata, .. } = &mut result.unit_type {
            metadata.insert("type".to_string(), "struct".to_string());
        }

        Ok(result)
    }

    /// Build union node
    fn build_union_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let mut result = self.build_class_node(node, source, parent)?;

        // Update metadata to indicate this is a union
        if let SemanticUnitType::DataStructure { metadata, .. } = &mut result.unit_type {
            metadata.insert("type".to_string(), "union".to_string());
        }

        Ok(result)
    }

    /// Build enum node
    fn build_enum_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let name = self.extract_enum_name(node, source);
        let variant_count = self.count_enum_variants(node);
        let is_class_enum = self.is_enum_class(node);

        // Find metadata nodes (attributes) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_attributes(node, parent_node)
        } else {
            Vec::new()
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "type".to_string(),
            if is_class_enum { "enum_class" } else { "enum" }.to_string(),
        );

        let semantic_node = SemanticNode::new(
            node,
            name.and_then(|_| {
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
            metadata_nodes,
        );
        Ok(semantic_node)
    }

    // Placeholder implementations for extraction methods
    fn extract_class_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return child
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
        None
    }

    fn count_class_members(&self, node: Node) -> usize {
        // Simplified count - count field declarations in body
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut body_cursor = child.walk();
                for body_child in child.children(&mut body_cursor) {
                    if matches!(
                        body_child.kind(),
                        "field_declaration" | "function_definition"
                    ) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn extract_base_classes(&self, node: Node, source: &str) -> Vec<String> {
        let mut bases = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "base_class_clause" {
                let mut base_cursor = child.walk();
                for base_child in child.children(&mut base_cursor) {
                    if base_child.kind() == "type_identifier" {
                        if let Ok(base_name) = base_child.utf8_text(source.as_bytes()) {
                            bases.push(base_name.to_string());
                        }
                    }
                }
            }
        }
        bases
    }

    fn is_template_specialization(&self, _node: Node, parent: Option<Node>) -> bool {
        if let Some(parent_node) = parent {
            parent_node.kind() == "template_declaration"
        } else {
            false
        }
    }

    fn extract_enum_name(&self, node: Node, source: &str) -> Option<String> {
        self.extract_class_name(node, source)
    }

    fn count_enum_variants(&self, node: Node) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "enumerator_list" {
                let mut enum_cursor = child.walk();
                for enum_child in child.children(&mut enum_cursor) {
                    if enum_child.kind() == "enumerator" {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn is_enum_class(&self, node: Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "class" {
                return true;
            }
        }
        false
    }

    /// Find preceding attribute siblings for a given node (following Rust parser pattern)
    fn find_preceding_attributes<'a>(
        &self,
        node: Node<'a>,
        parent: Node<'a>,
    ) -> Vec<MetadataNode<'a>> {
        let mut metadata_nodes = Vec::new();
        let mut cursor = parent.walk();
        let mut position = -1;

        // Iterate through parent's children to find preceding siblings
        let children: Vec<Node> = parent.children(&mut cursor).collect();
        let target_index = children.iter().position(|&n| n == node);

        if let Some(idx) = target_index {
            // Look backwards from the target node
            for i in (0..idx).rev() {
                let sibling = children[i];

                // C++ attributes: [[attribute]], __attribute__((attr))
                if matches!(
                    sibling.kind(),
                    "attribute_specifier" | "attribute_declaration"
                ) {
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

    // Placeholder implementations for other node builders
    fn build_function_node<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent: Option<Node<'a>>,
        parent_context: Option<&str>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        let (name, return_type, params, storage_class) = self.extract_function_info(node, source);
        let is_method = parent_context == Some("class") || parent_context == Some("struct");
        let is_template = self.is_template_specialization(node, parent);

        // Find metadata nodes (attributes) if we have a parent
        let metadata_nodes = if let Some(parent_node) = parent {
            self.find_preceding_attributes(node, parent_node)
        } else {
            Vec::new()
        };

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
        if let Some(storage) = storage_class {
            metadata.insert("storage_class".to_string(), storage);
        }
        if is_template {
            metadata.insert("template".to_string(), "true".to_string());
        }
        metadata.insert("kind".to_string(), "function".to_string());

        let mut semantic_node = SemanticNode::new(
            node,
            name.and_then(|_| {
                // Find the function name node
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .find(|&child| child.kind() == "identifier")
            }),
            SemanticUnitType::Callable {
                is_generic: is_template,
                parameter_count: params,
                return_type,
                is_async: false, // C++ doesn't have async keywords like JavaScript
                visibility: if is_method { "public" } else { "global" }.to_string(),
                is_method,
                signature_node: None,
                body_node: None,
                metadata,
            },
            metadata_nodes,
        );

        semantic_node.children = children;
        Ok(semantic_node)
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

    /// Count parameters in parameter list
    fn count_parameters(&self, parameter_list: Node) -> usize {
        let mut count = 0;
        let mut cursor = parameter_list.walk();
        for child in parameter_list.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                count += 1;
            }
        }
        count
    }

    fn build_template_node<'a>(
        &self,
        _node: Node<'a>,
        _source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        Err(SemanticError::TreeBuildError(
            "C++ template nodes not implemented yet".to_string(),
        ))
    }

    fn build_namespace_node<'a>(
        &self,
        _node: Node<'a>,
        _source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        Err(SemanticError::TreeBuildError(
            "C++ namespace nodes not implemented yet".to_string(),
        ))
    }

    fn build_using_node<'a>(
        &self,
        _node: Node<'a>,
        _source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        Err(SemanticError::TreeBuildError(
            "C++ using nodes not implemented yet".to_string(),
        ))
    }

    fn build_typedef_node<'a>(
        &self,
        _node: Node<'a>,
        _source: &str,
        _parent: Option<Node<'a>>,
    ) -> std::result::Result<SemanticNode<'a>, SemanticError> {
        Err(SemanticError::TreeBuildError(
            "C++ typedef nodes not implemented yet".to_string(),
        ))
    }

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

    /// Extract include path from preprocessor directive
    fn extract_include_path(&self, directive: &str) -> String {
        if let Some(start) = directive.find('<') {
            if let Some(end) = directive.find('>') {
                return directive[start + 1..end].to_string();
            }
        }
        if let Some(start) = directive.find('"') {
            if let Some(end) = directive.rfind('"') {
                if start != end {
                    return directive[start + 1..end].to_string();
                }
            }
        }
        directive.to_string()
    }
}

impl Default for CppParser {
    fn default() -> Self {
        Self::new()
    }
}
impl LanguageParser for CppParser {
    fn try_parse(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_cpp::language())
            .map_err(|e| ASTError::TreeSitterError {
                error: format!("C++ error: {e}"),
            })?;
        parser
            .parse(content, None)
            .ok_or_else(|| ASTError::ParseError {
                message: "Failed to parse C++".to_string(),
            })
    }
    fn get_language(&self) -> tree_sitter::Language {
        tree_sitter_cpp::language()
    }
    fn classify_node_kind(&self, node_kind: &str) -> SemanticNodeKind {
        match node_kind {
            "function_definition" => SemanticNodeKind::Function,
            "class_specifier" => SemanticNodeKind::Class,
            // Signature components
            "parameter_list"
            | "parameter_declaration"
            | "template_parameter_list"
            | "template_parameter"
            | "pointer_declarator"
            | "reference_declarator" => SemanticNodeKind::SignatureComponent,
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

        Ok(SemanticTree::new(semantic_root, ProgrammingLanguage::Cpp))
    }

    fn compare_semantic_units(
        &self,
        old: &SemanticNode,
        new: &SemanticNode,
        old_source: &dyn crate::ast_diff::SourceProvider,
        new_source: &dyn crate::ast_diff::SourceProvider,
    ) -> SemanticSimilarity {
        // Basic structural comparison for C++ units
        match (&old.unit_type, &new.unit_type) {
            // Compare classes/structs/unions
            (SemanticUnitType::DataStructure { .. }, SemanticUnitType::DataStructure { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));

                // Enhanced name matching for C++ - handle template vs non-template
                let names_match = match (&old_name, &new_name) {
                    (Some(old), Some(new)) => {
                        // Direct match
                        if old == new {
                            true
                        } else {
                            // Check if one is a template version of the other
                            // e.g., "DataProcessor" matches "DataProcessor<T, N>"
                            let old_base = old.split('<').next().unwrap_or(old);
                            let new_base = new.split('<').next().unwrap_or(new);
                            old_base == new_base
                        }
                    }
                    _ => false,
                };

                if names_match {
                    // Same base name - check for structural changes
                    let old_text = old_source
                        .node_text(&old.tree_sitter_node)
                        .unwrap_or_default();
                    let new_text = new_source
                        .node_text(&new.tree_sitter_node)
                        .unwrap_or_default();

                    if old_text == new_text {
                        SemanticSimilarity::identical()
                    } else {
                        // Check if it's template vs non-template change
                        let old_is_template = self.check_template_status(old);
                        let new_is_template = self.check_template_status(new);

                        if old_is_template != new_is_template {
                            // Template transformation - significant change but same entity
                            SemanticSimilarity::signature_change(true, 0.8)
                        } else {
                            // Check for enum -> enum class transformation
                            let old_enum_type = self.check_enum_type(old);
                            let new_enum_type = self.check_enum_type(new);

                            if old_enum_type != new_enum_type {
                                if matches!(
                                    (&old_enum_type, &new_enum_type),
                                    (Some(old), Some(new)) if (old == "enum" && new == "enum_class") || (old == "enum_class" && new == "enum")
                                ) {
                                    // Enum type transformation - significant signature change but same entity
                                    SemanticSimilarity::signature_change(true, 0.8)
                                } else {
                                    // Same template status - likely structural changes
                                    SemanticSimilarity::structural_refactor(0.9)
                                }
                            } else {
                                // Same template and enum status - likely structural changes
                                SemanticSimilarity::structural_refactor(0.9)
                            }
                        }
                    }
                } else {
                    SemanticSimilarity::unrelated()
                }
            }

            // Compare functions
            (SemanticUnitType::Callable { .. }, SemanticUnitType::Callable { .. }) => {
                let old_name = old
                    .name_node
                    .and_then(|n| old_source.node_text(&n).ok().map(|s| s.to_string()));
                let new_name = new
                    .name_node
                    .and_then(|n| new_source.node_text(&n).ok().map(|s| s.to_string()));

                if old_name == new_name {
                    // Same function name - check for signature vs body changes
                    let old_is_template = self.check_template_status(old);
                    let new_is_template = self.check_template_status(new);

                    if old_is_template != new_is_template {
                        // Function became template or vice versa
                        SemanticSimilarity::signature_change(true, 0.7)
                    } else {
                        // Extract parameter counts and return types for comparison
                        let old_params = match &old.unit_type {
                            SemanticUnitType::Callable {
                                parameter_count, ..
                            } => *parameter_count,
                            _ => 0,
                        };
                        let new_params = match &new.unit_type {
                            SemanticUnitType::Callable {
                                parameter_count, ..
                            } => *parameter_count,
                            _ => 0,
                        };

                        let old_return = match &old.unit_type {
                            SemanticUnitType::Callable { return_type, .. } => {
                                return_type.as_deref()
                            }
                            _ => None,
                        };
                        let new_return = match &new.unit_type {
                            SemanticUnitType::Callable { return_type, .. } => {
                                return_type.as_deref()
                            }
                            _ => None,
                        };

                        if old_params != new_params || old_return != new_return {
                            SemanticSimilarity::signature_change(false, 0.6)
                        } else {
                            SemanticSimilarity::body_change()
                        }
                    }
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

            // Compare imports/preprocessor directives
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

impl CppParser {
    /// Check if a semantic node represents a template
    fn check_template_status(&self, node: &SemanticNode) -> bool {
        match &node.unit_type {
            SemanticUnitType::DataStructure { is_generic, .. }
            | SemanticUnitType::Callable { is_generic, .. } => *is_generic,
            _ => false,
        }
    }

    /// Check the enum type (enum vs enum class) from metadata
    fn check_enum_type(&self, node: &SemanticNode) -> Option<String> {
        if let SemanticUnitType::DataStructure { metadata, .. } = &node.unit_type {
            metadata.get("type").cloned()
        } else {
            None
        }
    }
}
