use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{FileLocation, LspClient, Position};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

/// The detector id every data-clump `Symptom`/`SymptomId` is tagged with.
pub const DETECTOR_ID: &str = "data-clumps";

/// spec.md:229: pairs are hopeless ("(path, line) recurs legitimately
/// everywhere") — the clump must have at least this many members.
pub const MIN_CLUMP_SIZE: usize = 3;

/// spec.md:229: a clump only counts once it recurs across at least this
/// many distinct signatures.
pub const MIN_OCCURRENCES: usize = 3;

/// A clump's normalized member set: (parameter name, normalized type) pairs.
type MemberSet = Vec<(String, String)>;

/// Occurrences sharing one normalized member set, keyed by that set.
type GroupsByMembers = BTreeMap<MemberSet, Vec<Occurrence>>;

/// A struct's name and its normalized field set, for the bonus
/// subset-of-struct evidence.
type StructFieldSets = Vec<(String, HashSet<(String, String)>)>;

/// Every `function_item` in a scanned file — no minimum parameter count,
/// trait-impl methods included — so `is_closed_cluster` can resolve an
/// incoming caller back to its enclosing function's parameter set and
/// test-ness.
struct FunctionInfo {
    line_range: LineRange,
    /// 0-based tree-sitter position of the function's name identifier, the
    /// position `is_closed_cluster` queries when this function is absorbed
    /// into a cluster and its own callers must be chased.
    name_point: Point,
    /// Normalized `(name, type)` parameter set, same normalization as
    /// `Occurrence::members`.
    members: HashSet<(String, String)>,
    /// Whether the function sits under a `#[test]`/`#[cfg(test)]` attribute
    /// (directly or via an ancestor mod) — mirrors `dead_exports`'
    /// test-context classification.
    in_test_context: bool,
}

/// Per-file `FunctionInfo`s, keyed by root-relative path (the same path
/// convention `Occurrence::file` uses).
type FunctionIndex = BTreeMap<PathBuf, Vec<FunctionInfo>>;

#[derive(Debug, Error)]
pub enum DataClumpsError {
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

/// A single function/method signature carrying a candidate clump (>= 3
/// normalized parameters), plus whether its own body forwards those
/// parameters intact to another call.
struct Occurrence {
    file: PathBuf,
    qualified_name: String,
    members: Vec<(String, String)>,
    line_range: LineRange,
    forwarding: Option<LineRange>,
    /// 0-based tree-sitter position of the occurrence's own name
    /// identifier — the position `lspkit::LspClient::prepare_call_hierarchy`
    /// is queried against by `run_data_clumps_refined`'s closed-cluster
    /// check. Unused by the plain tree-sitter-only `run_data_clumps`.
    name_point: Point,
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
    let (groups, structs, _) = promoted_groups(root)?;
    Ok(build_symptoms(groups, &structs))
}

/// Phase 16 revision (decision D011): runs the same tree-sitter clump
/// extraction and promotion gates as `run_data_clumps`, then additionally
/// excludes any promoted group whose call graph is *closed* — at most one
/// distinct production call site anywhere outside the clump's traveling
/// family ever reaches in (see [`is_closed_cluster`] for the family
/// expansion and for why "at most one", not strictly zero). This is the
/// false-positive shape a private recursive-descent visitor's own helper
/// functions produce when they forward `(node, accumulator...)` state to
/// each other (e.g. `cognitive_complexity::score_node`/`score_if`/
/// `score_match`, whose only production entry is `run_cognitive_complexity`'s
/// single call into `score_node`) — confined to one module, never genuinely
/// reused from more than one outside call site. A group reached from >= 2
/// distinct external call sites is kept regardless of whether all its
/// members happen to share a file (spec.md/D011: a file/module-scope
/// heuristic was rejected precisely because it would wrongly exclude that
/// case too).
pub fn run_data_clumps_refined(root: &Path) -> Result<Vec<Symptom>, DataClumpsError> {
    // Same rationale as every other lspkit-backed detector in this crate:
    // `LspClient::start` builds a `file://` URI directly from `root`, which
    // is malformed for a relative path.
    let root = fs::canonicalize(root).map_err(|source| DataClumpsError::Canonicalize {
        path: root.to_path_buf(),
        source,
    })?;
    let root = root.as_path();

    let (groups, structs, functions) = promoted_groups(root)?;

    let client = LspClient::start(root)?;

    let mut kept: GroupsByMembers = BTreeMap::new();
    for (members, group) in groups {
        match is_closed_cluster(&client, root, &group, &functions) {
            Ok(true) => continue,
            Ok(false) => {
                kept.insert(members, group);
            }
            // A group whose call-hierarchy resolution failed outright
            // (e.g. a dependency's stale rust-analyzer-side build
            // metadata, unrelated to this group — see
            // `single_impl_traits::detector`'s identical fix) must not
            // abort every other group's evidence gathering, and must not
            // be silently dropped either: keep it, since failing to prove
            // "closed" is not the same as proving it.
            Err(err) => {
                eprintln!(
                    "data-clumps: keeping group {:?} — closed-cluster check failed: {err}",
                    members
                );
                kept.insert(members, group);
            }
        }
    }

    Ok(build_symptoms(kept, &structs))
}

/// Shared by `run_data_clumps` and `run_data_clumps_refined`: walks every
/// `.rs` file under `root`, collects candidate occurrences, struct field
/// sets, and the per-file function index, groups occurrences by normalized
/// member set, and applies the standard Phase 6 promotion gates (`>=
/// MIN_OCCURRENCES` members, at least one forwarding occurrence). Callers
/// apply any further filtering (e.g. the Phase 16 closed-cluster check)
/// before building symptoms.
fn promoted_groups(
    root: &Path,
) -> Result<(GroupsByMembers, StructFieldSets, FunctionIndex), DataClumpsError> {
    let mut occurrences: Vec<Occurrence> = Vec::new();
    let mut structs: StructFieldSets = Vec::new();
    let mut functions: FunctionIndex = BTreeMap::new();

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

        let mut file_functions = Vec::new();
        collect_functions(
            tree.root_node(),
            content.as_bytes(),
            false,
            &mut file_functions,
        );
        if !file_functions.is_empty() {
            functions.insert(relative, file_functions);
        }
    }

    let mut groups: GroupsByMembers = BTreeMap::new();
    for occurrence in occurrences {
        groups
            .entry(occurrence.members.clone())
            .or_default()
            .push(occurrence);
    }

    groups.retain(|_, group| {
        group.len() >= MIN_OCCURRENCES && group.iter().any(|o| o.forwarding.is_some())
    });

    Ok((groups, structs, functions))
}

fn build_symptoms(groups: GroupsByMembers, structs: &StructFieldSets) -> Vec<Symptom> {
    let mut symptoms = Vec::new();
    for (members, group) in groups {
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

        let forwarding: Vec<&Occurrence> =
            group.iter().filter(|o| o.forwarding.is_some()).collect();

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
    symptoms
}

/// Whether `group`'s call graph is *closed*: at most one distinct production
/// call site from outside the clump's traveling family reaches in anywhere.
/// Zero external callers is the textbook closed recursive/mutually-recursive
/// family; exactly one is a private algorithm's own single driver/entry
/// point (e.g. `run_cognitive_complexity`'s one call into `score_node` to
/// kick off the visitor) — still not the "clump travels between independent
/// call sites" smell this detector exists to find. Two or more distinct
/// external callers means the clump genuinely is being passed around from
/// separate places, so the group is kept regardless of whether those callers
/// happen to share a file with the group (decision D011 — this is exactly
/// the shape the rejected file/module heuristic would have wrongly
/// excluded).
///
/// Two caller shapes do not count as external, both learned from the
/// `cognitive_complexity` scorer family this check exists to exclude yet
/// initially failed to:
///
/// - A caller whose own normalized parameter set is a *superset* of the
///   clump (`score_binary`/`score_operand` carry `(node, nesting,
///   max_nesting_depth)` plus `run_op`) is the same family with an extra
///   argument, not an independent origin — it is absorbed into the cluster
///   and its own incoming calls are chased transitively, so an extra entry
///   point hiding behind it still counts.
/// - A caller in `#[test]`/`#[cfg(test)]` context (the `score_of` test
///   helper) is a test driver exercising the family, not production code
///   the clump travels through (mirrors `dead_exports`' test-only
///   classification).
///
/// A caller that cannot be resolved in `functions` (outside the scanned
/// root, or not enclosed by any indexed function) counts as external:
/// failing to prove family membership must fail toward reporting, same as
/// the caller-side error handling in `run_data_clumps_refined`.
///
/// `root` resolves each queried function's absolute file path for the
/// `prepare_call_hierarchy` query and relativizes caller paths back to the
/// index's root-relative convention.
fn is_closed_cluster(
    client: &LspClient,
    root: &Path,
    group: &[Occurrence],
    functions: &FunctionIndex,
) -> Result<bool, DataClumpsError> {
    let clump: HashSet<(String, String)> = group
        .iter()
        .flat_map(|o| o.members.iter().cloned())
        .collect();

    // Root-relative file + range of every cluster member found so far, and
    // the worklist of member name positions whose incoming calls are still
    // to be inspected.
    let mut cluster: Vec<(PathBuf, LineRange)> = group
        .iter()
        .map(|o| (o.file.clone(), o.line_range))
        .collect();
    let mut worklist: Vec<(PathBuf, Point)> = group
        .iter()
        .map(|o| (o.file.clone(), o.name_point))
        .collect();
    let mut queried: HashSet<(PathBuf, usize)> = HashSet::new();
    let mut external_callers: HashSet<(PathBuf, usize)> = HashSet::new();

    while let Some((file, name_point)) = worklist.pop() {
        if !queried.insert((file.clone(), name_point.row)) {
            continue;
        }
        let at = FileLocation {
            path: root.join(&file),
            position: Position {
                line: name_point.row as u32 + 1,
                character: name_point.column as u32 + 1,
            },
        };
        for item in client.prepare_call_hierarchy(&at)? {
            for caller in client.incoming_calls(&item)? {
                // lspkit's `CallHierarchyItem` range is the LSP
                // selectionRange — the caller's name identifier, 1-based.
                let caller_line = caller.item.location.range.start.line as usize;
                let caller_file = caller.item.location.path.strip_prefix(root).ok();

                if let Some(caller_file) = caller_file
                    && cluster.iter().any(|(f, range)| {
                        f == caller_file && range.start <= caller_line && caller_line <= range.end
                    })
                {
                    continue;
                }

                let info = caller_file.and_then(|caller_file| {
                    functions
                        .get(caller_file)?
                        .iter()
                        .filter(|f| {
                            f.line_range.start <= caller_line && caller_line <= f.line_range.end
                        })
                        .max_by_key(|f| f.line_range.start)
                });
                match (caller_file, info) {
                    (_, Some(f)) if f.in_test_context => {}
                    (Some(caller_file), Some(f)) if clump.is_subset(&f.members) => {
                        cluster.push((caller_file.to_path_buf(), f.line_range));
                        worklist.push((caller_file.to_path_buf(), f.name_point));
                    }
                    _ => {
                        external_callers.insert((caller.item.location.path.clone(), caller_line));
                    }
                }
                // Two distinct external callers already prove the cluster
                // open; every further round trip would only re-prove it.
                if external_callers.len() >= 2 {
                    return Ok(false);
                }
            }
        }
    }

    Ok(external_callers.len() <= 1)
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
    let name_point = node
        .child_by_field_name("name")
        .map(|n| n.start_position())
        .unwrap_or_else(|| node.start_position());
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
        name_point,
    })
}

/// Walks `node`'s subtree collecting one `FunctionInfo` per `function_item`
/// — every function, unlike `collect_occurrences`' candidate gates, because
/// `is_closed_cluster` must be able to resolve *any* incoming caller.
/// `in_test` propagates `#[test]`/`#[cfg(test)]` context down from an
/// annotated `mod_item`/`function_item` to everything nested inside it.
fn collect_functions(node: Node, source: &[u8], in_test: bool, out: &mut Vec<FunctionInfo>) {
    let mut child_in_test = in_test;
    if matches!(node.kind(), "mod_item" | "function_item")
        && has_preceding_attribute(node, source, "test")
    {
        child_in_test = true;
    }

    if node.kind() == "function_item" {
        let mut members = HashSet::new();
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for parameter in params_node.named_children(&mut cursor) {
                if parameter.kind() != "parameter" {
                    continue;
                }
                if let Some(pattern) = parameter.child_by_field_name("pattern")
                    && let Some(ty_node) = parameter.child_by_field_name("type")
                    && let Ok(name) = pattern.utf8_text(source)
                    && let Ok(ty) = ty_node.utf8_text(source)
                {
                    members.insert((name.to_string(), normalize_type(ty)));
                }
            }
        }
        let name_point = node
            .child_by_field_name("name")
            .map(|n| n.start_position())
            .unwrap_or_else(|| node.start_position());
        out.push(FunctionInfo {
            line_range: LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            },
            name_point,
            members,
            in_test_context: child_in_test,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, source, child_in_test, out);
    }
}

/// Whether `node` is immediately preceded (among its parent's children) by
/// an `attribute_item` whose text contains `keyword` — re-derived locally
/// per `dead_exports`' identical helper rather than sharing it across
/// detector modules (this crate's self-contained-detector precedent).
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
    let mut builder = ignore::WalkBuilder::new(root);
    builder.add_custom_ignore_filename(crate::detectors::IGNORE_FILE_NAME);
    for entry in builder.build() {
        let entry = entry.map_err(|source| DataClumpsError::Walk {
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
