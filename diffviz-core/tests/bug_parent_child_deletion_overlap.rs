use diffviz_core::{
    LanguageParser,
    ast_diff::{LineRange, NodeLike, SourceError, SourceProvider},
    common::ProgrammingLanguage,
    parsers::typescript::TypeScriptParser,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

#[cfg(test)]
mod parent_child_deletion_overlap {
    use super::*;

    #[test]
    fn typescript_class_to_functional_refactor_overlap() {
        // This test captures the regression where a deleted parent node (class)
        // is reported as a separate semantic pair from its deleted children (methods).
        // We should NOT report both the parent deletion AND individual child deletions.

        let old_code = r#"import React from 'react';

interface Props {
  user: { name: string };
  onGreeted?: () => void;
}

class Greeting extends React.Component<Props> {
  componentDidMount() {
    console.log('Component mounted');
  }

  render() {
    const { user, onGreeted } = this.props;
    return (
      <div className="greeting-container">
        <h1>Welcome, {user.name}!</h1>
        {onGreeted && <button onClick={onGreeted}>Acknowledged</button>}
      </div>
    );
  }
}
"#;

        let new_code = r#"import React, { useState, useEffect } from 'react';

interface Props {
  user: { name: string };
  onGreeted?: () => void;
}

const Greeting: React.FC<Props> = ({ user, onGreeted }) => {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    console.log('Component mounted');
  }, []);

  const handleAcknowledge = () => {
    if (onGreeted) {
      onGreeted();
    }
    setIsVisible(false);
  };

  if (!isVisible) return null;

  return (
    <div className="greeting-container">
      <h1>Welcome, {user.name}!</h1>
      {onGreeted && <button onClick={handleAcknowledge}>Acknowledged</button>}
    </div>
  );
};
"#;

        let parser = TypeScriptParser::new();

        // Parse AST trees
        let old_tree = parser
            .try_parse(old_code)
            .expect("Failed to parse old TypeScript code");
        let new_tree = parser
            .try_parse(new_code)
            .expect("Failed to parse new TypeScript code");

        let old_semantic_tree = parser
            .build_semantic_tree(&old_tree, old_code)
            .expect("Failed to build old semantic tree");
        let new_semantic_tree = parser
            .build_semantic_tree(&new_tree, new_code)
            .expect("Failed to build new semantic tree");

        #[derive(Clone)]
        struct SimpleSource {
            content: String,
        }

        impl SourceProvider for SimpleSource {
            fn node_text(&self, node: &dyn NodeLike) -> Result<&str, SourceError> {
                let start = node.start_byte();
                let end = node.end_byte();
                Ok(&self.content.as_str()[start..end])
            }

            fn line_range(&self, node: &dyn NodeLike) -> LineRange {
                if let Some(ts_node) = node.as_tree_sitter_node() {
                    let start_pos = ts_node.start_position();
                    let end_pos = ts_node.end_position();
                    LineRange {
                        start_line: start_pos.row + 1,
                        end_line: end_pos.row + 1,
                        start_column: start_pos.column,
                        end_column: end_pos.column,
                    }
                } else {
                    LineRange {
                        start_line: 1,
                        end_line: 1,
                        start_column: node.start_byte(),
                        end_column: node.end_byte(),
                    }
                }
            }

            fn clone_box(&self) -> Box<dyn SourceProvider> {
                Box::new(self.clone())
            }
        }

        let old_source = SimpleSource {
            content: old_code.to_string(),
        };
        let new_source = SimpleSource {
            content: new_code.to_string(),
        };

        // Build semantic pairs
        let semantic_pairs = build_semantic_pairs(
            &old_semantic_tree,
            &new_semantic_tree,
            &old_source,
            &new_source,
            &parser,
        )
        .expect("Failed to build semantic pairs");

        // Convert to reviewable diffs (not used in this test)
        let _reviewable_diffs = semantic_pairs_to_reviewable_diffs(
            &semantic_pairs,
            ProgrammingLanguage::TypeScript,
            &old_source,
            &new_source,
            &parser,
        );

        // Analyze the issue: check for overlapping parent-child deletions
        use diffviz_core::semantic_ast::SemanticPair;

        let deleted_pairs: Vec<_> = semantic_pairs
            .iter()
            .filter_map(|pair| {
                if let SemanticPair::Deletion { unit } = pair {
                    Some(*unit)
                } else {
                    None
                }
            })
            .collect();

        println!("\n=== Parent-Child Deletion Overlap Analysis ===");
        println!("Total deleted pairs: {}", deleted_pairs.len());

        for unit in &deleted_pairs {
            let ts_node = &unit.tree_sitter_node;
            println!(
                "Deleted: {:?} @ {}:{}",
                ts_node.kind(),
                ts_node.start_position().row,
                ts_node.start_position().column
            );
        }

        // The bug: we're getting separate semantic pairs for:
        // 1. The deleted class_declaration (parent)
        // 2. The deleted method_definition nodes (children)
        //
        // Expected: Either only the parent OR the children, not both
        // Current: Both parent and children are reported

        let has_parent_deletion = deleted_pairs
            .iter()
            .any(|unit| unit.tree_sitter_node.kind() == "class_declaration");

        let has_child_deletions = deleted_pairs
            .iter()
            .any(|unit| unit.tree_sitter_node.kind() == "method_definition");

        if has_parent_deletion && has_child_deletions {
            panic!(
                "Bug confirmed: Parent (class_declaration) and children (method_definition) \
                 are both reported as separate semantic pairs. This creates overlapping/redundant deletions."
            );
        }
    }

    #[test]
    fn rust_struct_impl_deletion_overlap() {
        // Same issue should occur in Rust when a struct and its impl block are deleted

        let old_code = r#"
pub struct DataProcessor {
    data: Vec<i32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add_data(&mut self, value: i32) {
        self.data.push(value);
    }

    pub fn process(&mut self) {
        for item in &mut self.data {
            *item *= 2;
        }
    }
}
"#;

        let new_code = r#"
pub struct DataProcessor<T> {
    data: Vec<T>,
}

impl<T> DataProcessor<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add_data(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn process<F>(&mut self, f: F)
    where
        F: Fn(&mut T),
    {
        for item in &mut self.data {
            f(item);
        }
    }
}
"#;

        let parser = diffviz_core::parsers::RustParser::new();

        let old_tree = parser
            .try_parse(old_code)
            .expect("Failed to parse old Rust code");
        let new_tree = parser
            .try_parse(new_code)
            .expect("Failed to parse new Rust code");

        let old_semantic = parser
            .build_semantic_tree(&old_tree, old_code)
            .expect("Failed to build old semantic tree");
        let new_semantic = parser
            .build_semantic_tree(&new_tree, new_code)
            .expect("Failed to build new semantic tree");

        #[derive(Clone)]
        struct SimpleSource {
            content: String,
        }

        impl SourceProvider for SimpleSource {
            fn node_text(&self, node: &dyn NodeLike) -> Result<&str, SourceError> {
                let start = node.start_byte();
                let end = node.end_byte();
                Ok(&self.content.as_str()[start..end])
            }

            fn line_range(&self, node: &dyn NodeLike) -> LineRange {
                if let Some(ts_node) = node.as_tree_sitter_node() {
                    let start_pos = ts_node.start_position();
                    let end_pos = ts_node.end_position();
                    LineRange {
                        start_line: start_pos.row + 1,
                        end_line: end_pos.row + 1,
                        start_column: start_pos.column,
                        end_column: end_pos.column,
                    }
                } else {
                    LineRange {
                        start_line: 1,
                        end_line: 1,
                        start_column: node.start_byte(),
                        end_column: node.end_byte(),
                    }
                }
            }

            fn clone_box(&self) -> Box<dyn SourceProvider> {
                Box::new(self.clone())
            }
        }

        let old_source = SimpleSource {
            content: old_code.to_string(),
        };
        let new_source = SimpleSource {
            content: new_code.to_string(),
        };

        let semantic_pairs = build_semantic_pairs(
            &old_semantic,
            &new_semantic,
            &old_source,
            &new_source,
            &parser,
        )
        .expect("Failed to build semantic pairs");

        // Check for parent-child overlap in Rust code
        use diffviz_core::semantic_ast::SemanticPair;

        let deleted_units: Vec<_> = semantic_pairs
            .iter()
            .filter_map(|pair| {
                if let SemanticPair::Deletion { unit } = pair {
                    Some(*unit)
                } else {
                    None
                }
            })
            .collect();

        println!("\n=== Rust Parent-Child Deletion Overlap Analysis ===");
        println!("Total deleted pairs: {}", deleted_units.len());

        for unit in &deleted_units {
            let ts_node = &unit.tree_sitter_node;
            println!(
                "Deleted: {:?} @ {}:{}",
                ts_node.kind(),
                ts_node.start_position().row,
                ts_node.start_position().column
            );
        }

        // Same issue could happen with struct and its function items
        let has_parent = deleted_units
            .iter()
            .any(|unit| unit.tree_sitter_node.kind() == "struct_item");

        let has_children = deleted_units
            .iter()
            .any(|unit| unit.tree_sitter_node.kind() == "function_item");

        if has_parent && has_children {
            panic!(
                "Bug confirmed in Rust: Parent and children nodes \
                 are both reported as separate semantic pairs"
            );
        }
    }
}
