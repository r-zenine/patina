use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use lspkit::{CallSite, FileLocation, LspClient, Position};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser, Point};

/// The detector id every middleman-delegation `Symptom`/`SymptomId` is
/// tagged with.
pub const DETECTOR_ID: &str = "middleman-delegation";

#[derive(Debug, Error)]
pub enum MiddlemanDelegationError {
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

/// One free function or inherent-impl method found while scanning.
/// `delegate_target` is the simple name of the function/method its body
/// calls, populated only when the whole body is a single delegating call
/// (spec.md:170) — everything else about the candidate is collected
/// regardless, since non-delegating candidates are still needed to resolve
/// other candidates' delegate targets and callers.
struct Candidate {
    file: PathBuf,
    qualified_name: String,
    simple_name: String,
    /// 0-based tree-sitter position of the candidate's own name identifier,
    /// the position `lspkit::LspClient::prepare_call_hierarchy` is queried
    /// against.
    name_point: Point,
    line_range: LineRange,
    delegate_target: Option<String>,
}

/// Runs the middleman-delegation detector (spec.md:165-177) against the Rust
/// crate rooted at `root`: tree-sitter finds functions whose body is a
/// single *pure pass-through* delegating call (excluding trait-impl and
/// trait-declared methods, which may be satisfying an interface; wrappers
/// that adapt arguments — see `delegate_call`; and encapsulation facades
/// over a private field of their own type — see `is_encapsulation_facade`);
/// `lspkit::LspClient::incoming_calls`
/// confirms each has exactly one same-crate caller. Confirmed middlemen that
/// delegate into one another are composed into a single chain per
/// spec.md:176 ("the chain (A → B → C) when middlemen compose").
pub fn run_middleman_delegation(root: &Path) -> Result<Vec<Symptom>, MiddlemanDelegationError> {
    // Same rationale as `dead_exports::run_dead_exports`: `LspClient::start`
    // builds a `file://` URI directly from `root`, which is malformed for a
    // relative path — canonicalize once up front.
    let root = fs::canonicalize(root).map_err(|source| MiddlemanDelegationError::Canonicalize {
        path: root.to_path_buf(),
        source,
    })?;
    let root = root.as_path();

    let mut candidates = Vec::new();
    for file in collect_rust_files(root)? {
        let content =
            fs::read_to_string(&file).map_err(|source| MiddlemanDelegationError::Read {
                path: file.clone(),
                source,
            })?;
        let tree = parse_rust(&content, &file)?;
        collect_candidates(
            tree.root_node(),
            content.as_bytes(),
            &file,
            false,
            &mut candidates,
        );
    }

    let mut by_simple_name: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        by_simple_name
            .entry(candidate.simple_name.as_str())
            .or_default()
            .push(index);
    }

    let client = LspClient::start(root)?;

    let mut confirmed: HashMap<usize, CallSite> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.delegate_target.is_none() {
            continue;
        }
        let position = Position {
            line: candidate.name_point.row as u32 + 1,
            character: candidate.name_point.column as u32 + 1,
        };
        let at = FileLocation {
            path: candidate.file.clone(),
            position,
        };
        // A single candidate's lspkit call failing (e.g. a dependency's stale
        // rust-analyzer-side build metadata, unrelated to the candidate
        // itself — see `near_duplicate_structs::detector`'s identical fix)
        // must not abort every other candidate's evidence gathering. Skip
        // and continue rather than `?`.
        let items = match client.prepare_call_hierarchy(&at) {
            Ok(items) => items,
            Err(err) => {
                eprintln!(
                    "middleman-delegation: skipping {} — prepare_call_hierarchy() failed: {err}",
                    candidate.qualified_name
                );
                continue;
            }
        };
        let Some(item) = items.into_iter().next() else {
            continue;
        };
        let callers = match client.incoming_calls(&item) {
            Ok(callers) => callers,
            Err(err) => {
                eprintln!(
                    "middleman-delegation: skipping {} — incoming_calls() failed: {err}",
                    candidate.qualified_name
                );
                continue;
            }
        };
        let same_crate_callers: Vec<CallSite> = callers
            .into_iter()
            .filter(|call_site| call_site.item.location.path.starts_with(root))
            .collect();
        if same_crate_callers.len() == 1 {
            confirmed.insert(index, same_crate_callers.into_iter().next().unwrap());
        }
    }

    // Edges among confirmed middlemen only: a confirmed middleman's
    // unambiguously-resolved delegate target, when that target is itself
    // confirmed, composes the chain (spec.md:176).
    let mut edge: HashMap<usize, usize> = HashMap::new();
    for &index in confirmed.keys() {
        let target_name = candidates[index]
            .delegate_target
            .as_ref()
            .expect("confirmed candidates always have a delegate_target");
        if let Some(target_indices) = by_simple_name.get(target_name.as_str())
            && let [target_index] = target_indices[..]
            && confirmed.contains_key(&target_index)
        {
            edge.insert(index, target_index);
        }
    }

    let chained: HashSet<usize> = edge.values().copied().collect();
    let mut heads: Vec<usize> = confirmed
        .keys()
        .copied()
        .filter(|index| !chained.contains(index))
        .collect();
    heads.sort();

    let mut symptoms = Vec::new();
    for head in heads {
        let mut chain_indices = vec![head];
        let mut current = head;
        while let Some(&next) = edge.get(&current) {
            chain_indices.push(next);
            current = next;
        }

        let chain_names: Vec<String> = chain_indices
            .iter()
            .map(|&index| candidates[index].qualified_name.clone())
            .collect();
        let mut fingerprint_parts = chain_names.clone();
        fingerprint_parts.sort();
        let fingerprint = fingerprint_parts.join("|");
        let id = SymptomId::new(DetectorId::new(DETECTOR_ID), fingerprint.as_bytes());

        let caller = confirmed
            .get(&head)
            .expect("head is always a key of confirmed");
        let caller_relative = caller
            .item
            .location
            .path
            .strip_prefix(root)
            .unwrap_or(caller.item.location.path.as_path())
            .to_path_buf();
        let mut sites = vec![Site {
            file: caller_relative,
            line_ranges: vec![LineRange {
                start: caller.item.location.range.start.line as usize,
                end: caller.item.location.range.end.line as usize,
            }],
            role: SiteRole::Caller,
            note: format!("Sole caller of {}", candidates[head].qualified_name),
        }];
        for &index in &chain_indices {
            let candidate = &candidates[index];
            let relative = candidate
                .file
                .strip_prefix(root)
                .unwrap_or(candidate.file.as_path())
                .to_path_buf();
            sites.push(Site {
                file: relative,
                line_ranges: vec![candidate.line_range],
                role: SiteRole::MatchSite,
                note: format!(
                    "Delegates to {}",
                    candidate.delegate_target.as_deref().unwrap_or("<unknown>")
                ),
            });
        }

        symptoms.push(Symptom {
            id,
            detector: DetectorId::new(DETECTOR_ID),
            title: if chain_names.len() > 1 {
                format!("Middleman delegation chain: {}", chain_names.join(" -> "))
            } else {
                format!("Middleman delegation: {}", chain_names[0])
            },
            evidence: Evidence::MiddlemanChain {
                chain: chain_names,
                caller_count: 1,
                body_shape: "single delegating call".to_string(),
            },
            sites,
        });
    }

    symptoms.sort_by_key(|s| s.id.to_string());
    Ok(symptoms)
}

fn parse_rust(content: &str, path: &Path) -> Result<tree_sitter::Tree, MiddlemanDelegationError> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    parser
        .parse(content, None)
        .ok_or_else(|| MiddlemanDelegationError::Parse {
            path: path.to_path_buf(),
        })
}

/// Walks `node`'s subtree collecting one `Candidate` per free function and
/// inherent-impl method. `in_trait_context` skips both trait-declared
/// methods (inside a `trait_item`) and trait-impl methods (inside an
/// `impl Trait for Type` block) — spec.md:172's exclusion, since either may
/// be satisfying an interface rather than being a pointless wrapper.
fn collect_candidates(
    node: Node,
    source: &[u8],
    file: &Path,
    in_trait_context: bool,
    out: &mut Vec<Candidate>,
) {
    let mut child_in_trait_context = in_trait_context;
    if node.kind() == "trait_item" {
        child_in_trait_context = true;
    } else if node.kind() == "impl_item" {
        child_in_trait_context = node.child_by_field_name("trait").is_some();
    }

    if !in_trait_context
        && node.kind() == "function_item"
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(source)
    {
        out.push(Candidate {
            file: file.to_path_buf(),
            qualified_name: qualified_name(node, source),
            simple_name: name.to_string(),
            name_point: name_node.start_position(),
            line_range: LineRange {
                start: node.start_position().row + 1,
                end: node.end_position().row + 1,
            },
            delegate_target: delegate_call(node, source),
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_candidates(child, source, file, child_in_trait_context, out);
    }
}

/// Whether `body` (a function's `block`) consists of exactly one statement —
/// a bare call expression tail, a `return <call>;`, or `<call>;` — and, if
/// so, whether that call's own subtree (receiver/arguments, including
/// inside any closures they carry) contains no *other* call — i.e. the body
/// is a single, unadorned pass-through, not a combinator/builder chain
/// (`.iter().filter(f).map(g).collect()`) that merely happens to present
/// one `call_expression` at the top. Revision (contribution 005, decision
/// 1): the original check only looked at the outer node's kind and missed
/// this whole FP class, confirmed against real code in this repo (not just
/// a synthetic case) — see spec.md:170's "single delegating call".
fn body_single_call(body: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = body.walk();
    let named: Vec<Node> = body.named_children(&mut cursor).collect();
    let [only] = named[..] else {
        return None;
    };

    let mut call = match only.kind() {
        "expression_statement" | "return_expression" => only.named_child(0)?,
        _ => only,
    };
    if call.kind() == "return_expression" {
        call = call.named_child(0)?;
    }
    if call.kind() != "call_expression" {
        return None;
    }
    if count_calls(call) != 1 {
        return None;
    }
    Some(call)
}

/// Analyzes `function_item`'s body for a *pure* pass-through delegation and
/// returns the callee's simple name. Two Phase 5
/// (plan-patina-detect-fp-fixes) exclusions apply after `body_single_call`
/// finds the lone call:
///
/// - **Adaptation check**: every argument must be a bare identifier naming
///   one of the wrapper's own parameters (or `self`). Any other argument —
///   a field borrow (`&self.data`), a literal, an expression — means the
///   wrapper *adapts* the call site (e.g. a trait-signature adapter like
///   `TriageApp::process_key_event` forwarding
///   `(&self.data, &mut self.baseline, key)`, or partial application like
///   `serialize_phase_6` pinning `"code_impact"`), which is real value, not
///   a pointless middleman.
/// - **Composition-facade check**: a call rooted at a `self` field
///   (`self.leader.activate()`, `self.0.push(c)`) is the ordinary Rust
///   composition idiom — the method presents an owned component's behavior
///   as the type's own API. The audit found this pattern (e.g.
///   `UiState::activate_leader`) dominating the false positives, on `pub`
///   fields as much as private ones, so field visibility deliberately plays
///   no part (the roadmap's initial visibility-based sketch would have kept
///   flagging the audit's own canonical examples).
fn delegate_call(function_item: Node, source: &[u8]) -> Option<String> {
    let body = function_item.child_by_field_name("body")?;
    let call = body_single_call(body)?;

    let params = parameter_names(function_item, source);
    let arguments = call.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    for argument in arguments.named_children(&mut cursor) {
        let pure_forward = match argument.kind() {
            "self" => true,
            "identifier" => argument
                .utf8_text(source)
                .is_ok_and(|name| params.iter().any(|p| p == name)),
            _ => false,
        };
        if !pure_forward {
            return None;
        }
    }

    let function = call.child_by_field_name("function")?;
    if is_self_field_receiver(function) {
        return None;
    }
    let text = function.utf8_text(source).ok()?;
    Some(simple_call_name(text))
}

/// Names bound by `function_item`'s parameter patterns (`x` for `x: u32`
/// or `mut x: u32`), the only values a pure pass-through may forward.
fn parameter_names(function_item: Node, source: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let Some(parameters) = function_item.child_by_field_name("parameters") else {
        return names;
    };
    let mut cursor = parameters.walk();
    for parameter in parameters.named_children(&mut cursor) {
        if parameter.kind() != "parameter" {
            continue;
        }
        let Some(pattern) = parameter.child_by_field_name("pattern") else {
            continue;
        };
        let identifier = match pattern.kind() {
            "identifier" => Some(pattern),
            "mut_pattern" => pattern.named_child(0).filter(|n| n.kind() == "identifier"),
            _ => None,
        };
        if let Some(identifier) = identifier
            && let Ok(name) = identifier.utf8_text(source)
        {
            names.push(name.to_string());
        }
    }
    names
}

/// Whether a method-call target is rooted at `self` through at least one
/// field (`self.state.leader.activate`) — the composition-facade shape.
/// `false` for plain `self.method()` calls, free-function and `Type::`
/// targets.
fn is_self_field_receiver(function: Node) -> bool {
    if function.kind() != "field_expression" {
        return false;
    }
    let mut current = function;
    loop {
        let Some(value) = current.child_by_field_name("value") else {
            return false;
        };
        match value.kind() {
            // `current` is the field expression directly on `self`; when
            // it's the call target itself (`self.method()`) there is no
            // intermediate field.
            "self" => return current.id() != function.id(),
            "field_expression" => current = value,
            _ => return false,
        }
    }
}

/// Counts `call_expression` nodes in `node`'s subtree, `node` included —
/// used to tell a genuine single pass-through call (count 1) from a
/// combinator/builder chain whose receiver or arguments are themselves
/// built from further calls (count > 1). Descends into closure bodies too,
/// since a computed predicate/mapper (`.filter(|x| x.is_valid())`) is
/// exactly the kind of real work a delegating wrapper shouldn't be doing.
fn count_calls(node: Node) -> usize {
    let mut count = if node.kind() == "call_expression" {
        1
    } else {
        0
    };
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        count += count_calls(child);
    }
    count
}

/// Reduces a call target's text (`bar`, `self.inner.bar`, `Type::bar`) to
/// its final segment, the name candidates are indexed by.
fn simple_call_name(text: &str) -> String {
    text.rsplit(['.', ':'])
        .next()
        .unwrap_or(text)
        .trim()
        .to_string()
}

/// Container-qualified name (mod/impl/trait ancestor names + own name),
/// re-derived locally per `data_clumps`/`dead_exports`'s precedent rather
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
        .unwrap_or("<anonymous>");
    parts.push(own_name.to_string());
    parts.join("::")
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, MiddlemanDelegationError> {
    let mut files = Vec::new();
    let mut builder = ignore::WalkBuilder::new(root);
    builder.add_custom_ignore_filename(crate::detectors::IGNORE_FILE_NAME);
    for entry in builder.build() {
        let entry = entry.map_err(|source| MiddlemanDelegationError::Walk {
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

    fn candidates_for(content: &str) -> Vec<Candidate> {
        let tree = parse_rust(content, Path::new("f.rs")).unwrap();
        let mut out = Vec::new();
        collect_candidates(
            tree.root_node(),
            content.as_bytes(),
            Path::new("f.rs"),
            false,
            &mut out,
        );
        out
    }

    #[test]
    fn a_free_function_that_bare_tail_calls_another_is_a_delegate() {
        let candidates = candidates_for("fn foo() { bar() }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_free_function_that_explicitly_returns_a_call_is_a_delegate() {
        let candidates = candidates_for("fn foo() { return bar(); }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_free_function_with_a_semicolon_terminated_call_is_a_delegate() {
        let candidates = candidates_for("fn foo() { bar(); }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn a_method_call_through_a_self_field_is_a_composition_facade_not_a_delegate() {
        // Phase 5 (plan-patina-detect-fp-fixes): presenting an owned
        // component's behavior as the type's own API is ordinary
        // composition, the audit's dominant FP family (e.g.
        // `UiState::activate_leader` over the pub `leader` field).
        let candidates = candidates_for("fn foo() { self.inner.bar() }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_function_with_more_than_one_statement_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() { let x = 1; bar(x) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_function_doing_real_work_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() -> i32 { 42 }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_combinator_chain_is_not_a_delegate() {
        // Real FP shape found during Phase 10 real-repo verification
        // (contribution 005, decision 1): `determine_line_relevance_with_precedence`
        // -shaped body — one top-level `call_expression` (`.unwrap_or(...)`)
        // whose receiver is itself built from `.iter().filter().map().min()`.
        let candidates = candidates_for(
            "fn foo(xs: &[i32]) -> i32 { \
             xs.iter().filter(|x| **x > 0).map(|x| *x).min().unwrap_or(0) }",
        );
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_builder_chain_is_not_a_delegate() {
        // Real FP shape found during Phase 10 real-repo verification:
        // `PathBuf::from(...).join(...)`-shaped body.
        let candidates = candidates_for("fn foo() -> String { String::from(\"a\").add(\"b\") }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_call_whose_argument_is_itself_a_call_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() { bar(baz()) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_lone_method_call_on_a_field_with_a_simple_argument_is_a_facade_too() {
        // Reverses contribution 005's expectation: the Phase 5 audit
        // reclassified `self.state.get_reviewable_diff(id)`-shaped wrappers
        // as composition facades, not middlemen.
        let candidates = candidates_for("fn foo(id: u32) -> i32 { self.state.bar(id) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_trait_declared_method_is_excluded_entirely() {
        let candidates = candidates_for("trait Greeter { fn greet(&self) { bar() } }");
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_trait_impl_method_is_excluded_entirely() {
        let candidates = candidates_for(
            "trait Greeter { fn greet(&self); }\n\
             struct Thing;\n\
             impl Greeter for Thing { fn greet(&self) { bar() } }",
        );
        assert!(candidates.is_empty());
    }

    #[test]
    fn an_inherent_impl_method_is_a_candidate() {
        let candidates = candidates_for("struct Thing;\nimpl Thing { fn make() { bar() } }");
        assert_eq!(candidates[0].qualified_name, "Thing::make");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn forwarding_own_parameters_in_order_is_a_delegate() {
        let candidates = candidates_for("fn foo(a: i32, b: i32) -> i32 { bar(a, b) }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("bar"));
    }

    #[test]
    fn an_argument_borrowing_a_self_field_is_an_adapter_not_a_delegate() {
        // Real FP shape from the Phase 5 audit: `TriageApp::process_key_event`
        // forwarding `(&self.data, &mut self.baseline, &mut self.ui_state, key)`
        // to a free `_impl` fn — the wrapper adapts `self` into field borrows
        // so a trait-impl caller with a fixed signature can use it.
        let candidates = candidates_for(
            "struct App; impl App { fn process(&mut self, key: u32) -> u32 { \
             imp(&self.data, key) } }",
        );
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_literal_argument_is_partial_application_not_a_delegate() {
        // Real FP shape from the Phase 5 audit: `serialize_phase_6` pinning
        // `"code_impact"` before forwarding.
        let candidates = candidates_for("fn foo(x: u32) { bar(\"code_impact\", x) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn an_identifier_argument_that_is_not_a_parameter_is_not_a_delegate() {
        let candidates = candidates_for("fn foo() { bar(global) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_macro_argument_is_not_a_delegate() {
        // Real FP shape from the Phase 5 audit: `create_help_line` passing
        // `vec![Span::styled(..)]` — calls inside macro token trees are
        // invisible to `count_calls`, so the argument check must catch it.
        let candidates = candidates_for("fn foo() -> L { L::from(vec![1, 2]) }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn passing_self_through_is_still_a_delegate() {
        let candidates =
            candidates_for("struct S; impl S { fn to_json(&self) -> J { ser(self) } }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("ser"));
    }

    #[test]
    fn a_delegate_through_a_self_field_is_a_composition_facade() {
        let candidates =
            candidates_for("struct S; impl S { fn go(&mut self) { self.leader.activate() } }");
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_deep_self_field_receiver_chain_is_a_composition_facade() {
        let candidates = candidates_for(
            "struct S; impl S { fn go(&mut self) { self.state.leader.activate() } }",
        );
        assert_eq!(candidates[0].delegate_target, None);
    }

    #[test]
    fn a_plain_self_method_call_is_still_a_delegate() {
        // No intermediate field — forwarding to another method of the same
        // type is genuine middleman shape, not composition.
        let candidates = candidates_for("struct S; impl S { fn go(&self) { self.raw_go() } }");
        assert_eq!(candidates[0].delegate_target.as_deref(), Some("raw_go"));
    }

    #[test]
    fn a_tuple_field_receiver_is_a_composition_facade() {
        // Real FP shape from the Phase 5 audit: `ListFilter::push_back` →
        // `self.0.push(c)`.
        let candidates = candidates_for(
            "struct F(String); impl F { fn push_back(&mut self, c: char) { self.0.push(c) } }",
        );
        assert_eq!(candidates[0].delegate_target, None);
    }
}
