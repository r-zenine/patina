//! Utilities for extracting readable names from AST nodes

use crate::{common::SemanticNodeKind, reviewable_diff::ReviewableDiff};

/// Extract a readable name for the boundary
pub fn extract_boundary_name(reviewable: &ReviewableDiff) -> String {
    let (display_node, source_provider) =
        reviewable.boundary.change_status.display_node_with_source(
            reviewable.old_source.as_ref(),
            reviewable.new_source.as_ref(),
        );

    match reviewable.boundary.semantic_kind {
        SemanticNodeKind::Function => display_node
            .identifier
            .clone()
            .unwrap_or_else(|| "function".to_string()),
        SemanticNodeKind::Struct => display_node
            .identifier
            .clone()
            .unwrap_or_else(|| "struct".to_string()),
        SemanticNodeKind::Enum => display_node
            .identifier
            .clone()
            .unwrap_or_else(|| "enum".to_string()),
        SemanticNodeKind::Class
        | SemanticNodeKind::Interface
        | SemanticNodeKind::ImplBlock
        | SemanticNodeKind::Module
        | SemanticNodeKind::Import
        | SemanticNodeKind::Variable
        | SemanticNodeKind::SignatureComponent
        | SemanticNodeKind::Statement
        | SemanticNodeKind::Expression
        | SemanticNodeKind::TypeDefinition
        | SemanticNodeKind::Comment
        | SemanticNodeKind::SourceFile
        | SemanticNodeKind::Other(_) => {
            if let Ok(source_text) = source_provider.node_text(display_node) {
                // User requested: "drop size conditions everywhere, any diff even if it's one line is meaningful"
                // Show the full first line instead of truncating
                source_text.lines().next().unwrap_or("").trim().to_string()
            } else {
                format!("{:?}", reviewable.boundary.semantic_kind).to_lowercase()
            }
        }
    }
}
