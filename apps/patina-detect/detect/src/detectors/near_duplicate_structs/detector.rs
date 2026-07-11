use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{FileLocation, Location, LspClient, Position};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

/// Minimum count and ratio of shared normalized `(name, type)` fields
/// before a struct pair is even considered (spec.md:198-199).
const MIN_SHARED_FIELDS: usize = 4;
const MIN_JACCARD: f64 = 0.7;

pub const DETECTOR_ID: &str = "near-duplicate-structs";

#[derive(Debug, Error)]
pub enum NearDuplicateStructsError {
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

/// One named-field struct found while scanning, with its normalized field
/// multiset ready for Jaccard comparison against every other candidate.
struct StructCandidate {
    file: PathBuf,
    qualified_name: String,
    /// 0-based tree-sitter position of the struct's own name identifier,
    /// the position `lspkit::LspClient::references` is queried against.
    name_point: Point,
    line_range: LineRange,
    fields: Vec<(String, String)>,
}

/// Runs the near-duplicate-data-structures detector (spec.md:194-210,
/// Phase 11). Tree-sitter enumerates every named-field struct's normalized
/// `(name, type)` field multiset; pairs whose Jaccard similarity is
/// `>= 0.7` with `>= 4` shared fields become candidates. Each candidate
/// pair is promoted only once `lspkit::LspClient::references` (reused from
/// Phase 7, no new lspkit method) finds real conversion code — a function
/// or `impl From` block referencing both structs — between them (the
/// conversion-evidence gate, spec.md's FP control against "legitimate
/// pairs... that mostly lack hand-rolled conversion churn").
pub fn run_near_duplicate_structs(root: &Path) -> Result<Vec<Symptom>, NearDuplicateStructsError> {
    // `LspClient::start` builds a `file://` URI directly from `root`, which
    // is malformed for a relative path — canonicalize once up front so
    // every downstream file path is absolute (mirrors `dead_exports`).
    let root =
        fs::canonicalize(root).map_err(|source| NearDuplicateStructsError::Canonicalize {
            path: root.to_path_buf(),
            source,
        })?;
    let root = root.as_path();

    let mut candidates = Vec::new();
    for file in collect_rust_files(root)? {
        let content =
            fs::read_to_string(&file).map_err(|source| NearDuplicateStructsError::Read {
                path: file.clone(),
                source,
            })?;
        let tree = parse_rust(&content, &file)?;
        collect_structs(
            tree.root_node(),
            content.as_bytes(),
            &file,
            root,
            &mut candidates,
        );
    }
    candidates.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

    let pairs = candidate_pairs(&candidates);
    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    let client = LspClient::start(root)?;
    let mut file_cache: HashMap<PathBuf, (String, tree_sitter::Tree)> = HashMap::new();
    let mut symptoms = Vec::new();

    for (a, b, shared_field_count, total_field_count) in pairs {
        let position = Position {
            line: a.name_point.row as u32 + 1,
            character: a.name_point.column as u32 + 1,
        };
        let at = FileLocation {
            path: a.file.clone(),
            position,
        };
        // A single candidate pair's `references()` call failing (e.g. a
        // dependency's stale rust-analyzer-side build metadata, unrelated
        // to the pair itself — observed against `catppuccin`'s generated
        // output on a full-workspace run) must not abort every other
        // pair's evidence gathering. Skip and continue rather than `?`.
        let references = match client.references(&at, false) {
            Ok(references) => references,
            Err(err) => {
                eprintln!(
                    "near-duplicate-structs: skipping {} / {} — references() failed: {err}",
                    a.qualified_name, b.qualified_name
                );
                continue;
            }
        };

        let conversion_sites = match conversion_sites(
            &references,
            &b.qualified_name,
            root,
            &mut file_cache,
        ) {
            Ok(sites) => sites,
            Err(err) => {
                eprintln!(
                    "near-duplicate-structs: skipping {} / {} — conversion-site resolution failed: {err}",
                    a.qualified_name, b.qualified_name
                );
                continue;
            }
        };
        if conversion_sites.is_empty() {
            // No real conversion code between the pair — not a finding
            // (the conversion-evidence gate).
            continue;
        }

        let relative_a = a
            .file
            .strip_prefix(root)
            .unwrap_or(a.file.as_path())
            .to_path_buf();
        let relative_b = b
            .file
            .strip_prefix(root)
            .unwrap_or(b.file.as_path())
            .to_path_buf();

        let mut footprint: HashSet<PathBuf> = HashSet::new();
        footprint.insert(relative_a.clone());
        footprint.insert(relative_b.clone());
        for site_file in &conversion_sites {
            footprint.insert(site_file.file.clone());
        }
        let footprint_file_count = footprint.len();

        let overlap_percent =
            ((shared_field_count as f64 / total_field_count as f64) * 100.0).round() as u8;

        let (sorted_a, sorted_b) = if a.qualified_name <= b.qualified_name {
            (a.qualified_name.as_str(), b.qualified_name.as_str())
        } else {
            (b.qualified_name.as_str(), a.qualified_name.as_str())
        };
        let id = SymptomId::new(
            DetectorId::new(DETECTOR_ID),
            format!("{sorted_a}|{sorted_b}").as_bytes(),
        );

        let mut sites = vec![
            Site {
                file: relative_a,
                line_ranges: vec![a.line_range],
                role: SiteRole::MatchSite,
                note: format!("Near-duplicate candidate: {}", b.qualified_name),
            },
            Site {
                file: relative_b,
                line_ranges: vec![b.line_range],
                role: SiteRole::MatchSite,
                note: format!("Near-duplicate candidate: {}", a.qualified_name),
            },
        ];
        for site in &conversion_sites {
            sites.push(Site {
                file: site.file.clone(),
                line_ranges: vec![site.line_range],
                role: SiteRole::ConversionSite,
                note: format!(
                    "Conversion between {} and {}",
                    a.qualified_name, b.qualified_name
                ),
            });
        }

        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: format!(
                "Near-duplicate structs: {} / {}",
                a.qualified_name, b.qualified_name
            ),
            evidence: Evidence::NearDuplicateStructs {
                struct_a: a.qualified_name.clone(),
                struct_b: b.qualified_name.clone(),
                shared_field_count,
                total_field_count,
                overlap_percent,
                conversion_sites: conversion_sites
                    .iter()
                    .map(|s| s.qualified_name.clone())
                    .collect(),
                footprint_file_count,
            },
            sites,
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

/// Every unordered pair of candidates whose normalized field multisets
/// meet the Jaccard `>= 0.7` / `>= 4` shared fields gate (spec.md:198-199).
/// Returns `(a, b, shared_field_count, total_field_count)` with
/// `total_field_count` the union size (the Jaccard ratio's denominator).
fn candidate_pairs(
    candidates: &[StructCandidate],
) -> Vec<(&StructCandidate, &StructCandidate, usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let a = &candidates[i];
            let b = &candidates[j];
            let a_fields: HashSet<&(String, String)> = a.fields.iter().collect();
            let b_fields: HashSet<&(String, String)> = b.fields.iter().collect();
            let shared = a_fields.intersection(&b_fields).count();
            let total = a_fields.union(&b_fields).count();
            if total == 0 {
                continue;
            }
            let jaccard = shared as f64 / total as f64;
            if shared >= MIN_SHARED_FIELDS && jaccard >= MIN_JACCARD {
                pairs.push((a, b, shared, total));
            }
        }
    }
    pairs
}

/// One reference location to `struct_a` (from the pair) that resolves to
/// real conversion code — its enclosing function or `impl` block also
/// mentions `other_struct_name`, per spec.md's conversion-evidence gate
/// (`impl From`, or an A -> B signature function).
struct ConversionSite {
    file: PathBuf,
    line_range: LineRange,
    qualified_name: String,
}

/// Filters `references` down to locations whose enclosing `function_item`
/// or `impl_item` also mentions `other_struct_name` — the textual signal
/// that this site is a conversion between the pair, not an unrelated use
/// of the struct. Deduplicates by enclosing-item identity (one struct can
/// be referenced multiple times within the same conversion function).
fn conversion_sites(
    references: &[Location],
    other_struct_name: &str,
    root: &Path,
    file_cache: &mut HashMap<PathBuf, (String, tree_sitter::Tree)>,
) -> Result<Vec<ConversionSite>, NearDuplicateStructsError> {
    let other_leaf = other_struct_name
        .rsplit("::")
        .next()
        .unwrap_or(other_struct_name);
    let mut seen = HashSet::new();
    let mut sites = Vec::new();

    for reference in references {
        let point = Point {
            row: (reference.range.start.line as usize).saturating_sub(1),
            column: (reference.range.start.character as usize).saturating_sub(1),
        };
        if !file_cache.contains_key(&reference.path) {
            let content = fs::read_to_string(&reference.path).map_err(|source| {
                NearDuplicateStructsError::Read {
                    path: reference.path.clone(),
                    source,
                }
            })?;
            let tree = parse_rust(&content, &reference.path)?;
            file_cache.insert(reference.path.clone(), (content, tree));
        }
        let (content, tree) = file_cache
            .get(&reference.path)
            .expect("just inserted above");
        let Some(node) = tree.root_node().descendant_for_point_range(point, point) else {
            continue;
        };
        let Some(item) = enclosing_conversion_item(node) else {
            continue;
        };
        let Ok(item_text) = item.utf8_text(content.as_bytes()) else {
            continue;
        };
        if !item_text.contains(other_leaf) {
            continue;
        }

        let key = (reference.path.clone(), item.start_byte());
        if !seen.insert(key) {
            continue;
        }
        sites.push(ConversionSite {
            file: reference.path.clone(),
            line_range: LineRange {
                start: item.start_position().row + 1,
                end: item.end_position().row + 1,
            },
            qualified_name: qualify_with_crate(item, content.as_bytes(), &reference.path, root),
        });
    }

    Ok(sites)
}

/// Walks up from `node` to the nearest enclosing `function_item` or
/// `impl_item` — the two shapes spec.md names as conversion evidence
/// (`impl From<A> for B`, or a free function with an A -> B signature).
fn enclosing_conversion_item(node: Node) -> Option<Node> {
    let mut current = Some(node);
    while let Some(n) = current {
        if n.kind() == "function_item" || n.kind() == "impl_item" {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, NearDuplicateStructsError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| NearDuplicateStructsError::Parse {
            path: path.to_path_buf(),
        })
}

/// Walks `node`'s subtree collecting one `StructCandidate` per named-field
/// struct (`struct Foo { ... }`) — tuple structs, unit structs, and enums
/// carry no field multiset to compare, so they're not candidates.
fn collect_structs(
    node: Node,
    source: &[u8],
    file: &Path,
    root: &Path,
    out: &mut Vec<StructCandidate>,
) {
    if node.kind() == "struct_item"
        && let Some(name_node) = node.child_by_field_name("name")
        && let Some(body) = node.child_by_field_name("body")
        && body.kind() == "field_declaration_list"
    {
        let mut fields = Vec::new();
        let mut cursor = body.walk();
        for field in body.named_children(&mut cursor) {
            if field.kind() != "field_declaration" {
                continue;
            }
            if let Some(field_name_node) = field.child_by_field_name("name")
                && let Some(ty_node) = field.child_by_field_name("type")
                && let Ok(field_name) = field_name_node.utf8_text(source)
                && let Ok(ty) = ty_node.utf8_text(source)
            {
                fields.push((field_name.to_string(), normalize_type(ty)));
            }
        }
        fields.sort();
        fields.dedup();

        out.push(StructCandidate {
            file: file.to_path_buf(),
            qualified_name: qualify_with_crate(node, source, file, root),
            name_point: name_node.start_position(),
            line_range: LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            },
            fields,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_structs(child, source, file, root, out);
    }
}

/// Strips a leading `&`/`&mut`/`mut` from a field type's text, mirroring
/// `data_clumps::detector::normalize_type` (spec.md's stripping convention,
/// re-derived locally per this crate's per-detector precedent).
fn normalize_type(text: &str) -> String {
    let mut ty = text.trim();
    if let Some(rest) = ty.strip_prefix('&') {
        ty = rest.trim_start();
        if let Some(rest) = ty.strip_prefix("mut ") {
            ty = rest.trim_start();
        }
    } else if let Some(rest) = ty.strip_prefix("mut ") {
        ty = rest.trim_start();
    }
    ty.to_string()
}

/// `qualified_name(node, source)` prefixed with the name of the crate
/// containing `file`, so that two same-named structs in different crates
/// (e.g. a workspace member vs. that crate's own generated-code output)
/// never collide in a title or in `candidate_pairs`' comparisons — see
/// contribution 007's flagged precision gap.
fn qualify_with_crate(node: Node, source: &[u8], file: &Path, root: &Path) -> String {
    let name = qualified_name(node, source);
    match crate_name_for(file, root) {
        Some(crate_name) => format!("{crate_name}::{name}"),
        None => name,
    }
}

/// The `[package] name` of the nearest ancestor directory of `file` (up to
/// `root`) that has its own `Cargo.toml` — i.e. the crate `file` belongs to.
/// Falls back to that directory's own name if the manifest has no
/// parseable `name` (should not happen for a well-formed `Cargo.toml`, but
/// this crate fails fast only on I/O/parse errors that block the whole
/// run, not on a single candidate's crate-name lookup).
fn crate_name_for(file: &Path, root: &Path) -> Option<String> {
    let mut dir = file.parent();
    while let Some(d) = dir {
        let manifest = d.join("Cargo.toml");
        if manifest.is_file() {
            return fs::read_to_string(&manifest)
                .ok()
                .and_then(|content| package_name(&content))
                .or_else(|| d.file_name().map(|n| n.to_string_lossy().into_owned()));
        }
        if d == root {
            return None;
        }
        dir = d.parent();
    }
    None
}

/// The value of `name` under `Cargo.toml`'s `[package]` table. Plain line
/// scanning (not a TOML parser) is intentional: this crate only needs one
/// field, and no `toml` dependency exists in this workspace crate today.
fn package_name(cargo_toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if let Some(section) = trimmed.strip_prefix('[') {
            in_package = section.trim_end_matches(']') == "package";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per this crate's other detectors' precedent.
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
        .unwrap_or("<anonymous>");
    parts.push(own_name.to_string());
    parts.join("::")
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, NearDuplicateStructsError> {
    let mut files = Vec::new();
    let mut builder = ignore::WalkBuilder::new(root);
    builder.add_custom_ignore_filename(crate::detectors::IGNORE_FILE_NAME);
    for entry in builder.build() {
        let entry = entry.map_err(|source| NearDuplicateStructsError::Walk {
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

    fn structs_for(content: &str) -> Vec<StructCandidate> {
        let root = Path::new("/nonexistent-test-root");
        let file = root.join("f.rs");
        let tree = parse_rust(content, &file).unwrap();
        let mut out = Vec::new();
        collect_structs(tree.root_node(), content.as_bytes(), &file, root, &mut out);
        out
    }

    fn names(candidates: &[StructCandidate]) -> Vec<&str> {
        candidates
            .iter()
            .map(|c| c.qualified_name.as_str())
            .collect()
    }

    #[test]
    fn a_named_field_struct_is_a_candidate() {
        let candidates = structs_for("struct Foo { a: u32, b: String }");
        assert_eq!(names(&candidates), vec!["Foo"]);
        assert_eq!(
            candidates[0].fields,
            vec![
                ("a".to_string(), "u32".to_string()),
                ("b".to_string(), "String".to_string())
            ]
        );
    }

    #[test]
    fn a_tuple_struct_is_not_a_candidate() {
        let candidates = structs_for("struct Foo(u32, String);");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_unit_struct_is_not_a_candidate() {
        let candidates = structs_for("struct Foo;");
        assert!(candidates.is_empty());
    }

    #[test]
    fn normalize_type_strips_reference_and_mut() {
        assert_eq!(normalize_type("u64"), "u64");
        assert_eq!(normalize_type("&u64"), "u64");
        assert_eq!(normalize_type("&mut u64"), "u64");
        assert_eq!(normalize_type("mut u64"), "u64");
    }

    #[test]
    fn candidate_pairs_gates_on_shared_count_and_jaccard() {
        let candidates = structs_for(
            "struct A { f1: u32, f2: u32, f3: u32, f4: u32 }\n\
             struct B { f1: u32, f2: u32, f3: u32, f4: u32, f5: u32 }\n\
             struct C { f1: u32, f2: u32 }",
        );
        let pairs = candidate_pairs(&candidates);
        let names: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(a, b, _, _)| (a.qualified_name.as_str(), b.qualified_name.as_str()))
            .collect();
        assert_eq!(
            names,
            vec![("A", "B")],
            "A/B share 4/5 fields (Jaccard 0.8); \
            C only shares 2 fields with either and must not qualify"
        );
    }

    #[test]
    fn candidate_pairs_rejects_high_jaccard_below_shared_minimum() {
        let candidates = structs_for(
            "struct A { f1: u32, f2: u32 }\n\
             struct B { f1: u32, f2: u32 }",
        );
        let pairs = candidate_pairs(&candidates);
        assert!(
            pairs.is_empty(),
            "Jaccard is 1.0 but only 2 shared fields (below the >= 4 minimum)"
        );
    }
}
