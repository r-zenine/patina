use diffviz_core::{
    LanguageParser,
    ast_diff::{LineRange, NodeLike, SourceError, SourceProvider},
    common::ProgrammingLanguage,
    parsers::cpp::CppParser,
    reviewable_diff_from_semantic::semantic_pairs_to_reviewable_diffs,
    semantic_ast::build_semantic_pairs,
};

#[cfg(test)]
mod cpp_semantic_pairing_tests {
    use super::*;

    #[test]
    fn test_template_class_evolution_pairing() {
        // This test captures the regression where template vs non-template class
        // evolution isn't properly matched, causing massive delete/add blocks
        let old_code = r#"
#include <iostream>
#include <vector>

class DataProcessor {
private:
    std::vector<int> data;
    
public:
    DataProcessor() = default;
    
    void addData(int value) {
        data.push_back(value);
    }
    
    void processData() {
        for (auto& item : data) {
            item *= 2;
        }
    }
    
    void printData() {
        for (const auto& item : data) {
            std::cout << item << " ";
        }
        std::cout << std::endl;
    }
};

enum Status {
    IDLE,
    PROCESSING,
    COMPLETE
};
"#;

        let new_code = r#"
#include <iostream>
#include <vector>
#include <algorithm>

template<typename T>
class DataProcessor {
private:
    std::vector<T> data;
    
public:
    DataProcessor() = default;
    
    void addData(const T& value) {
        data.push_back(value);
    }
    
    void processData() {
        std::transform(data.begin(), data.end(), data.begin(), 
                      [](const T& item) { return item * 2; });
    }
    
    void printData() const {
        for (const auto& item : data) {
            std::cout << item << " ";
        }
        std::cout << std::endl;
    }
};

enum class Status {
    IDLE,
    PROCESSING,
    COMPLETE
};
"#;

        let parser = CppParser::new();

        // Parse AST trees
        let old_tree = parser
            .try_parse(old_code)
            .expect("Failed to parse old C++ code");
        let new_tree = parser
            .try_parse(new_code)
            .expect("Failed to parse new C++ code");

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
                // Check if this is a TreeSitter node with position info
                if let Some(ts_node) = node.as_tree_sitter_node() {
                    let start_pos = ts_node.start_position();
                    let end_pos = ts_node.end_position();
                    LineRange {
                        start_line: start_pos.row + 1, // TreeSitter is 0-based, convert to 1-based
                        end_line: end_pos.row + 1,
                        start_column: start_pos.column,
                        end_column: end_pos.column,
                    }
                } else {
                    // Fall back to calculating from byte positions for OwnedNodeData
                    // For test simplicity, just create a minimal LineRange
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

        // Convert to reviewable diffs
        let reviewable_diffs = semantic_pairs_to_reviewable_diffs(
            &semantic_pairs,
            ProgrammingLanguage::Cpp,
            &old_source,
            &new_source,
        );

        // Test that we're getting proper semantic pairing
        // We should have focused semantic changes instead of massive delete/add blocks

        // Analyze the semantic pairs to ensure proper matching
        let matched_pairs = semantic_pairs
            .iter()
            .filter(|pair| pair.should_diff())
            .count();

        let total_pairs = semantic_pairs.len();

        // We should have some matched pairs that indicate successful semantic pairing
        assert!(
            matched_pairs > 0,
            "No matched semantic pairs found - indicates pairing failure"
        );

        // The key success metric: we should have proper semantic matches
        // rather than everything being treated as separate add/delete operations
        let match_ratio = matched_pairs as f64 / total_pairs as f64;
        assert!(
            match_ratio >= 0.3, // At least 30% of pairs should be matched
            "Low match ratio ({match_ratio:.2}): {matched_pairs} matched out of {total_pairs} total pairs - indicates poor semantic matching"
        );

        // Verify we have reasonable number of reviewable diffs
        assert!(
            reviewable_diffs.len() <= 10,
            "Too many reviewable diffs ({}), indicates poor semantic matching",
            reviewable_diffs.len()
        );
    }

    #[test]
    fn test_enum_to_enum_class_pairing() {
        // Specific test for enum -> enum class transformation
        let old_code = r#"
enum Status {
    IDLE,
    PROCESSING,
    COMPLETE
};
"#;

        let new_code = r#"
enum class Status {
    IDLE,
    PROCESSING,
    COMPLETE
};
"#;

        let parser = CppParser::new();

        // Parse AST trees
        let old_tree = parser
            .try_parse(old_code)
            .expect("Failed to parse old C++ enum code");
        let new_tree = parser
            .try_parse(new_code)
            .expect("Failed to parse new C++ enum class code");

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
                // Check if this is a TreeSitter node with position info
                if let Some(ts_node) = node.as_tree_sitter_node() {
                    let start_pos = ts_node.start_position();
                    let end_pos = ts_node.end_position();
                    LineRange {
                        start_line: start_pos.row + 1, // TreeSitter is 0-based, convert to 1-based
                        end_line: end_pos.row + 1,
                        start_column: start_pos.column,
                        end_column: end_pos.column,
                    }
                } else {
                    // Fall back to calculating from byte positions for OwnedNodeData
                    // For test simplicity, just create a minimal LineRange
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

        // Should be recognized as a matched pair, not separate delete + add
        let matched_pairs = semantic_pairs
            .iter()
            .filter(|pair| pair.should_diff())
            .count();

        assert_eq!(
            matched_pairs, 1,
            "Enum to enum class should be a single matched pair"
        );

        // Verify it's actually a semantic match, not just add/delete
        assert!(
            semantic_pairs.iter().any(|pair| matches!(
                pair,
                diffviz_core::semantic_ast::SemanticPair::Matched { .. }
            )),
            "Enum to enum class should be a matched pair, not separate add/delete"
        );
    }
}
