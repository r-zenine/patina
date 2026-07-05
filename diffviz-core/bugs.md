# Active Bugs - diffviz-core

## ✅ Fixed: Myers Diff Drops Lines on Duplicate-Line Inputs

**Issue (historical)**: In `shortest_edit_script_semantic` (`renderable_diff/myers_diff.rs`), the greedy
"snake" loop advances only `y` while `x` stays fixed, comparing the same old line against
successive new lines. Correct Myers extends diagonally (`x += 1; y += 1` while `a[x] == b[y]`).
Whenever one old line equals several consecutive new lines (duplicate blank lines, repeated
statements, repeated `}` lines), the recorded diagonal run is invalid and the backtrack
produces an edit script that silently drops lines.

**Impact (historical)**:
- Rendered diffs were missing lines — reviewers saw incomplete/incorrect diffs
- `["a"]` → `["a","a","a"]` produced an **empty** diff
- Inserting a blank line next to an existing blank line showed `a();` as deleted and never showed the addition
- Every `Modified` boundary rendered through this path

**Affected Languages**: All (language-agnostic diff engine bug)

**Test Location**: `tests/bug_myers_diff_drops_duplicate_lines.rs`
- `repeated_statement_insertion_reconstructs_both_sources()` — [PASSING] ✅
- `blank_line_insertion_reconstructs_both_sources()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 2 — `myers_diff.rs` deleted wholesale and replaced
by `renderable_diff/line_diff.rs`, a `similar`-backed engine (Patience algorithm) with
index-carrying `DiffOp`s and a separate `align_by_anchors` post-pass. The reconstruction
property test (`tests/diff_reconstruction.rs`) is unignored and green over 1024 proptest cases.

---

## ✅ Fixed: Decompose Path Drops Units Ending on the Range's End Line

**Issue (historical)**: `line_to_byte_offset(source, end_line)` returned the byte offset of
the **start** of `end_line`, and `find_contained_units_recursive` required
`node_range.end <= end_byte`. A unit whose last line was the range's end line was not
"contained" and was silently omitted. `tests/bug_decompose_path_unchanged_units.rs` worked
around this with a "load-bearing" trailing blank line in its fixture (comment there is now
stale — kept for history, the workaround is no longer required).

**Impact (historical)**:
- A range covering exactly two complete functions yielded only 1 ReviewableDiff
- For single-line ranges, `start_byte == end_byte` — the contained query was an empty
  interval (this is why the `find_units_touching_range_recursive` fallback existed)

**Affected Languages**: All

**Test Location**: `tests/bug_range_end_line_excluded.rs`
- `range_covering_two_functions_yields_two_diffs()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 3 — `LineIndex::byte_range_of_lines` is now the
only place an inclusive line range converts to a half-open byte range; the end of the range
is the end of `end_line` (not its start).

---

## ✅ Fixed: Same-Named Units Pair Across Containers

**Issue (historical)**: `find_semantic_unit_by_name` (`decision_based_diff.rs`) matched
old-tree counterparts by (unit-type discriminant, name text) over a flat scan and returned
the first hit, ignoring container context. With `impl A { fn get }` and `impl B { fn get }`,
a change to `B::get` was paired against `A::get`.

**Impact (historical)**:
- The old/new diff shown to the reviewer was fiction (wrong old counterpart)
- If the new body of `B::get` happened to equal `A::get`'s body, the identical-content skip
  dropped the unit entirely
- Fired constantly in Rust (`fn new`, `fn get`, trait impls)

**Affected Languages**: All (worst in Rust/Go where same-named methods are idiomatic)

**Test Location**: `tests/bug_same_name_cross_container_pairing.rs`
- `modified_method_pairs_with_same_impl_counterpart()` — [PASSING] ✅
- `same_named_fns_in_sibling_modules_do_not_mispair()` — [PASSING] ✅
- `method_and_free_function_with_same_bare_name_do_not_mispair()` — [PASSING] ✅
- `renamed_container_surfaces_as_addition_not_bogus_match()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 4 — unit identity is now (container-qualified
name, unit type) rather than (bare name, unit type). `SemanticNode`/`OwnedNodeData` carry a
`qualified_name` (`"Type::name"` / `"mod::name"`), built by threading `parent_context`
through `build_impl_container`/`build_module_container` (now chained, supporting nesting)
and computed for `Callable`/`DataStructure`/`Variable` units. `find_semantic_unit_by_name`
matches on this qualified key instead of bare name.

**Known limitation** (decision D007, unchanged): same qualified name + same unit type at the
same nesting level (e.g. TypeScript declaration merging) still first-matches. Accepted, not
fixed by this phase.

---

## ✅ Fixed: Python Module-Level Assignments Never Become Variable Units

**Issue (historical)**: In tree-sitter-python, `X = 1` at module level parses as
`expression_statement → assignment`. `GenericSemanticTreeBuilder::build_typed_node`
classified `expression_statement` as `Statement` and dropped it **without recursing into
children**, so the `("assignment", Variable)` entry in `PYTHON_SEMANTIC_KIND_MAP` was
unreachable dead code.

**Impact (historical)**:
- Ranges over module-level constants failed with `NoUnitsInRange` instead of yielding diffs
- Same non-recursion pattern hid anything nested under classification-only kinds

**Affected Languages**: Python (same mechanism affects other languages' wrapped constructs)

**Test Location**: `tests/bug_python_module_level_assignment_invisible.rs`
- `range_over_module_level_constants_yields_variable_units()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 5 — a new `LanguageDescriptor::statement_wrapper_kinds()`
hook lists node kinds with no semantic value of their own (Python's `expression_statement`);
`build_container_children` splices such a wrapper's children directly into the enclosing
container instead of classifying (and dropping) the wrapper.

---

## ✅ Fixed: line_range Off by One for Nodes Starting at Column 0

**Issue (historical)**: `SourceCode::line_range_from_bytes` (`ast_diff/source.rs`) computed
the start line as `prefix.lines().count()`, but `str::lines()` ignores a trailing newline. A
node starting at column 0 of line N (prefix ends with `'\n'` — the common case for top-level
items) reported `start_line = N-1`. Mid-line offsets were correct, which is why this survived
casual testing.

**Impact (historical)**:
- Every `RenderableDiff.overall_line_range` went through this path (boundary nodes are
  `OwnedNodeData` with no tree-sitter position info) — reported line ranges were shifted

**Affected Languages**: All

**Test Location**: `tests/bug_line_range_column_zero_off_by_one.rs`
- `node_starting_at_column_zero_reports_correct_start_line()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 3 — both endpoints of `line_range_from_bytes` now go
through `LineIndex::byte_to_line` (`partition_point` over a newline-start table), replacing
the `str::lines().count()` undercount.

---

## ✅ Fixed (all 8 languages): Class Bodies Have No Semantic Children — Methods Invisible to Range Lookup

**Issue (historical)**: `GenericSemanticTreeBuilder::build_data_structure` never collected
children, so for class-based languages every method inside a class was absent from the
semantic tree. Rust escaped only because methods live in `impl` blocks, special-cased as
Module containers that recurse. The unused `container_body_field` descriptor hook existed
to fix exactly this.

**Impact (historical)**:
- A range over one method resolved to the entire class — one-method changes produced
  whole-class diffs, defeating step-by-step review

**Affected Languages**: Python, TypeScript (Phase 5); JavaScript, Java, C++ (Phase 6). C has
no methods (structs are fields-only) — unaffected, verified by regression test. Go has no
class bodies (methods are top-level with a receiver) — unaffected, verified in Phase 5.

**Test Location**: `tests/bug_class_bodies_have_no_semantic_children.rs`
- `range_over_python_method_resolves_to_method_not_class()` — [PASSING] ✅
- `tests/container_recursion_core_languages.rs` —
  `typescript_method_inside_class_resolves_to_method_not_class()` — [PASSING] ✅
- `tests/container_recursion_remaining_languages.rs` —
  `javascript_method_inside_class_resolves_to_method_not_class()`,
  `java_method_inside_class_resolves_to_method_not_class()`,
  `cpp_method_inside_class_resolves_to_method_not_class()`,
  `cpp_function_inside_namespace_resolves_to_function_not_namespace()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 5 — `build_data_structure` now recurses into the
container body via `container_body_field`, passing the data structure's own qualified path
as `parent_context` for its members. The decompose-path trigger in `decision_based_diff.rs`
(previously hardcoded to `Module` only) now also recognizes a `DataStructure` with populated
children as a recursable container (`is_recursable_container`) — without this, classes would
build children but the range lookup would still resolve to the whole class. A childless
`DataStructure` (Rust struct/enum, Go struct/interface — kinds that don't wire
`container_body_field`) is unaffected, preserving existing single-unit expand behavior
exactly. Phase 6 wires the same mechanism for JavaScript (`class_declaration`), Java
(`class_declaration`/`interface_declaration`/`enum_declaration`), and C++
(`class_specifier`/`struct_specifier`, newly classified as `Struct` alongside the pre-existing
`Class`, plus `namespace_definition` newly classified as `Module` — previously C++ namespaces
fell through to an unclassified `Unknown`-wrapper fallback that worked by accident for the
expand path but not for decompose). C is verified unaffected (no methods to hide).

**Plan**: `plan-core-hardening` Phase 6 (remaining 4 languages: JavaScript, Java, C, C++).

---

## ✅ Fixed: CRLF Line Endings Cause Byte-Offset Drift

**Issue (historical)**: `split_into_lines_with_positions` (line_utils.rs) and the Modified rendering
path advanced offsets by `line.len() + 1`, but `str::lines()` strips both `\r` and `\n`.
On CRLF sources, each line's byte range drifted one byte earlier per preceding line.

**Impact (historical)**:
- Annotation-to-line mapping (relevance, change highlighting) was progressively misaligned
  on Windows-formatted files

**Affected Languages**: All (any CRLF source)

**Test Location**: `tests/bug_crlf_byte_offset_drift.rs`
- `crlf_source_line_byte_ranges_match_actual_offsets()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 2 — added `line_utils::line_byte_spans`, a shared
helper computing terminator-width-accurate (`\n` vs `\r\n`) content-only byte spans per line,
used by both `split_into_lines_with_positions` (single-source path) and the rewritten
Modified-path renderer (which now indexes spans directly instead of accumulating
`+1`-per-line offsets).

---

## ✅ Fixed: Deleted Boundaries Read the Wrong Source

**Issue**: `extract_boundary_name` (name_extractors.rs) and the `overall_line_range`
computation in `RenderableDiff::try_from` always read `new_source`, even for `Deleted`
boundaries whose byte ranges belong to the old file. `line_utils` handled this correctly
via `get_display_node_with_source`; the other two call sites didn't.

**Impact (historical)**:
- Deleted function rendered with fallback name ("function") instead of its actual name
- `overall_line_range` computed against the wrong file

**Affected Languages**: All

**Test Location**: `tests/bug_deleted_boundary_reads_new_source.rs`
- `deleted_function_boundary_name_comes_from_old_source()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 1 (decision D009) — the `get_display_node`
consolidation onto `NodeChangeStatus::display_node_with_source` made every call site
(line_utils, name_extractors, mod.rs's `overall_line_range`) use the correct
Deleted-aware source; the unreachable `ChangeClassification::Deletion` path was
deleted in the same phase.

---

## 🐛 Bug: Mixed Line-Number Coordinate Systems in RenderableDiff

**Issue**: Lines from the Modified rendering path are numbered 1..n relative to the
boundary, and `changed_line_numbers` stores those relative values — while
`metadata.overall_line_range` is file-absolute. Consumers correlating the two mix frames.

**Impact**:
- "Which file lines changed?" answered with boundary-relative numbers that fall outside
  the advertised absolute line range

**Affected Languages**: All

**Test Location**: `tests/bug_mixed_line_number_coordinate_systems.rs`
- `changed_line_numbers_fall_within_overall_line_range()` — [FAILING, #[ignore]] 🐛

**Plan**: explicitly **out of scope** for `plan-core-hardening` (decision D012) —
unifying the coordinate frame is a TUI-affecting behavior change, not a hardening
fix, and needs its own plan with `diffviz-review` involvement. Left filed and
`#[ignore]`d exactly as-is; do not fix or rescope under this plan.

---

## ✅ Fixed: Callable parameter_count Counted Comma Tokens

**Issue**: `build_callable` computed `parameters_node.child_count() - 2`, which removed the
parens but counted `,` separators. `fn f(a: i32, b: i32)` reported parameter_count = 3.

**Impact (historical)**:
- Wrong metadata on every multi-parameter Callable (harmless in practice only because both
  sides of comparisons were computed the same wrong way)

**Affected Languages**: All

**Test Location**: `tests/bug_parameter_count_includes_commas.rs`
- `two_parameter_function_reports_parameter_count_two()` — [PASSING] ✅

**Fixed by**: `plan-core-hardening` Phase 1 (micro-simplification) — switched to
`named_child_count()`.

---

# Fixed Bugs

## ✅ Bug: Parent-Child Node Deletion Overlap (FIXED)

**Issue**: When a parent AST node is deleted (e.g., a class declaration), the algorithm reported BOTH:
1. The parent node deletion as a separate semantic pair
2. All child node deletions (e.g., method definitions) as individual semantic pairs

This created redundant/overlapping semantic pairs that represented the same structural change.

**Impact**:
- Review systems showed the same deletion at multiple nesting levels
- Duplicate review items for logically identical changes
- Confused reviewers about what actually changed

**Affected Languages**: All languages (language-agnostic Tree-sitter bug)

**Test Location**: `tests/bug_parent_child_deletion_overlap.rs`
- `typescript_class_to_functional_refactor_overlap()` - [PASSING] ✅
- `rust_struct_impl_deletion_overlap()` - [PASSING] ✅

**Solution**: Modified `build_semantic_pairs()` and `build_semantic_pairs_with_coverage()` in `semantic_ast.rs` to call `mark_node_and_children_as_used()` after creating deletion/addition pairs in Phase 2. This ensures that when a parent node is processed as a deletion/addition, all its children are marked as used and won't create separate redundant pairs.

**Changes**:
- `semantic_ast.rs` lines 990-1001: Added child marking for `build_semantic_pairs()`
- `semantic_ast.rs` lines 1089-1100: Added child marking for `build_semantic_pairs_with_coverage()`

**Behavior Change**:
- Before: Multiple overlapping deletion pairs for parent + all children
- After: Single deletion pair for parent node (children implicitly included)

---

## ✅ Bug: Rust `impl` Blocks Not Classified as Semantic Units (FIXED)

**Issue**: The old `RustParser` flattened `impl` blocks — methods were promoted as module-level
siblings and the `impl_item` node itself was never wrapped in a `SemanticNode`. Byte ranges
covering an impl header escalated to the file-level Module node (100–150× expansion).

**Affected Languages**: Rust only

**Test Location**: `tests/bug_rust_impl_block_not_classified.rs`
- `impl_block_range_should_not_expand_to_module()` — [PASSING] ✅
- `impl_block_method_should_resolve_to_impl_not_module()` — [PASSING] ✅

**Fix**: Phase 1 of the parser refactor. `RustDescriptor` maps `impl_item` → `ImplBlock` in
`RUST_SEMANTIC_KIND_MAP` and sets `container_body_field("impl_item") = Some("body")`.
`GenericSemanticTreeBuilder` recurses into the body, producing an `ImplBlock` node that wraps
its `function_item` children. Ranges inside impl blocks now resolve to the `ImplBlock` node,
not the file-level module.

---

## ✅ Bug: Struct Declaration Range Expansion (FIXED)

**Issue**: `SemanticNode` for a struct reported a byte range covering only its name/fields,
not the full struct including attributes and `pub` keyword. `find_semantic_unit_at_range()`
missed ranges that touched the keyword or leading attribute.

**Affected Languages**: Rust

**Test Location**: `tests/bug_struct_range_expansion.rs`
- `test_struct_declaration_range_should_expand_to_full_struct()` — [PASSING] ✅
- `test_decision_log_scenario_struct_range_should_expand()` — [PASSING] ✅

**Fix**: Phase 1 of the parser refactor. `GenericSemanticTreeBuilder::build_semantic_node()`
uses the tree-sitter node's own `.byte_range()`, which covers the complete source span including
leading modifiers and decorators already gathered as `metadata_nodes`.

---

## ✅ Bug: TypeScript Files Classified as "New File" Instead of "Modified" (Parser Layer) (FIXED)

**Issue**: `TypeScriptParser::build_semantic_tree()` returned an error for valid TypeScript source,
causing the review layer to fall back to a "new file" classification.

**Affected Languages**: TypeScript, TSX

**Test Location**: `tests/bug_typescript_file_classification.rs`
- `test_typescript_modified_file_classification()` — [PASSING] ✅
- `test_typescript_file_type_detection()` — [PASSING] ✅
- `test_typescript_new_files_work_correctly()` — [PASSING] ✅

**Fix**: Phase 2 of the parser refactor. `TypeScriptDescriptor` correctly maps TypeScript
tree-sitter node kinds (`function_declaration`, `class_declaration`, `interface_declaration`,
`type_alias_declaration`, etc.) and `GenericSemanticTreeBuilder` produces a valid `SemanticTree`
for all TypeScript source.

---

## ✅ Bug: JavaScript Files Shown as Unsupported Language Error (FIXED)

**Issue**: `JavaScriptParser` was a stub that returned `SemanticError::UnsupportedLanguage` for
all input, causing the review layer to surface a spurious "unsupported language" error for `.js`
and `.jsx` files.

**Affected Languages**: JavaScript, JSX

**Test Location**: `tests/bug_javascript_error_message.rs`
- `test_javascript_modified_files_should_not_show_error()` — [PASSING] ✅
- `test_javascript_new_files_work_correctly()` — [PASSING] ✅
- `test_cross_language_modified_file_error_pattern()` — [PASSING] ✅

**Fix**: Phase 2 of the parser refactor. `JavaScriptDescriptor` implements the full
`LanguageDescriptor` trait mapping JS node kinds (`function_declaration`, `class_declaration`,
`arrow_function`, `import_statement`, `export_statement`, etc.). JavaScript is now a fully
supported language in the semantic pipeline.
