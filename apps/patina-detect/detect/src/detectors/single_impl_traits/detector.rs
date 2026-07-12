use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{FileLocation, Location, LspClient, Position};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

pub const DETECTOR_ID: &str = "single-impl-traits";

#[derive(Debug, Error)]
pub enum SingleImplTraitsError {
    #[error("failed to resolve {path} to an absolute path")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to walk directory {path}")]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },

    #[error("failed to read file {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to configure tree-sitter Rust grammar")]
    Language(#[from] tree_sitter::LanguageError),

    #[error("failed to parse {path} as Rust")]
    Parse { path: PathBuf },

    #[error("language server error")]
    Lsp(#[from] lspkit::Error),
}

/// One trait declaration found while scanning, surviving the mechanical
/// exclusion list (marker traits, sealed traits, traits declared inside
/// test code) — spec.md:255-258.
struct TraitCandidate {
    file: PathBuf,
    qualified_name: String,
    /// 0-based tree-sitter position of the trait's own name identifier,
    /// the position `lspkit::LspClient::implementations` is queried
    /// against.
    name_point: Point,
    line_range: LineRange,
}

/// Runs the single-impl-traits detector (spec.md:250-259, Phase 15).
/// Tree-sitter enumerates every trait declaration, excluding marker traits
/// (empty body), sealed traits (a supertrait bound naming `Sealed`), and
/// traits declared inside test code. Each remaining candidate is checked
/// via a real `rust-analyzer` process (`lspkit::LspClient::implementations`)
/// for its implementor count: a candidate is reported only when exactly one
/// production implementor exists and no test-double implementor
/// accompanies it — a trait with a test-double impl alongside its one
/// production impl is the legitimate Environment/DI pattern, not
/// speculative generality (spec.md's explicit FP control).
pub fn run_single_impl_traits(root: &Path) -> Result<Vec<Symptom>, SingleImplTraitsError> {
    // `LspClient::start` builds a `file://` URI directly from `root`, which
    // is malformed for a relative path — canonicalize once up front so
    // every downstream file path is absolute (mirrors `dead_exports`).
    let root = fs::canonicalize(root).map_err(|source| SingleImplTraitsError::Canonicalize {
        path: root.to_path_buf(),
        source,
    })?;
    let root = root.as_path();

    let mut candidates = Vec::new();
    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| SingleImplTraitsError::Read {
            path: file.clone(),
            source,
        })?;
        let tree = parse_rust(&content, &file)?;
        collect_traits(tree.root_node(), content.as_bytes(), &file, &mut candidates);
    }

    let client = LspClient::start(root)?;
    let mut file_cache: HashMap<PathBuf, (String, tree_sitter::Tree)> = HashMap::new();
    let mut symptoms = Vec::new();

    for candidate in &candidates {
        let position = Position {
            line: candidate.name_point.row as u32 + 1,
            character: candidate.name_point.column as u32 + 1,
        };
        let at = FileLocation {
            path: candidate.file.clone(),
            position,
        };
        // A single candidate's `implementations()` call failing (e.g. a
        // dependency's stale rust-analyzer-side build metadata, unrelated
        // to the candidate itself — see `dead_exports::detector`'s
        // identical fix) must not abort every other candidate's evidence
        // gathering. Skip and continue rather than `?`.
        let implementations = match client.implementations(&at) {
            Ok(locations) => locations,
            Err(err) => {
                eprintln!(
                    "single-impl-traits: skipping {} — implementations() failed: {err}",
                    candidate.qualified_name
                );
                continue;
            }
        };

        let (production, test_doubles) =
            match partition_by_test_context(&implementations, &mut file_cache) {
                Ok(partitioned) => partitioned,
                Err(err) => {
                    eprintln!(
                        "single-impl-traits: skipping {} — failed to classify implementors: {err}",
                        candidate.qualified_name
                    );
                    continue;
                }
            };

        if production.len() != 1 || !test_doubles.is_empty() {
            continue;
        }
        let sole_implementor = &production[0];
        let implementor_relative = sole_implementor
            .path
            .strip_prefix(root)
            .unwrap_or(sole_implementor.path.as_path())
            .to_path_buf();

        let relative = candidate
            .file
            .strip_prefix(root)
            .unwrap_or(candidate.file.as_path())
            .to_path_buf();
        let id = SymptomId::new(
            DetectorId::new(DETECTOR_ID),
            candidate.qualified_name.as_bytes(),
        );

        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: format!("Single-impl trait: {}", candidate.qualified_name),
            evidence: Evidence::SingleImplTrait {
                trait_name: candidate.qualified_name.clone(),
                impl_count: production.len(),
                impl_locations: vec![location_label(root, sole_implementor)],
                test_doubles_present: false,
            },
            // `Location.range` from lspkit is already 1-based
            // (`native.rs::from_lsp_position` converts on the way in),
            // matching `LineRange`'s own convention — no adjustment needed
            // here, unlike the 0-based tree-sitter `Point`s used elsewhere
            // in this file.
            sites: vec![
                Site {
                    file: relative,
                    line_ranges: vec![candidate.line_range],
                    role: SiteRole::MatchSite,
                    note: "Trait declaration with exactly one implementor".to_string(),
                },
                Site {
                    file: implementor_relative,
                    line_ranges: vec![
                        enclosing_impl_line_range(sole_implementor, &file_cache).unwrap_or(
                            LineRange {
                                start: sole_implementor.range.start.line as usize,
                                end: sole_implementor.range.end.line as usize,
                            },
                        ),
                    ],
                    role: SiteRole::Definition,
                    note: "Sole implementor of this trait".to_string(),
                },
            ],
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

/// `implementations()` locations point at the implementor type's name
/// identifier, not the surrounding `impl` block — rendering that bare
/// single-line range fails (`create_reviewable_diff_from_range` requires a
/// complete semantic unit). Walk up to the enclosing `impl_item` so the
/// site covers the whole block, the way `collect_traits` already does for
/// the trait declaration itself.
fn enclosing_impl_line_range(
    location: &Location,
    file_cache: &HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Option<LineRange> {
    let (_, tree) = file_cache.get(&location.path)?;
    let point = Point {
        row: (location.range.start.line as usize).saturating_sub(1),
        column: (location.range.start.character as usize).saturating_sub(1),
    };
    let mut node = tree.root_node().descendant_for_point_range(point, point)?;
    while node.kind() != "impl_item" {
        node = node.parent()?;
    }
    Some(LineRange {
        start: node.start_position().row + 1,
        end: node.end_position().row + 1,
    })
}

fn location_label(root: &Path, location: &Location) -> String {
    let relative = location
        .path
        .strip_prefix(root)
        .unwrap_or(location.path.as_path());
    format!("{}:{}", relative.display(), location.range.start.line)
}

/// Splits `implementations()`'s results into production implementors and
/// test-double implementors (an implementor whose declaration site sits
/// inside `#[cfg(test)]`/`#[test]` context) — the DI/Environment-pattern FP
/// control (spec.md:255-257).
fn partition_by_test_context(
    locations: &[Location],
    file_cache: &mut HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Result<(Vec<Location>, Vec<Location>), SingleImplTraitsError> {
    let mut production = Vec::new();
    let mut test_doubles = Vec::new();
    for location in locations {
        let point = Point {
            row: (location.range.start.line as usize).saturating_sub(1),
            column: (location.range.start.character as usize).saturating_sub(1),
        };
        if is_location_test_only(&location.path, point, file_cache)? {
            test_doubles.push(location.clone());
        } else {
            production.push(location.clone());
        }
    }
    Ok((production, test_doubles))
}

fn is_location_test_only(
    path: &Path,
    point: Point,
    file_cache: &mut HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Result<bool, SingleImplTraitsError> {
    if !file_cache.contains_key(path) {
        let content = fs::read_to_string(path).map_err(|source| SingleImplTraitsError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let tree = parse_rust(&content, path)?;
        file_cache.insert(path.to_path_buf(), (content, tree));
    }
    let (content, tree) = file_cache.get(path).expect("just inserted above");
    let node = tree.root_node().descendant_for_point_range(point, point);
    Ok(node.is_some_and(|n| is_in_test_context(n, content.as_bytes())))
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, SingleImplTraitsError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| SingleImplTraitsError::Parse {
            path: path.to_path_buf(),
        })
}

/// Walks `node`'s subtree collecting one `TraitCandidate` per `trait_item`,
/// applying spec.md:255-258's mechanical exclusion list: a trait declared
/// inside test code is skipped outright (it exists only for test
/// scaffolding, e.g. a trait used solely via `dyn`/generic bounds inside
/// `#[cfg(test)]` code — never a production speculative-generality
/// candidate); an empty-body trait is a marker trait; a trait whose
/// supertrait bounds name `Sealed` is the sealed-trait pattern (a
/// deliberate, not speculative, single-impl-per-crate design).
fn collect_traits(node: Node, source: &[u8], file: &Path, out: &mut Vec<TraitCandidate>) {
    if node.kind() == "trait_item"
        && !is_in_test_context(node, source)
        && !is_marker_trait(node)
        && !is_sealed_trait(node, source)
        && let Some(name_node) = node.child_by_field_name("name")
    {
        out.push(TraitCandidate {
            file: file.to_path_buf(),
            qualified_name: qualified_name(node, source),
            name_point: name_node.start_position(),
            line_range: LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            },
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_traits(child, source, file, out);
    }
}

/// A marker trait has an empty `declaration_list` body — no methods,
/// associated types, or constants (spec.md:257's "marker/sealed traits"
/// exclusion). Recognizing it via structure rather than naming convention
/// avoids missing marker traits that don't happen to be named `*Marker`.
fn is_marker_trait(node: Node) -> bool {
    node.child_by_field_name("body")
        .is_some_and(|body| body.named_child_count() == 0)
}

/// The sealed-trait pattern names its sealing supertrait `Sealed` by
/// convention (e.g. `trait Public: private::Sealed`) — a deliberate,
/// intentional single-crate-impl design, not speculative generality.
fn is_sealed_trait(node: Node, source: &[u8]) -> bool {
    node.child_by_field_name("bounds").is_some_and(|bounds| {
        bounds
            .utf8_text(source)
            .is_ok_and(|text| text.contains("Sealed"))
    })
}

/// Whether `node` (or any ancestor) is immediately preceded by a `#[test]`
/// or `#[cfg(test)]` attribute — covers both a directly-annotated `#[test]`
/// function and a declaration nested inside a `#[cfg(test)] mod tests { }`.
fn is_in_test_context(node: Node, source: &[u8]) -> bool {
    let mut current = node;
    loop {
        if has_preceding_attribute(current, source, "test") {
            return true;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

fn has_preceding_attribute(node: Node, source: &[u8], keyword: &str) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    let mut cursor = parent.walk();
    let siblings: Vec<Node> = parent.children(&mut cursor).collect();
    let Some(index) = siblings.iter().position(|s| s.id() == node.id()) else {
        return false;
    };
    for sibling in siblings[..index].iter().rev() {
        if sibling.kind() != "attribute_item" {
            break;
        }
        if sibling
            .utf8_text(source)
            .is_ok_and(|text| text.contains(keyword))
        {
            return true;
        }
    }
    false
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per `dead_exports`/`data_clumps`'s precedent rather
/// than depending on `diffviz-core`'s `SemanticTree` pipeline.
fn qualified_name(node: Node, source: &[u8]) -> String {
    let mut parts = Vec::new();
    let mut current = node.parent();
    while let Some(p) = current {
        let name = match p.kind() {
            "mod_item" | "trait_item" => p.child_by_field_name("name"),
            "impl_item" => p.child_by_field_name("type"),
            _ => None,
        };
        if let Some(name) = name.and_then(|n| n.utf8_text(source).ok()) {
            parts.push(name.to_string());
        }
        current = p.parent();
    }
    parts.reverse();

    let own_name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("<unknown>");
    parts.push(own_name.to_string());
    parts.join("::")
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, SingleImplTraitsError> {
    let mut files = Vec::new();
    let mut builder = ignore::WalkBuilder::new(root);
    builder.add_custom_ignore_filename(crate::detectors::IGNORE_FILE_NAME);
    for entry in builder.build() {
        let entry = entry.map_err(|source| SingleImplTraitsError::Walk {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") && path.is_file() {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn traits_for(content: &str) -> Vec<TraitCandidate> {
        let file = Path::new("f.rs");
        let tree = parse_rust(content, file).unwrap();
        let mut out = Vec::new();
        collect_traits(tree.root_node(), content.as_bytes(), file, &mut out);
        out
    }

    fn names(candidates: &[TraitCandidate]) -> Vec<&str> {
        candidates
            .iter()
            .map(|c| c.qualified_name.as_str())
            .collect()
    }

    #[test]
    fn a_plain_pub_trait_is_a_candidate() {
        let candidates = traits_for("pub trait Greeter { fn greet(&self) -> String; }");
        assert_eq!(names(&candidates), vec!["Greeter"]);
    }

    #[test]
    fn an_empty_body_trait_is_excluded_as_a_marker_trait() {
        let candidates = traits_for("pub trait Marker {}");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_sealed_supertrait_bound_is_excluded() {
        let candidates = traits_for(
            "mod private { pub trait Sealed {} }\n\
             pub trait Public: private::Sealed { fn method(&self); }",
        );
        assert!(names(&candidates).iter().all(|n| !n.contains("Public")));
    }

    #[test]
    fn a_trait_declared_inside_a_cfg_test_module_is_excluded() {
        let candidates =
            traits_for("#[cfg(test)]\nmod tests {\n    pub trait Helper { fn help(&self); }\n}");
        assert!(candidates.is_empty());
    }

    #[test]
    fn qualified_name_includes_module_path() {
        let candidates =
            traits_for("pub mod outer {\n    pub trait Inner { fn method(&self); }\n}");
        assert_eq!(names(&candidates), vec!["outer::Inner"]);
    }
}
