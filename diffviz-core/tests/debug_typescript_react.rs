use diffviz_core::{
    LanguageParser,
    ast_diff::{LineRange, NodeLike, SourceError, SourceProvider},
    common::ProgrammingLanguage,
    parsers::typescript::TypeScriptParser,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

const OLD_CODE: &str = r#"import React from 'react';
import { useContext } from 'react';

// User profile type
interface UserProfile {
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
}

// Props for the Greeting component
interface Props {
  user: UserProfile;
  onGreeted?: () => void;
}

/**
 * Greeting component - displays user greeting
 * @param props - Component props
 */
class Greeting extends React.Component<Props> {
  componentDidMount() {
    console.log('Component mounted');
  }

  render() {
    const { user, onGreeted } = this.props;
    return (
      <div className="greeting-container">
        <h1>Welcome, {user.name}!</h1>
        <p>Email: {user.email}</p>
        <p>Role: {user.role}</p>
        {onGreeted && (
          <button onClick={onGreeted}>Acknowledged</button>
        )}
      </div>
    );
  }
}"#;

const NEW_CODE: &str = r#"import React, { useState, useEffect } from 'react';
import { useContext } from 'react';

// User profile type
interface UserProfile {
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
}

// Props for the Greeting component
interface Props {
  user: UserProfile;
  onGreeted?: () => void;
}

/**
 * Greeting component - displays user greeting using hooks
 * @param props - Component props
 */
const Greeting: React.FC<Props> = ({ user, onGreeted }) => {
  const [isVisible, setIsVisible] = useState(true);
  const [message, setMessage] = useState('');

  useEffect(() => {
    console.log('Component mounted');
    setMessage(`Welcome, ${user.name}!`);
  }, [user.name]);

  const handleAcknowledge = () => {
    if (onGreeted) {
      onGreeted();
    }
    setIsVisible(false);
  };

  if (!isVisible) return null;

  return (
    <div className="greeting-container">
      <h1>{message}</h1>
      <p>Email: {user.email}</p>
      <p>Role: {user.role}</p>
      {onGreeted && (
        <button onClick={handleAcknowledge}>Acknowledged</button>
      )}
    </div>
  );
};"#;

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

#[test]
fn debug_typescript_react_component_semantic_pairs() {
    println!("\n=== Debugging TypeScript React Component ===\n");

    let parser = TypeScriptParser::new();

    println!("Step 1: Parsing old code...");
    let old_tree = parser
        .try_parse(OLD_CODE)
        .expect("Failed to parse old TypeScript code");

    println!("Step 2: Parsing new code...");
    let new_tree = parser
        .try_parse(NEW_CODE)
        .expect("Failed to parse new TypeScript code");

    println!("Step 3: Building OLD semantic tree...");
    let old_semantic_tree = parser
        .build_semantic_tree(&old_tree, OLD_CODE)
        .expect("Failed to build old semantic tree");
    println!(
        "  Old semantic tree root: {:?}",
        old_semantic_tree.root.tree_sitter_node.kind()
    );
    println!(
        "  Old semantic tree children: {}",
        old_semantic_tree.root.children.len()
    );
    for (i, child) in old_semantic_tree.root.children.iter().enumerate() {
        println!("    Child {}: {:?}", i, child.tree_sitter_node.kind());
    }

    println!("\nStep 4: Building NEW semantic tree...");
    let new_semantic_tree = parser
        .build_semantic_tree(&new_tree, NEW_CODE)
        .expect("Failed to build new semantic tree");
    println!(
        "  New semantic tree root: {:?}",
        new_semantic_tree.root.tree_sitter_node.kind()
    );
    println!(
        "  New semantic tree children: {}",
        new_semantic_tree.root.children.len()
    );
    for (i, child) in new_semantic_tree.root.children.iter().enumerate() {
        println!("    Child {}: {:?}", i, child.tree_sitter_node.kind());
    }

    let old_source = SimpleSource {
        content: OLD_CODE.to_string(),
    };
    let new_source = SimpleSource {
        content: NEW_CODE.to_string(),
    };

    println!("\nStep 5: Building semantic pairs...");
    let pairs = build_semantic_pairs(
        &old_semantic_tree,
        &new_semantic_tree,
        &old_source,
        &new_source,
        &parser,
    )
    .expect("Failed to build semantic pairs");

    println!("  Semantic pairs produced: {}", pairs.len());
    if pairs.is_empty() {
        println!("\n  ❌ NO SEMANTIC PAIRS PRODUCED!");
        println!("\n  This is the bug we're investigating.");
    } else {
        println!("\n  ✓ Semantic pairs were produced!");
        println!("  Analyzing each pair:\n");

        for (i, pair) in pairs.iter().enumerate() {
            use diffviz_core::semantic_ast::SemanticPair;
            match pair {
                SemanticPair::Matched {
                    old_unit, new_unit, ..
                } => {
                    println!(
                        "  Pair #{}: Matched - {} -> {}",
                        i + 1,
                        old_unit.tree_sitter_node.kind(),
                        new_unit.tree_sitter_node.kind()
                    );
                }
                SemanticPair::Deletion { unit } => {
                    println!(
                        "  Pair #{}: Deletion - {} with {} children",
                        i + 1,
                        unit.tree_sitter_node.kind(),
                        unit.children.len()
                    );
                    println!("      Unit type: {:?}", unit.unit_type);
                    println!("      Byte range: {:?}", unit.tree_sitter_node.byte_range());
                    if !unit.children.is_empty() {
                        println!("      Children that were not matched:");
                        for child in &unit.children {
                            println!(
                                "        - {} @ {:?}",
                                child.tree_sitter_node.kind(),
                                child.tree_sitter_node.byte_range()
                            );
                        }
                    }
                }
                SemanticPair::Addition { unit } => {
                    println!(
                        "  Pair #{}: Addition - {} with {} children",
                        i + 1,
                        unit.tree_sitter_node.kind(),
                        unit.children.len()
                    );
                    println!("      Unit type: {:?}", unit.unit_type);
                    println!("      Byte range: {:?}", unit.tree_sitter_node.byte_range());
                    if !unit.children.is_empty() {
                        println!("      Children that were not matched:");
                        for child in &unit.children {
                            println!(
                                "        - {} @ {:?}",
                                child.tree_sitter_node.kind(),
                                child.tree_sitter_node.byte_range()
                            );
                        }
                    }
                }
            }
        }
    }

    println!("\nStep 6: Converting to ReviewableDiffs...");
    let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
        &pairs,
        ProgrammingLanguage::TypeScript,
        &old_source,
        &new_source,
        &parser,
    );

    println!("  ReviewableDiffs produced: {}", reviewable_diffs.len());
    if reviewable_diffs.is_empty() {
        println!("\n  ❌ NO REVIEWABLE DIFFS PRODUCED!");
        println!("  This is why the fixture test fails!");
    } else {
        println!("\n  ✓ ReviewableDiffs were produced!");
    }

    println!("\n=== Debug Complete ===\n");
}
