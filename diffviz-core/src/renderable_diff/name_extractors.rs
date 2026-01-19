//! Utilities for extracting readable names from AST nodes

use crate::{
    common::SemanticNodeKind,
    reviewable_diff::{NodeChangeStatus, ReviewableDiff},
};

/// Extract a readable name for the boundary
pub fn extract_boundary_name(reviewable: &ReviewableDiff) -> String {
    let display_node =
        get_display_node(&reviewable.boundary.change_status).expect("Should have display node");

    if let Ok(source_text) = reviewable.new_source.node_text(display_node) {
        let first_line = source_text.lines().next().unwrap_or("").trim();

        match reviewable.boundary.semantic_kind {
            SemanticNodeKind::Function => {
                extract_function_name(first_line).unwrap_or_else(|| "function".to_string())
            }
            SemanticNodeKind::Struct => {
                extract_struct_name(first_line).unwrap_or_else(|| "struct".to_string())
            }
            SemanticNodeKind::Enum => {
                extract_enum_name(first_line).unwrap_or_else(|| "enum".to_string())
            }
            _ => {
                // User requested: "drop size conditions everywhere, any diff even if it's one line is meaningful"
                // Show the full first line instead of truncating
                first_line.to_string()
            }
        }
    } else {
        format!("{:?}", reviewable.boundary.semantic_kind).to_lowercase()
    }
}

/// Extract function name from function definition line
fn extract_function_name(line: &str) -> Option<String> {
    if let Some(fn_pos) = line.find("fn ") {
        let after_fn = &line[fn_pos + 3..];
        if let Some(paren_pos) = after_fn.find('(') {
            let name = after_fn[..paren_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract struct name from struct definition line
fn extract_struct_name(line: &str) -> Option<String> {
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

/// Extract enum name from enum definition line  
fn extract_enum_name(line: &str) -> Option<String> {
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

/// Get the display node from a NodeChangeStatus (helper function)
fn get_display_node(change_status: &NodeChangeStatus) -> Option<&crate::ast_diff::OwnedNodeData> {
    match change_status {
        NodeChangeStatus::Unchanged { node, .. } => Some(node),
        NodeChangeStatus::Added { node, .. } => Some(node),
        NodeChangeStatus::Deleted { node, .. } => Some(node),
        NodeChangeStatus::Modified { new_node, .. } => Some(new_node),
        NodeChangeStatus::Moved { new_node, .. } => Some(new_node),
        NodeChangeStatus::Reordered { new_node, .. } => Some(new_node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            extract_function_name("fn hello() {"),
            Some("hello".to_string())
        );
        assert_eq!(
            extract_function_name("pub fn test_something(a: i32) -> bool {"),
            Some("test_something".to_string())
        );
        assert_eq!(
            extract_function_name("async fn async_func() {"),
            Some("async_func".to_string())
        );
        assert_eq!(extract_function_name("not a function"), None);
    }

    #[test]
    fn test_extract_struct_name() {
        assert_eq!(
            extract_struct_name("struct MyStruct {"),
            Some("MyStruct".to_string())
        );
        assert_eq!(
            extract_struct_name("pub struct Config<T> {"),
            Some("Config".to_string())
        );
        assert_eq!(
            extract_struct_name("struct Point { x: i32, y: i32 }"),
            Some("Point".to_string())
        );
        assert_eq!(extract_struct_name("not a struct"), None);
    }

    #[test]
    fn test_extract_enum_name() {
        assert_eq!(extract_enum_name("enum Color {"), Some("Color".to_string()));
        assert_eq!(
            extract_enum_name("pub enum Result<T, E> {"),
            Some("Result".to_string())
        );
        assert_eq!(
            extract_enum_name("enum Status { Active, Inactive }"),
            Some("Status".to_string())
        );
        assert_eq!(extract_enum_name("not an enum"), None);
    }
}
