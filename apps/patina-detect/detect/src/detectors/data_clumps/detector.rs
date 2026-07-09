use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser};

/// The detector id every data-clump `Symptom`/`SymptomId` is tagged with.
pub const DETECTOR_ID: &str = "data-clumps";

/// spec.md:229: pairs are hopeless ("(path, line) recurs legitimately
/// everywhere") — the clump must have at least this many members.
pub const MIN_CLUMP_SIZE: usize = 3;

/// spec.md:229: a clump only counts once it recurs across at least this
/// many distinct signatures.
pub const MIN_OCCURRENCES: usize = 3;

#[derive(Debug, Error)]
pub enum DataClumpsError {
    #[error("failed to walk directory {path}")]
    Walk {
        path: PathBuf,
        #[source]
        source: std::io::Error,
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
}

/// A single function/method signature carrying a candidate clump (>= 3
/// normalized parameters), plus whether its own body forwards those
/// parameters intact to another call.
struct Occurrence {
    file: PathBuf,
    qualified_name: String,
    members: Vec<(String, String)>,
    line_range: LineRange,
    forwarding: Option<LineRange>,
}

/// Runs the data-clumps detector (spec.md:226-248) against every `.rs` file
/// found recursively under `root`: every free function, inherent-impl
/// method, and trait-declared signature with >= 3 normalized parameters is
/// a candidate; candidates sharing the same normalized member set are
/// grouped, promoted to a `Symptom` only when the group has >=
/// `MIN_OCCURRENCES` members AND at least one member forwards the clump
/// intact to another call (spec.md:238's precision gate — non-traveling
/// clumps are dropped).
pub fn run_data_clumps(root: &Path) -> Result<Vec<Symptom>, DataClumpsError> {
    let mut occurrences: Vec<Occurrence> = Vec::new();
    let mut structs: Vec<(String, HashSet<(String, String)>)> = Vec::new();

    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| DataClumpsError::Read {
            path: file.clone(),
            source,
        })?;
        let tree = parse_rust(&content, &file)?;
        let relative = file
            .strip_prefix(root)
            .unwrap_or(file.as_path())
            .to_path_buf();

        collect_occurrences(
            tree.root_node(),
            content.as_bytes(),
            &relative,
            false,
            &mut occurrences,
        );
        collect_structs(tree.root_node(), content.as_bytes(), &mut structs);
    }

    let mut groups: BTreeMap<Vec<(String, String)>, Vec<&Occurrence>> = BTreeMap::new();
    for occurrence in &occurrences {
        groups
            .entry(occurrence.members.clone())
            .or_default()
            .push(occurrence);
    }

    let mut symptoms = Vec::new();
    for (members, group) in groups {
        if group.len() < MIN_OCCURRENCES {
            continue;
        }
        let forwarding: Vec<&&Occurrence> =
            group.iter().filter(|o| o.forwarding.is_some()).collect();
        if forwarding.is_empty() {
            continue;
        }

        let member_set: HashSet<(String, String)> = members.iter().cloned().collect();
        let subset_of_struct = structs
            .iter()
            .find(|(_, fields)| member_set.is_subset(fields))
            .map(|(name, _)| name.clone());

        let fingerprint = members
            .iter()
            .map(|(name, ty)| format!("{name}:{ty}"))
            .collect::<Vec<_>>()
            .join("|");
        let id = SymptomId::new(DetectorId::new(DETECTOR_ID), fingerprint.as_bytes());

        let member_names = members
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let mut sites = Vec::new();
        for occurrence in &group {
            sites.push(Site {
                file: occurrence.file.clone(),
                line_ranges: vec![occurrence.line_range],
                role: SiteRole::MatchSite,
                note: format!("Signature clump member: {}", occurrence.qualified_name),
            });
            if let Some(forwarding_range) = occurrence.forwarding {
                sites.push(Site {
                    file: occurrence.file.clone(),
                    line_ranges: vec![forwarding_range],
                    role: SiteRole::ForwardingSite,
                    note: format!(
                        "{} forwards the clump intact to another call",
                        occurrence.qualified_name
                    ),
                });
            }
        }

        let forwarding_chain = forwarding
            .iter()
            .map(|o| o.qualified_name.clone())
            .collect();

        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: format!(
                "Data clump ({} members, {} occurrences): {member_names}",
                members.len(),
                group.len()
            ),
            evidence: Evidence::DataClump {
                members,
                occurrence_count: group.len(),
                forwarding_chain,
                subset_of_struct,
            },
            sites,
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, DataClumpsError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| DataClumpsError::Parse {
            path: path.to_path_buf(),
        })
}

/// Walks `node`'s subtree collecting one `Occurrence` per free function,
/// inherent-impl method, or trait-declared signature with >= 3 normalized
/// parameters. `in_trait_impl` tracks whether the current node is nested
/// inside an `impl Trait for Type` block — spec.md:232-234's dedup rule
/// ("count a signature once per trait, not once per impl") is implemented
/// as exclusion: methods inside a trait impl never become candidates at
/// all, leaving only the trait's own declared signature (if present in the
/// scanned tree) as the counted occurrence.
fn collect_occurrences(
    node: Node,
    source: &[u8],
    file: &Path,
    in_trait_impl: bool,
    out: &mut Vec<Occurrence>,
) {
    let mut child_in_trait_impl = in_trait_impl;
    if node.kind() == "impl_item" {
        child_in_trait_impl = node.child_by_field_name("trait").is_some();
    }

    if !in_trait_impl
        && matches!(node.kind(), "function_item" | "function_signature_item")
        && let Some(occurrence) = build_occurrence(node, source, file)
    {
        out.push(occurrence);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_occurrences(child, source, file, child_in_trait_impl, out);
    }
}

fn build_occurrence(node: Node, source: &[u8], file: &Path) -> Option<Occurrence> {
    let params_node = node.child_by_field_name("parameters")?;
    let mut cursor = params_node.walk();
    let mut members = Vec::new();
    for parameter in params_node.named_children(&mut cursor) {
        if parameter.kind() != "parameter" {
            continue;
        }
        let pattern = parameter.child_by_field_name("pattern")?;
        let name = pattern.utf8_text(source).ok()?.to_string();
        let ty_node = parameter.child_by_field_name("type")?;
        let ty = normalize_type(ty_node.utf8_text(source).ok()?);
        members.push((name, ty));
    }

    members.sort();
    members.dedup();
    if members.len() < MIN_CLUMP_SIZE {
        return None;
    }

    let qualified_name = qualified_name(node, source);
    let line_range = LineRange {
        start: node.start_position().row + 1,
        end: node.end_position().row + 1,
    };
    let forwarding = node
        .child_by_field_name("body")
        .and_then(|body| find_forwarding_call(body, source, &members));

    Some(Occurrence {
        file: file.to_path_buf(),
        qualified_name,
        members,
        line_range,
        forwarding,
    })
}

/// Strips a leading `&`/`&mut`/`mut` from a parameter or field type's text
/// (spec.md:228: "stripping `&`/`mut`") so `u64`, `&u64`, and `&mut u64` all
/// normalize to the same clump member type.
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

/// Finds a `call_expression` anywhere in `body` whose arguments are simple
/// identifiers matching, as a set, `members`' parameter names exactly
/// (spec.md:238: "matching call-site argument identifiers against
/// enclosing parameters" — order-independent, since a clump can be
/// reordered at the call site without ceasing to be forwarded intact).
fn find_forwarding_call(
    body: Node,
    source: &[u8],
    members: &[(String, String)],
) -> Option<LineRange> {
    let member_names: HashSet<&str> = members.iter().map(|(name, _)| name.as_str()).collect();
    find_forwarding_call_in(body, source, &member_names)
}

fn find_forwarding_call_in(
    node: Node,
    source: &[u8],
    member_names: &HashSet<&str>,
) -> Option<LineRange> {
    if node.kind() == "call_expression"
        && let Some(arguments) = node.child_by_field_name("arguments")
    {
        let mut cursor = arguments.walk();
        let mut arg_set = HashSet::new();
        let mut all_identifiers = true;
        for arg in arguments.named_children(&mut cursor) {
            if arg.kind() == "identifier"
                && let Ok(text) = arg.utf8_text(source)
            {
                arg_set.insert(text);
            } else {
                all_identifiers = false;
            }
        }
        if all_identifiers && arg_set.len() == member_names.len() && &arg_set == member_names {
            return Some(LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(range) = find_forwarding_call_in(child, source, member_names) {
            return Some(range);
        }
    }
    None
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per `cognitive_complexity`'s precedent rather than
/// depending on `diffviz-core`'s `SemanticTree` pipeline.
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

/// Collects every `struct_item`'s normalized field set, for the bonus
/// subset-of-struct evidence (spec.md:243-245).
fn collect_structs(node: Node, source: &[u8], out: &mut Vec<(String, HashSet<(String, String)>)>) {
    if node.kind() == "struct_item"
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(source)
        && let Some(body) = node.child_by_field_name("body")
        && body.kind() == "field_declaration_list"
    {
        let mut fields = HashSet::new();
        let mut cursor = body.walk();
        for field in body.named_children(&mut cursor) {
            if field.kind() != "field_declaration" {
                continue;
            }
            if let Some(fname) = field.child_by_field_name("name")
                && let Some(ftype) = field.child_by_field_name("type")
                && let Ok(fname_text) = fname.utf8_text(source)
                && let Ok(ftype_text) = ftype.utf8_text(source)
            {
                fields.insert((fname_text.to_string(), normalize_type(ftype_text)));
            }
        }
        out.push((name.to_string(), fields));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_structs(child, source, out);
    }
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, DataClumpsError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), DataClumpsError> {
    let entries = fs::read_dir(dir).map_err(|source| DataClumpsError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DataClumpsError::Walk {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            visit(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}
