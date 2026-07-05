# Code Context for patina-detect

Line numbers verified against the working tree on 2026-07-05; re-verify before
each phase (this plan spans many phases and the tree will move).

## Source of truth

- **`docs/patina-detect/spec.md`** — the design spec. Every phase below cites
  the relevant "### N. <symptom>" section instead of re-deriving thresholds,
  FP controls, and evidence shapes; read the cited section before implementing.

## Phase 0 target — TUI framework skill + component extraction

- **`.claude/skills/diffviz-tui-contribution/skill.md`** — rules V1-V5 and the
  8 rejection criteria are correct in substance but phrased entirely in
  `diffviz-review-tui` terms (hardcoded paths like `src/app.rs`,
  `src/events/bindings.rs`; domain types `UiState`/`ReviewEngine`). Becomes the
  framework-wide skill.
- **`diffviz-review-tui/src/ui/components/`** — components still coupled to
  Decision/Reviewable that belong in `tui-design`:
  - `drillnav_browse.rs` (108 lines), `drillnav_common.rs` (205 lines),
    `drillnav_drill.rs` (315 lines) — the navigate-list/drill-into-item
    pattern. `drillnav_common.rs` mixes generic helpers (`content_rect`,
    `wrap_text`, `make_card`, `dot_pagination_line`) with Decision-specific
    ones (`note_for`, `note_rows` over `Instruction`, `line_change_type` over
    `RenderableLine`) — the extraction has to split these, not move the file
    wholesale.
  - `which_key.rs` (16 lines), `help_overlay.rs` (28 lines),
    `status_bar.rs` (133 lines), `input_modal.rs` (174 lines) — render-only;
    the corresponding state/logic already lives in `tui-elm`
    (`which_key.rs`, `help.rs`, `registry.rs`, `leader.rs`).
- **`libs/tui-elm/src/`** — `which_key.rs`, `help.rs`, `registry.rs`,
  `leader.rs` (941 lines total) — state/logic layer these render components
  should bind against generically.
- **`libs/tui-design/src/`** — `card.rs`, `scroll.rs`, `stylesheet.rs`,
  `tokens.rs`, `palette.rs` (582 lines total) — destination for the extracted
  generic components.
- **`libs/tui-harness/src/`** — `runtime.rs`, `traits.rs`, `manifest.rs`,
  `agent_cli.rs`, `repl.rs`, `input_parser.rs`, `input_test.rs`,
  `render_test.rs`, `combined.rs`, `error.rs` (1952 lines total) — ELM runtime
  and headless testing; unaffected by this phase, `patina-detect-tui`
  bootstraps directly on it in Phase 3.

## Phase 1 target — entity model + persistence

- No existing code; new crate `patina-detect`. Model per spec's "Concept"
  section (`docs/patina-detect/spec.md:22-40`): `Symptom`, `Site`,
  `TriageVerdict`. `SymptomId` = detector id + normalized fingerprint (open
  design question 1, `spec.md:276-283` — bake fingerprint design into each
  detector phase, not this one; this phase only defines the `SymptomId` type
  and the hashing contract).
- Baseline ratchet mechanism (spec design rule 5, `spec.md:87-90`): re-runs
  must surface only symptoms whose `SymptomId` has no persisted verdict. This
  is the detector-engine's filtering step, built once here, reused by every
  detector phase.

## Phase 2 target — detector 1 (house-rule violations)

- `docs/patina-detect/spec.md:116-132` — full rule pack, FP controls,
  evidence shape.
- **Workspace `Cargo.toml`** (`Cargo.toml:1-73`) — `ast-grep-core`,
  `ast-grep-config`, `ast-grep-language` not yet present; add as pinned 0.x
  versions (lockstep), consumed only by the new adapter module inside
  `patina-detect` (design rule 6, `spec.md:91-95`: "the ast-grep runtime never
  touches diffviz-core").

## Phase 3 target — patina-detect-tui bootstrap

- Built on `tui-harness`/`tui-elm`/`tui-design` post-Phase-0, plus the
  extracted drillnav pattern for finding→site navigation.
- `diffviz-review-tui/src/app.rs`, `src/state.rs`, `src/command.rs`,
  `src/events/{bindings,business,input}.rs` — the Elm-loop shape to mirror
  (update/Command/event-registry split), not code to reuse directly (domain
  types differ).

## Phase 4 target — detector 2 (Type-2 clones)

- `docs/patina-detect/spec.md:134-148` — hashing method, FP controls
  (min size, whole-function granularity, cross/same-file ranking, test-tagging).
- **`diffviz-core/src/semantic_ast.rs:21`** (`SemanticTree`),
  **`:52`** (`SemanticNode`) — subtree hashing builds on the normalization
  already used by (now-deleted, see plan-core-hardening Phase 1)
  `is_semantically_identical`; re-derive the placeholder-substitution logic
  fresh here rather than reviving the deleted function.
- **`diffviz-core/src/decision_based_diff.rs:482`**
  (`create_reviewable_diff_from_range`) — confirmed working for single-unit
  audit rendering with `old_source: None`
  (`diffviz-core/tests/pin_added_boundary_rendering.rs:17-36`). Side-by-side
  rendering of two independent clone members (two separate `new_source`
  calls, neither "old" for the other) is a different call shape, not covered
  by that test — verify with a small fixture test at the start of this phase
  before building the clone-group symptom's site rendering.

## Phase 5 target — detector 5 (cognitive complexity)

- `docs/patina-detect/spec.md:179-192` — Sonar spec increments, threshold
  ≥ 25, per-language increment tables fit `LanguageDescriptor`
  (`diffviz-core/src/parsers/descriptor.rs:15`).

## Phase 6 target — detector 8 (data clumps)

- `docs/patina-detect/spec.md:226-248` — clump/signature/forwarding-chain
  method, trait-impl dedup (count once per trait declaration, not per impl).

## Phase 7 target — lspkit transport + `references`

- **`libs/lspkit/src/lib.rs`** — types fully defined (`Position`, `Range`,
  `Location`, `FileLocation`, `SymbolKind`, `DocumentSymbol`, `Hover`,
  `CallHierarchyItem`, `CallSite`, `PeekResult`, `BlastRadius`); `LspClient::
  start` (`lib.rs:144-160`) spawns `rust-analyzer` but performs no
  initialize/initialized handshake and no background reader thread — the
  `TODO` at `lib.rs:152-154` is the entire gap.
- **`libs/lspkit/src/native.rs`** — every method is `todo!()`
  (`hover:10-12`, `definition:14-16`, `references:18-24`,
  `implementations:26-28`, `document_symbols:30-32`,
  `workspace_symbols:34-36`, `prepare_call_hierarchy:38-40`,
  `incoming_calls:42-44`, `outgoing_calls:46-48`). This phase wires the
  transport/handshake plus `references` only.
- **`libs/lspkit/src/composite.rs`** — `peek_definition`, `blast_radius`,
  `sibling_methods` (all `todo!()`, 28 lines total) — cross-request
  orchestration layer, not needed until multiple native methods exist.

## Phase 8 target — detector 3 (dead exports / write-only code)

- `docs/patina-detect/spec.md:150-163` — exclusion list (trait-impl methods,
  derive-heavy fields, bin entry points), test-only tagging.

## Phase 9 target — lspkit call hierarchy

- `native.rs:38-48` — `prepare_call_hierarchy`, `incoming_calls`,
  `outgoing_calls`.

## Phase 10 target — detector 4 (middleman delegation chains)

- `docs/patina-detect/spec.md:165-177` — tree-sitter ∧ lspkit conjunction,
  same-crate requirement, chain composition.

## Phase 11 target — detector 6 (near-duplicate data structures)

- `docs/patina-detect/spec.md:194-210` — Jaccard clustering + conversion-
  evidence gate. Reuses `references` from Phase 7; no new lspkit method.

## Phase 12 target — lspkit `definition` + `hover`

- `native.rs:10-16`.

## Phase 13 target — detector 7 (parallel dispatch)

- `docs/patina-detect/spec.md:212-224` — scrutinee type resolution via
  definition/hover, ≥3 sites across ≥2 files threshold, std-type exclusion.

## Phase 14 target — lspkit `implementations`

- `native.rs:26-28`.

## Phase 15 target — detector 9 (single-impl traits)

- `docs/patina-detect/spec.md:250-259` — test-double exclusion, marker/sealed
  trait exclusion.

## Workspace registration

- **`Cargo.toml:3-18`** (members), **`:20-37`** (workspace.dependencies) — add
  `patina-detect`, `patina-detect-tui` (Phase 1), `ast-grep-*` (Phase 2).
