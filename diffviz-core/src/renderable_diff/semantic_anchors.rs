//! Utilities for extracting semantic anchors from lines
//!
//! This module provides functionality to identify semantic anchors in code lines,
//! such as function names, variable names, and field names. These anchors are used
//! by the Myers diff algorithm to treat semantically related lines as having edit
//! distance 0, resulting in better diff grouping.

use super::{SemanticAnchor, SemanticAnchorType};
use crate::{
    common::{ProgrammingLanguage, SemanticNodeKind},
    reviewable_diff::ReviewableDiff,
};

/// Extract semantic anchor from a line based on the AST context
pub fn extract_semantic_anchor(
    line: &str,
    reviewable: &ReviewableDiff,
    _line_byte_start: usize,
) -> Option<SemanticAnchor> {
    // Quick checks to skip obviously non-anchorable lines
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
        return None;
    }

    // Try to extract based on the boundary's semantic type
    match &reviewable.boundary.semantic_kind {
        SemanticNodeKind::Function => extract_function_anchor(line, reviewable),
        SemanticNodeKind::Struct => extract_struct_anchor(line, reviewable),
        SemanticNodeKind::Enum => extract_enum_anchor(line, reviewable),
        _ => {
            // For other types, try to detect patterns
            extract_generic_anchor(line, reviewable.language)
        }
    }
}

/// Extract function-related anchors
fn extract_function_anchor(line: &str, reviewable: &ReviewableDiff) -> Option<SemanticAnchor> {
    let trimmed = line.trim();

    // Check if this is a function signature line
    if let Some(name) = extract_function_name_from_signature(trimmed, reviewable.language) {
        return Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: name,
        });
    }

    // Otherwise try generic patterns
    extract_generic_anchor(line, reviewable.language)
}

/// Extract struct-related anchors
fn extract_struct_anchor(line: &str, reviewable: &ReviewableDiff) -> Option<SemanticAnchor> {
    let trimmed = line.trim();

    // Check if this is a struct declaration
    if trimmed.starts_with("struct ") {
        if let Some(name) = extract_struct_name_from_declaration(trimmed) {
            return Some(SemanticAnchor {
                anchor_type: SemanticAnchorType::StructDeclaration,
                identifier: name,
            });
        }
    }

    // Otherwise try generic patterns
    extract_generic_anchor(line, reviewable.language)
}

/// Extract enum-related anchors
fn extract_enum_anchor(line: &str, reviewable: &ReviewableDiff) -> Option<SemanticAnchor> {
    let trimmed = line.trim();

    // Check if this is an enum declaration
    if trimmed.starts_with("enum ") {
        if let Some(name) = extract_enum_name_from_declaration(trimmed) {
            return Some(SemanticAnchor {
                anchor_type: SemanticAnchorType::EnumDeclaration,
                identifier: name,
            });
        }
    }

    // Otherwise try generic patterns
    extract_generic_anchor(line, reviewable.language)
}

/// Extract generic anchors (assignments, method calls, etc.)
fn extract_generic_anchor(line: &str, language: ProgrammingLanguage) -> Option<SemanticAnchor> {
    let trimmed = line.trim();

    // Check for variable assignments
    if let Some(name) = extract_assignment_target(trimmed, language) {
        return Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::VariableAssignment,
            identifier: name,
        });
    }

    // Check for field assignments
    if let Some(name) = extract_field_assignment(trimmed, language) {
        return Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FieldAssignment,
            identifier: name,
        });
    }

    // Check for method calls
    if let Some(name) = extract_method_call(trimmed, language) {
        return Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::MethodCall,
            identifier: name,
        });
    }

    // Check for imports
    if let Some(name) = extract_import(trimmed, language) {
        return Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::Import,
            identifier: name,
        });
    }

    None
}

/// Extract function name from a function signature line
fn extract_function_name_from_signature(
    line: &str,
    language: ProgrammingLanguage,
) -> Option<String> {
    match language {
        ProgrammingLanguage::Rust => {
            // fn function_name(
            if let Some(fn_pos) = line.find("fn ") {
                let after_fn = &line[fn_pos + 3..];
                if let Some(paren_pos) = after_fn.find('(') {
                    let name = after_fn[..paren_pos].trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        ProgrammingLanguage::Go => {
            // func functionName(
            if let Some(func_pos) = line.find("func ") {
                let after_func = &line[func_pos + 5..];
                // Handle receiver methods: func (r Receiver) methodName(
                let start = if after_func.starts_with('(') {
                    after_func.find(')').map(|i| i + 1).unwrap_or(0)
                } else {
                    0
                };
                if let Some(paren_pos) = after_func[start..].find('(') {
                    let name = after_func[start..start + paren_pos].trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        ProgrammingLanguage::Python => {
            // def function_name(
            if let Some(def_pos) = line.find("def ") {
                let after_def = &line[def_pos + 4..];
                if let Some(paren_pos) = after_def.find('(') {
                    let name = after_def[..paren_pos].trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        ProgrammingLanguage::TypeScript | ProgrammingLanguage::JavaScript => {
            // function functionName(
            // const functionName = (
            // async function functionName(
            if let Some(func_pos) = line.find("function ") {
                let after_func = &line[func_pos + 9..];
                if let Some(paren_pos) = after_func.find('(') {
                    let name = after_func[..paren_pos].trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            } else if line.contains("const ") || line.contains("let ") || line.contains("var ") {
                // Arrow functions
                if let Some(eq_pos) = line.find(" = ") {
                    let before_eq = &line[..eq_pos];
                    if let Some(last_space) = before_eq.rfind(' ') {
                        let name = before_eq[last_space + 1..].trim();
                        if !name.is_empty() && line[eq_pos + 3..].trim_start().starts_with('(') {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }
        _ => {}
    }
    None
}

/// Extract struct name from declaration
fn extract_struct_name_from_declaration(line: &str) -> Option<String> {
    if let Some(struct_pos) = line.find("struct ") {
        let after_struct = &line[struct_pos + 7..];
        let name_end = after_struct
            .find(|c: char| c.is_whitespace() || c == '{' || c == '<')
            .unwrap_or(after_struct.len());
        let name = after_struct[..name_end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Extract enum name from declaration
fn extract_enum_name_from_declaration(line: &str) -> Option<String> {
    if let Some(enum_pos) = line.find("enum ") {
        let after_enum = &line[enum_pos + 5..];
        let name_end = after_enum
            .find(|c: char| c.is_whitespace() || c == '{' || c == '<')
            .unwrap_or(after_enum.len());
        let name = after_enum[..name_end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Extract variable name from assignment
fn extract_assignment_target(line: &str, language: ProgrammingLanguage) -> Option<String> {
    match language {
        ProgrammingLanguage::Rust => {
            // let var_name =
            // let mut var_name =
            // const VAR_NAME: Type =
            if line.starts_with("let ") || line.starts_with("const ") {
                if let Some(eq_pos) = line.find(" = ") {
                    let between = if line.starts_with("let ") {
                        &line[4..eq_pos]
                    } else {
                        &line[6..eq_pos]
                    };
                    // Remove mut keyword and type annotation
                    let name = between.trim_start_matches("mut ").split(':').next()?.trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        ProgrammingLanguage::Go => {
            // var_name :=
            // var var_name =
            if let Some(assign_pos) = line.find(":=") {
                let name = line[..assign_pos].trim();
                if !name.is_empty() && !name.contains(' ') {
                    return Some(name.to_string());
                }
            } else if line.starts_with("var ") {
                if let Some(eq_pos) = line.find(" = ") {
                    let between = &line[4..eq_pos];
                    let name = between.split_whitespace().next()?.trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        ProgrammingLanguage::Python => {
            // var_name =
            if let Some(eq_pos) = line.find(" = ") {
                let name = line[..eq_pos].trim();
                if !name.is_empty() && !name.contains(' ') {
                    return Some(name.to_string());
                }
            }
        }
        ProgrammingLanguage::TypeScript | ProgrammingLanguage::JavaScript => {
            // const var_name =
            // let var_name =
            // var var_name =
            if line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var ") {
                if let Some(eq_pos) = line.find(" = ") {
                    let start = line.find(' ').unwrap() + 1;
                    let between = &line[start..eq_pos];
                    // Remove type annotation
                    let name = between.split(':').next()?.trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        _ => {}
    }
    None
}

/// Extract field assignment pattern (e.g., obj.field =)
fn extract_field_assignment(line: &str, _language: ProgrammingLanguage) -> Option<String> {
    // Look for pattern: something.field =
    if let Some(eq_pos) = line.find(" = ") {
        let before_eq = line[..eq_pos].trim();
        if let Some(_dot_pos) = before_eq.rfind('.') {
            // Get the full field path (e.g., "user.name" or "config.timeout")
            return Some(before_eq.to_string());
        }
    }
    None
}

/// Extract method call pattern
fn extract_method_call(line: &str, _language: ProgrammingLanguage) -> Option<String> {
    // Look for pattern: something.method(
    if let Some(paren_pos) = line.find('(') {
        let before_paren = line[..paren_pos].trim();
        if let Some(dot_pos) = before_paren.rfind('.') {
            let method_part = &before_paren[dot_pos + 1..];
            if !method_part.is_empty() {
                // Return the full method path for better matching
                return Some(before_paren.to_string());
            }
        }
    }
    None
}

/// Extract import pattern
fn extract_import(line: &str, language: ProgrammingLanguage) -> Option<String> {
    match language {
        ProgrammingLanguage::Rust => {
            // use some::module;
            if let Some(after_use) = line.strip_prefix("use ") {
                let end = after_use.find(';').unwrap_or(after_use.len());
                let import = after_use[..end].trim();
                if !import.is_empty() {
                    return Some(import.to_string());
                }
            }
        }
        ProgrammingLanguage::Go => {
            // import "package"
            if line.trim().starts_with("import ") {
                let after_import = line.trim()[7..].trim();
                let import = after_import.trim_matches('"');
                if !import.is_empty() {
                    return Some(import.to_string());
                }
            }
        }
        ProgrammingLanguage::Python => {
            // import module
            // from module import something
            if line.starts_with("import ") || line.starts_with("from ") {
                return Some(line.trim().to_string());
            }
        }
        ProgrammingLanguage::TypeScript | ProgrammingLanguage::JavaScript => {
            // import something from 'module'
            if line.starts_with("import ") {
                return Some(line.trim().to_string());
            }
        }
        _ => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_signature_rust() {
        let anchor = extract_function_name_from_signature(
            "fn calculate_total(items: Vec<Item>) -> i32 {",
            ProgrammingLanguage::Rust,
        );
        assert_eq!(anchor, Some("calculate_total".to_string()));

        let anchor = extract_function_name_from_signature(
            "pub async fn fetch_data() {",
            ProgrammingLanguage::Rust,
        );
        assert_eq!(anchor, Some("fetch_data".to_string()));
    }

    #[test]
    fn test_extract_function_signature_go() {
        let anchor = extract_function_name_from_signature(
            "func calculateTotal(items []Item) int {",
            ProgrammingLanguage::Go,
        );
        assert_eq!(anchor, Some("calculateTotal".to_string()));

        let anchor = extract_function_name_from_signature(
            "func (s *Server) handleRequest(w http.ResponseWriter) {",
            ProgrammingLanguage::Go,
        );
        assert_eq!(anchor, Some("handleRequest".to_string()));
    }

    #[test]
    fn test_extract_function_signature_python() {
        let anchor = extract_function_name_from_signature(
            "def calculate_total(items: List[Item]) -> int:",
            ProgrammingLanguage::Python,
        );
        assert_eq!(anchor, Some("calculate_total".to_string()));

        let anchor = extract_function_name_from_signature(
            "async def fetch_data():",
            ProgrammingLanguage::Python,
        );
        assert_eq!(anchor, Some("fetch_data".to_string()));
    }

    #[test]
    fn test_extract_assignment_rust() {
        let anchor =
            extract_assignment_target("let config = Config::new();", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("config".to_string()));

        let anchor = extract_assignment_target("let mut counter = 0;", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("counter".to_string()));

        let anchor =
            extract_assignment_target("const MAX_SIZE: usize = 100;", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("MAX_SIZE".to_string()));
    }

    #[test]
    fn test_extract_assignment_go() {
        let anchor = extract_assignment_target("config := NewConfig()", ProgrammingLanguage::Go);
        assert_eq!(anchor, Some("config".to_string()));

        let anchor = extract_assignment_target("var counter = 0", ProgrammingLanguage::Go);
        assert_eq!(anchor, Some("counter".to_string()));
    }

    #[test]
    fn test_extract_field_assignment() {
        let anchor = extract_field_assignment("user.name = \"Alice\";", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("user.name".to_string()));

        let anchor = extract_field_assignment("config.timeout = 30;", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("config.timeout".to_string()));

        let anchor = extract_field_assignment("self.data.value = 42;", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("self.data.value".to_string()));
    }

    #[test]
    fn test_extract_method_call() {
        let anchor = extract_method_call("client.send_request(data);", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("client.send_request".to_string()));

        let anchor = extract_method_call("self.process_data();", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("self.process_data".to_string()));
    }

    #[test]
    fn test_extract_import() {
        let anchor = extract_import("use std::collections::HashMap;", ProgrammingLanguage::Rust);
        assert_eq!(anchor, Some("std::collections::HashMap".to_string()));

        let anchor = extract_import("import \"fmt\"", ProgrammingLanguage::Go);
        assert_eq!(anchor, Some("fmt".to_string()));

        let anchor = extract_import("from typing import List, Dict", ProgrammingLanguage::Python);
        assert_eq!(anchor, Some("from typing import List, Dict".to_string()));
    }
}
