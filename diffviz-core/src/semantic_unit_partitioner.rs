//! Semantic Unit Partitioning for Large-Scale AST Diffs
//!
//! This module implements a preprocessing step that partitions AST trees into pairs
//! of comparable semantic units before applying the existing diff algorithm.
//!
//! ## Problem
//! The existing diff algorithm works well for bounded semantic units (comparing two
//! versions of the same function) but breaks when comparing full files with major
//! structural changes, producing fragmented meaningless boundaries.
//!
//! ## Solution  
//! 1. Extract meaningful semantic units from both AST trees (functions, structs, modules)
//! 2. Form pairs by name matching, structural similarity, and change detection
//! 3. Apply existing diff algorithm only to comparable pairs
//! 4. Handle additions/deletions as separate change types
//!
//! ## Usage
//! ```ignore
//! // Example usage (this is now deprecated in favor of semantic_ast)
//! let pairs = partition_ast_trees(&old_tree, &new_tree, old_source, new_source, language)?;
//! for pair in pairs {
//!     match pair {
//!         UnitPair::Exact { old_unit, new_unit } => {
//!             // Apply diff algorithm to this pair
//!         }
//!         UnitPair::Addition { unit } => {
//!             // Handle as pure addition
//!         }
//!         // ... handle other pair types
//!     }
//! }
//! ```

use crate::common::ProgrammingLanguage;
use tree_sitter::{Node, Tree};

/// A semantic unit extracted from an AST tree
///
/// Represents a meaningful language construct that can be compared
/// across different versions of a file.
#[derive(Debug, Clone)]
pub struct SemanticUnit<'a> {
    /// Unique identifier for this unit (e.g., "main", "Cli", "Commands::Show")
    pub name: String,

    /// The AST node representing this semantic unit
    pub node: Node<'a>,

    /// The type of semantic unit this represents
    pub unit_type: SemanticUnitType,

    /// Parent context for hierarchical naming (e.g., struct name for methods)
    pub parent_context: Option<String>,
}

impl<'a> SemanticUnit<'a> {
    /// Create a new semantic unit
    pub fn new(
        name: String,
        node: Node<'a>,
        unit_type: SemanticUnitType,
        parent_context: Option<String>,
    ) -> Self {
        Self {
            name,
            node,
            unit_type,
            parent_context,
        }
    }

    /// Get the fully qualified name including parent context
    pub fn qualified_name(&self) -> String {
        match &self.parent_context {
            Some(parent) => format!("{}::{}", parent, self.name),
            None => self.name.clone(),
        }
    }

    /// Check if this unit represents the same semantic construct as another
    pub fn is_same_construct(&self, other: &SemanticUnit) -> bool {
        self.qualified_name() == other.qualified_name() && self.unit_type == other.unit_type
    }

    /// Calculate structural similarity with another unit (0.0 to 1.0)
    pub fn structural_similarity(&self, other: &SemanticUnit) -> f32 {
        if self.unit_type != other.unit_type {
            return 0.0;
        }

        // Basic structural similarity based on node characteristics
        let same_kind = if self.node.kind() == other.node.kind() {
            0.4
        } else {
            0.0
        };
        let child_similarity = self.calculate_child_similarity(other);

        same_kind + child_similarity * 0.6
    }

    fn calculate_child_similarity(&self, other: &SemanticUnit) -> f32 {
        let self_children = self.node.child_count();
        let other_children = other.node.child_count();

        if self_children == 0 && other_children == 0 {
            return 1.0;
        }

        let max_children = self_children.max(other_children) as f32;
        let min_children = self_children.min(other_children) as f32;

        min_children / max_children
    }
}

/// Types of semantic units that can be extracted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticUnitType {
    /// Function or method definition
    Function,
    /// Struct definition
    Struct,
    /// Enum definition
    Enum,
    /// Module declaration
    Module,
    /// Import/use statement group
    ImportGroup,
    /// Type alias or constant
    TypeAlias,
    /// Trait definition
    Trait,
    /// Implementation block
    Implementation,
}

/// Pairing result for semantic units between old and new trees
#[derive(Debug, Clone)]
pub enum UnitPair<'a> {
    /// Exact match by name and type - suitable for detailed diffing
    Exact {
        old_unit: SemanticUnit<'a>,
        new_unit: SemanticUnit<'a>,
    },

    /// Structural match indicating a potential rename or refactor
    Structural {
        old_unit: SemanticUnit<'a>,
        new_unit: SemanticUnit<'a>,
        similarity_score: f32,
    },

    /// Unit that exists only in the new version (addition)
    Addition { unit: SemanticUnit<'a> },

    /// Unit that exists only in the old version (deletion)
    Deletion { unit: SemanticUnit<'a> },
}

impl<'a> UnitPair<'a> {
    /// Check if this pair represents a change that should be diffed
    pub fn should_diff(&self) -> bool {
        matches!(self, UnitPair::Exact { .. } | UnitPair::Structural { .. })
    }

    /// Get the old unit if this pair has one
    pub fn old_unit(&self) -> Option<&SemanticUnit<'a>> {
        match self {
            UnitPair::Exact { old_unit, .. } | UnitPair::Structural { old_unit, .. } => {
                Some(old_unit)
            }
            UnitPair::Deletion { unit } => Some(unit),
            UnitPair::Addition { .. } => None,
        }
    }

    /// Get the new unit if this pair has one  
    pub fn new_unit(&self) -> Option<&SemanticUnit<'a>> {
        match self {
            UnitPair::Exact { new_unit, .. } | UnitPair::Structural { new_unit, .. } => {
                Some(new_unit)
            }
            UnitPair::Addition { unit } => Some(unit),
            UnitPair::Deletion { .. } => None,
        }
    }
}

/// Trait for language-specific semantic unit extraction
pub trait SemanticUnitExtractor {
    /// Extract semantic units from an AST tree
    fn extract_units<'a>(&self, tree: &'a Tree, source: &str) -> Vec<SemanticUnit<'a>>;
}

/// Configuration for the partitioning algorithm
#[derive(Debug, Clone)]
pub struct PartitioningConfig {
    /// Minimum similarity threshold for structural matching (0.0 to 1.0)
    pub similarity_threshold: f32,

    /// Whether to enable structural matching for potential renames
    pub enable_structural_matching: bool,

    /// Whether to group related units together (e.g., struct + its impl blocks)
    pub group_related_units: bool,
}

impl Default for PartitioningConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            enable_structural_matching: true,
            group_related_units: true,
        }
    }
}

/// Main entry point: Partition two AST trees into comparable semantic unit pairs
///
/// This function orchestrates the complete partitioning pipeline:
/// 1. Extract semantic units from both trees
/// 2. Perform exact name matching
/// 3. Perform structural similarity matching for renames
/// 4. Mark remaining units as additions/deletions
pub fn partition_ast_trees<'a>(
    old_tree: &'a Tree,
    new_tree: &'a Tree,
    old_source: &str,
    new_source: &str,
    language: ProgrammingLanguage,
    config: Option<PartitioningConfig>,
) -> Result<Vec<UnitPair<'a>>, PartitioningError> {
    let config = config.unwrap_or_default();

    // Get language-specific extractor
    let extractor = get_extractor_for_language(language)?;

    // Extract semantic units from both trees
    let old_units = extractor.extract_units(old_tree, old_source);
    let new_units = extractor.extract_units(new_tree, new_source);

    // Create pairing algorithm and execute
    let pairing_algorithm = SemanticUnitPairingAlgorithm::new(config);
    pairing_algorithm.pair_units(old_units, new_units)
}

/// Algorithm for pairing semantic units between old and new versions
struct SemanticUnitPairingAlgorithm {
    config: PartitioningConfig,
}

impl SemanticUnitPairingAlgorithm {
    fn new(config: PartitioningConfig) -> Self {
        Self { config }
    }

    /// Execute the complete pairing algorithm
    fn pair_units<'a>(
        &self,
        old_units: Vec<SemanticUnit<'a>>,
        new_units: Vec<SemanticUnit<'a>>,
    ) -> Result<Vec<UnitPair<'a>>, PartitioningError> {
        let mut pairs = Vec::new();
        let mut used_old = vec![false; old_units.len()];
        let mut used_new = vec![false; new_units.len()];

        // Phase 1: Exact matching by qualified name
        self.exact_matching(
            &old_units,
            &new_units,
            &mut pairs,
            &mut used_old,
            &mut used_new,
        );

        // Phase 2: Structural matching for potential renames (if enabled)
        if self.config.enable_structural_matching {
            self.structural_matching(
                &old_units,
                &new_units,
                &mut pairs,
                &mut used_old,
                &mut used_new,
            );
        }

        // Phase 3: Mark remaining units as additions/deletions
        self.mark_additions_deletions(&old_units, &new_units, &mut pairs, &used_old, &used_new);

        Ok(pairs)
    }

    fn exact_matching<'a>(
        &self,
        old_units: &[SemanticUnit<'a>],
        new_units: &[SemanticUnit<'a>],
        pairs: &mut Vec<UnitPair<'a>>,
        used_old: &mut [bool],
        used_new: &mut [bool],
    ) {
        for (old_idx, old_unit) in old_units.iter().enumerate() {
            if used_old[old_idx] {
                continue;
            }

            for (new_idx, new_unit) in new_units.iter().enumerate() {
                if used_new[new_idx] {
                    continue;
                }

                if old_unit.is_same_construct(new_unit) {
                    pairs.push(UnitPair::Exact {
                        old_unit: old_unit.clone(),
                        new_unit: new_unit.clone(),
                    });
                    used_old[old_idx] = true;
                    used_new[new_idx] = true;
                    break;
                }
            }
        }
    }

    fn structural_matching<'a>(
        &self,
        old_units: &[SemanticUnit<'a>],
        new_units: &[SemanticUnit<'a>],
        pairs: &mut Vec<UnitPair<'a>>,
        used_old: &mut [bool],
        used_new: &mut [bool],
    ) {
        let mut potential_matches = Vec::new();

        // Find all potential structural matches above threshold
        for (old_idx, old_unit) in old_units.iter().enumerate() {
            if used_old[old_idx] {
                continue;
            }

            for (new_idx, new_unit) in new_units.iter().enumerate() {
                if used_new[new_idx] {
                    continue;
                }

                let similarity = old_unit.structural_similarity(new_unit);
                if similarity >= self.config.similarity_threshold {
                    potential_matches.push((old_idx, new_idx, similarity));
                }
            }
        }

        // Sort by similarity (highest first) and greedily match
        potential_matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        for (old_idx, new_idx, similarity) in potential_matches {
            if used_old[old_idx] || used_new[new_idx] {
                continue;
            }

            pairs.push(UnitPair::Structural {
                old_unit: old_units[old_idx].clone(),
                new_unit: new_units[new_idx].clone(),
                similarity_score: similarity,
            });
            used_old[old_idx] = true;
            used_new[new_idx] = true;
        }
    }

    fn mark_additions_deletions<'a>(
        &self,
        old_units: &[SemanticUnit<'a>],
        new_units: &[SemanticUnit<'a>],
        pairs: &mut Vec<UnitPair<'a>>,
        used_old: &[bool],
        used_new: &[bool],
    ) {
        // Mark unused old units as deletions
        for (old_idx, old_unit) in old_units.iter().enumerate() {
            if !used_old[old_idx] {
                pairs.push(UnitPair::Deletion {
                    unit: old_unit.clone(),
                });
            }
        }

        // Mark unused new units as additions
        for (new_idx, new_unit) in new_units.iter().enumerate() {
            if !used_new[new_idx] {
                pairs.push(UnitPair::Addition {
                    unit: new_unit.clone(),
                });
            }
        }
    }
}

/// Errors that can occur during semantic unit partitioning
#[derive(Debug)]
pub enum PartitioningError {
    /// Unsupported programming language
    UnsupportedLanguage(ProgrammingLanguage),
    /// AST parsing failed
    ParseError(String),
    /// Unit extraction failed
    ExtractionError(String),
}

impl std::fmt::Display for PartitioningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitioningError::UnsupportedLanguage(lang) => {
                write!(
                    f,
                    "Unsupported language for semantic partitioning: {lang:?}"
                )
            }
            PartitioningError::ParseError(msg) => write!(f, "AST parse error: {msg}"),
            PartitioningError::ExtractionError(msg) => write!(f, "Unit extraction error: {msg}"),
        }
    }
}

impl std::error::Error for PartitioningError {}

/// Get the appropriate semantic unit extractor for a programming language
fn get_extractor_for_language(
    language: ProgrammingLanguage,
) -> Result<Box<dyn SemanticUnitExtractor>, PartitioningError> {
    match language {
        ProgrammingLanguage::Rust => Ok(Box::new(RustSemanticUnitExtractor::new())),
        // TODO: Add support for other languages
        _ => Err(PartitioningError::UnsupportedLanguage(language)),
    }
}

/// Rust-specific semantic unit extractor
struct RustSemanticUnitExtractor {
    // Future: Could hold Rust-specific configuration
}

impl RustSemanticUnitExtractor {
    fn new() -> Self {
        Self {}
    }

    /// Extract the name from various Rust AST node types
    fn extract_name_from_node(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "function_item" => {
                // Look for identifier child
                node.child_by_field_name("name")
                    .and_then(|name_node| self.node_text(name_node, source))
            }
            "struct_item" | "enum_item" | "trait_item" => {
                // Look for type identifier
                node.child_by_field_name("name")
                    .or_else(|| {
                        node.children(&mut node.walk())
                            .find(|c| c.kind() == "type_identifier")
                    })
                    .and_then(|name_node| self.node_text(name_node, source))
            }
            "mod_item" => {
                // Module name
                node.child_by_field_name("name")
                    .and_then(|name_node| self.node_text(name_node, source))
            }
            "impl_item" => {
                // For impl blocks, use the type being implemented
                node.child_by_field_name("type")
                    .and_then(|type_node| self.node_text(type_node, source))
                    .map(|type_name| format!("impl {type_name}"))
            }
            "use_declaration" => {
                // For use statements, extract the root module
                self.extract_use_root(node, source)
            }
            _ => None,
        }
    }

    /// Extract the root module from a use declaration
    fn extract_use_root(&self, node: Node, source: &str) -> Option<String> {
        // Find the use_list or scoped_identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "use_list" || child.kind() == "scoped_identifier" {
                // Get the first part of the path
                if let Some(first_part) = child.children(&mut child.walk()).next() {
                    if first_part.kind() == "identifier" {
                        return self.node_text(first_part, source);
                    }
                }
            }
        }
        None
    }

    /// Get text content of a node
    fn node_text(&self, node: Node, source: &str) -> Option<String> {
        let start = node.start_byte();
        let end = node.end_byte();
        source.get(start..end).map(|s| s.to_string())
    }
}

impl SemanticUnitExtractor for RustSemanticUnitExtractor {
    fn extract_units<'a>(&self, tree: &'a Tree, source: &str) -> Vec<SemanticUnit<'a>> {
        let mut units = Vec::new();
        let root = tree.root_node();

        self.extract_units_recursive(root, source, None, &mut units);
        units
    }
}

impl RustSemanticUnitExtractor {
    fn extract_units_recursive<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        parent_context: Option<String>,
        units: &mut Vec<SemanticUnit<'a>>,
    ) {
        // Check if this node represents a semantic unit
        let unit_type = match node.kind() {
            "function_item" => Some(SemanticUnitType::Function),
            "struct_item" => Some(SemanticUnitType::Struct),
            "enum_item" => Some(SemanticUnitType::Enum),
            "mod_item" => Some(SemanticUnitType::Module),
            "trait_item" => Some(SemanticUnitType::Trait),
            "impl_item" => Some(SemanticUnitType::Implementation),
            "use_declaration" => Some(SemanticUnitType::ImportGroup),
            "type_item" => Some(SemanticUnitType::TypeAlias),
            _ => None,
        };

        if let Some(unit_type) = unit_type {
            if let Some(name) = self.extract_name_from_node(node, source) {
                let semantic_unit =
                    SemanticUnit::new(name, node, unit_type, parent_context.clone());
                units.push(semantic_unit);
            }
        }

        // Recurse into children, updating parent context for nested structures
        let new_parent_context = if matches!(
            node.kind(),
            "struct_item" | "enum_item" | "trait_item" | "impl_item"
        ) {
            self.extract_name_from_node(node, source).or(parent_context)
        } else {
            parent_context
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_units_recursive(child, source, new_parent_context.clone(), units);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::RustParser;
    use tree_sitter::Parser;

    /// Create a Rust parser for testing
    fn create_rust_parser() -> Parser {
        use crate::common::LanguageParser;

        let mut parser = Parser::new();
        let rust_parser = RustParser::new();
        parser.set_language(rust_parser.get_language()).unwrap();
        parser
    }

    #[test]
    fn test_semantic_unit_qualified_name() {
        let mut parser = create_rust_parser();
        let code = "fn test() {}";
        let tree = parser.parse(code, None).unwrap();
        let node = tree.root_node();

        let unit = SemanticUnit::new("test".to_string(), node, SemanticUnitType::Function, None);
        assert_eq!(unit.qualified_name(), "test");

        let unit_with_context = SemanticUnit::new(
            "method".to_string(),
            node,
            SemanticUnitType::Function,
            Some("MyStruct".to_string()),
        );
        assert_eq!(unit_with_context.qualified_name(), "MyStruct::method");
    }

    #[test]
    fn test_semantic_unit_same_construct() {
        let mut parser = create_rust_parser();
        let code = "fn test() {}";
        let tree = parser.parse(code, None).unwrap();
        let node = tree.root_node();

        let unit1 = SemanticUnit::new("test".to_string(), node, SemanticUnitType::Function, None);

        let unit2 = SemanticUnit::new("test".to_string(), node, SemanticUnitType::Function, None);

        let unit3 = SemanticUnit::new("other".to_string(), node, SemanticUnitType::Function, None);

        assert!(unit1.is_same_construct(&unit2));
        assert!(!unit1.is_same_construct(&unit3));
    }

    #[test]
    fn test_unit_pair_should_diff() {
        let mut parser = create_rust_parser();
        let code = "fn test() {}";
        let tree = parser.parse(code, None).unwrap();
        let node = tree.root_node();

        let unit = SemanticUnit::new("test".to_string(), node, SemanticUnitType::Function, None);

        let exact_pair = UnitPair::Exact {
            old_unit: unit.clone(),
            new_unit: unit.clone(),
        };
        assert!(exact_pair.should_diff());

        let structural_pair = UnitPair::Structural {
            old_unit: unit.clone(),
            new_unit: unit.clone(),
            similarity_score: 0.8,
        };
        assert!(structural_pair.should_diff());

        let addition_pair = UnitPair::Addition { unit: unit.clone() };
        assert!(!addition_pair.should_diff());

        let deletion_pair = UnitPair::Deletion { unit };
        assert!(!deletion_pair.should_diff());
    }

    // TODO: Add integration tests once we have the main.rs test data
}
