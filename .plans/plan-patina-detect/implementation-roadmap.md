# Implementation Roadmap — patina-detect

**Strategy**: TDD — every phase's acceptance criteria are assertions written
before the code that satisfies them (fingerprint tests, threshold fixtures,
FP-control fixtures). Each phase is a complete, independently shippable
deliverable; the workspace builds and is warning-free at every phase
boundary.

**Total phases**: 17 (0–16), deliberately small — favor more, smaller phases
over few large ones per explicit instruction. Phases 0–2 are specified in
full detail; phases 3–15 carry enough to start, deferring fine-grained
decisions per Last Responsible Moment (thresholds/evidence shapes are already
pinned in `docs/patina-detect/spec.md` — each phase cites its section instead
of repeating it). Phase 16 is a revision of Phase 6, added after Phase 6's
own real-repo verification surfaced a false-positive shape (decision D011);
it depends on Phase 9's call hierarchy and so must run after it despite its
position at the end of this list. Line references are in `code-context.md`.

**Standing acceptance criteria for every phase** (not repeated below):
`cargo build --workspace` / `cargo test --workspace` green, `cargo clippy
--workspace` and `cargo fmt --all -- --check` clean, no other crate's
compilation breaks unless the phase explicitly says so.

---

## Phase 0 — Generalize the TUI contribution skill + extract shared components

**Deliverable**: One skill governs both `diffviz-review-tui` and the future
`patina-detect-tui`; the drillnav pattern and four render-only components live
in `tui-design`, generic over a trait instead of `Reviewable`/`Decision`.
`diffviz-review-tui` itself is behavior-unchanged (its own test suite is the
witness).

### Objectives

1. **Generalize the skill**: rewrite `.claude/skills/diffviz-tui-contribution/`
   into a framework-wide skill (e.g. `tui-framework-contribution`). Same five
   rules (pure views / Command system / time-as-events / state encapsulation
   / registry-only key matching) and same 8 rejection criteria, but phrased
   against the framework crates with per-crate path patterns (e.g. "view
   functions in `*-tui/src/ui/**/*.rs` must take `&UiState`" rather than
   hardcoding `diffviz-review-tui/src/ui/`). Keep `diffviz-review-tui`'s own
   onboarding/testing specifics (its `--describe` output, its keybinding
   file) as a thin per-crate addendum, not duplicated rule text.
2. **Extract the drillnav pattern** into `tui-design`: split
   `drillnav_common.rs`'s generic helpers (`content_rect`, `wrap_text`,
   `make_card`, `dot_pagination_line`) from its Decision-specific ones
   (`note_for`, `note_rows`, `line_change_type`) — only the generic half
   moves. Define a trait (e.g. `DrillableItem`/`Site`-shaped) that
   `drillnav_browse.rs`/`drillnav_drill.rs` render against, with
   `diffviz-review-tui` supplying the `Reviewable`/`Decision` impl.
3. **Extract the four render-only components** (`which_key.rs`,
   `help_overlay.rs`, `status_bar.rs`, `input_modal.rs`) into `tui-design`,
   binding against `tui-elm`'s existing state/logic (`which_key.rs`,
   `help.rs`, `registry.rs`, `leader.rs`) rather than `diffviz-review-tui`
   types.
4. **Rewire `diffviz-review-tui`** to consume the extracted versions from
   `tui-design`; delete the now-empty originals from
   `diffviz-review-tui/src/ui/components/`.

### Acceptance Criteria
- `diffviz-review-tui`'s full test suite (`cargo test --package
  diffviz-review-tui --features test-harness`) passes unchanged — no
  assertion in `tests/dispatch_characterization_tests.rs` is edited to make
  this phase pass (skill rejection criterion 8 applies to itself here).
- `cargo run --bin review-tui -- --describe` output is byte-identical before
  and after (proves the extraction didn't change behavior).
- The generalized skill file contains zero hardcoded `diffviz-review-tui`
  paths in its rule/rejection-criteria sections.

---

## Phase 1 — patina-detect entity model + baseline persistence

**Deliverable**: New `patina-detect` and `patina-detect-tui` crates registered
in the workspace; the `Symptom`/`Evidence`/`Site`/`TriageVerdict` entity model
exists with a working `SymptomId` content-addressing scheme and a persisted
baseline that filters already-triaged symptoms out of re-runs. No detector
yet — this phase is pure entity/persistence plumbing, proven with synthetic
symptoms in tests.

### Objectives

1. **Workspace registration**: add `patina-detect`, `patina-detect-tui` to
   `Cargo.toml` members + workspace.dependencies.
2. **Entity model** (`patina-detect/src/entities/`): `Symptom`, `Evidence`
   (as a trait or enum — decide at implementation, detectors will each add a
   variant/impl), `Site`, `SiteRole`, `TriageVerdict` per
   `docs/patina-detect/spec.md:22-40`.
3. **`SymptomId`**: `detector_id: DetectorId` + opaque fingerprint bytes,
   hashed (not ordinal). Define the hashing contract as a trait/function each
   detector phase implements against (`fn fingerprint(&self) -> Vec<u8>` or
   similar) — this phase does NOT design any detector's actual fingerprint
   (that's each detector phase's own job, decision D007).
4. **Baseline persistence**: a triage-state store keyed by `SymptomId`
   (format TBD at implementation — JSON is the safe default, matches
   `serde_json` already in the workspace); a detector-engine run function
   that takes raw `Symptom`s + the persisted baseline and returns only the
   ones with no verdict yet.
5. **Symptom-log export**: the decision-log-YAML-shaped debug export the spec
   calls out as "cheap interop, not the model" — minimal, one function.

### Acceptance Criteria
- Unit tests: a symptom with a persisted `Dismissed`/`Fix` verdict is
  excluded from a synthetic re-run; a symptom with no verdict passes through;
  two runs producing the same fingerprint produce the same `SymptomId`
  (survives being constructed twice).
- `patina-detect` compiles with dependencies limited to `diffviz-core` and
  `lspkit` (no `diffviz-review` dependency — verify with `cargo tree
  --package patina-detect`).

---

## Phase 2 — Detector 1: house-rule violations (ast-grep)

**Deliverable**: First real detector producing real `Symptom`s from an
initial rule pack, via an ast-grep adapter module inside `patina-detect`. A
`patina-detect` binary can run `detect`/`export`.

See `docs/patina-detect/spec.md:116-132` for the full rule pack, FP-control
notes, and evidence shape (rule id + matched snippet).

### Objectives

1. Add `ast-grep-core`/`ast-grep-config`/`ast-grep-language` to workspace
   dependencies, pinned lockstep versions, consumed only by `patina-detect`.
2. Adapter module (`patina-detect/src/detectors/house_rules/`): loads YAML
   rule files at runtime (same files usable by the ast-grep CLI), runs them
   against the target tree, maps matches to `Symptom`s.
3. Initial rule pack (as YAML, scoped to `diffviz-core` paths where the spec
   says so): fallbacks (`unwrap_or_default()`, `.ok()` discarding errors,
   catch-all `_ =>` defaults), stringly-typed errors (`map_err(|e|
   e.to_string())`), `let _ = <Result>`, `#[allow(dead_code)]`/`todo!`/
   `unimplemented!` outside tests.
4. Fingerprint: rule id + normalized match text (not line numbers) — per
   decision D007, this detector's fingerprint design is due now, not later.
5. `patina-detect` binary: `detect --audit` (whole-repo) and `export`
   subcommands (per decision D009 — no umbrella CLI).

### Acceptance Criteria
- Running against `diffviz-core` produces symptoms for at least one
  known-present violation of each rule-pack category (verify by grep first,
  then assert the detector finds it).
- Re-running with no code change and no baseline produces identical
  `SymptomId`s (fingerprint stability test).
- `export` produces valid decision-log-shaped YAML.

---

## Phase 3 — patina-detect-tui bootstrap

**Deliverable**: A working triage TUI — browse detector-1 findings, drill
into sites, dismiss or fix, verdict persists via Phase 1's baseline. Built on
`tui-harness`/`tui-elm`/`tui-design` plus Phase 0's extracted drillnav/render
components.

Governed by the Phase-0-generalized skill from its first line of code.

### Objectives

1. New `patina-detect-tui` binary (e.g. `triage-tui`) bootstrapped on
   `tui-harness`'s ELM runtime, mirroring `diffviz-review-tui`'s
   update/Command/event-registry split (`app.rs`/`command.rs`/
   `events/{bindings,business,input}.rs`) with `patina-detect` types instead
   of `ReviewEngine`/`Reviewable`.
2. Browse view: list of `Symptom`s (title + evidence summary) using the
   Phase-0-extracted drillnav browse component.
3. Drill view: `Site`s for the selected symptom, rendered via
   `diffviz-core`'s range-based rendering (audit mode, `old_source: None`,
   per decision D006).
4. Verdict actions: `Dismissed { reason }` / `Fix { instruction }`, wired to
   Phase 1's persistence.
5. `--describe`/`--test-input`/`--agent-repl` all work (inherited from
   `tui-harness`; verify, don't assume).

### Acceptance Criteria
- `cargo run --bin triage-tui -- --test-full "<sequence>"` drives browse →
  drill → dismiss end-to-end against detector-1 output and shows the
  persisted verdict on next `detect` run (symptom no longer surfaces).
- Skill rejection criteria all pass (no `&mut UiState` in views, no inline
  I/O in handlers, registry-only key matching).

---

## Phase 4 — Detector 2: Type-2 clones

**Deliverable**: Clone-group detector via subtree hashing; audit-mode
side-by-side rendering verified for real (not just single-unit).

See `docs/patina-detect/spec.md:134-148`.

### Objectives

1. Fixture test first: two independent `SemanticNode` subtrees (from
   separate `new_source` calls, neither "old" for the other) rendered
   side-by-side via `create_reviewable_diff_from_range`/`RenderableDiff` —
   resolve the caveat from decision D006 before building on top of it.
2. Subtree hashing: re-derive the placeholder-substitution normalization
   `is_semantically_identical` used (identifiers/literals → placeholders,
   structure + node kinds retained) fresh in `patina-detect` — don't depend
   on the function `plan-core-hardening` is deleting.
3. FP controls: minimum ~30 semantic nodes, whole-function granularity only,
   cross-file groups ranked above same-file, all-test-code groups tagged
   separately.
4. Fingerprint: clone group's subtree hash (per decision D007).
5. Symptom shape: one symptom per group, one `Site`/`SiteRole::CloneMember`
   per member.

### Acceptance Criteria
- Two independently-written near-identical functions in different files
  produce one clone-group symptom; a trivial 3-line function does not
  (min-size gate).
- Renders correctly in `patina-detect-tui`'s drill view (manual or
  `--test-full` check).

---

## Phase 5 — Detector 5: cognitive complexity extremes

**Deliverable**: Sonar-spec cognitive complexity scoring over tree-sitter
nodes, per-language increment tables on `LanguageDescriptor`.

See `docs/patina-detect/spec.md:179-192`.

### Objectives

1. Per-language increment/nesting-increment tables as a new
   `LanguageDescriptor` method (or a small parallel trait, decide at
   implementation — follow the existing descriptor pattern).
2. Scorer: +1 per branch, +1 extra per nesting level, per the Sonar spec.
3. Threshold ≥25; ranked output (score descending).
4. Fingerprint: function's qualified name + normalized body hash (survives
   pure line-number drift; a fully-rewritten function gets a new id, which
   is fine — the finding is about the content, not identity across rewrites).

### Acceptance Criteria
- A known deeply-nested function in the workspace (find one via manual
  inspection first) scores ≥25 and is reported; a match-heavy but flat Rust
  dispatch function does not false-positive.

---

## Phase 6 — Detector 8: data clumps

**Deliverable**: Parameter-clump detector with the forwarding-chain
precision gate.

See `docs/patina-detect/spec.md:226-248`.

### Objectives

1. Signature clump extraction: ≥3 normalized (name, type) parameters
   co-occurring in ≥3 distinct signatures, stripping `&`/`mut`.
2. Trait-impl dedup: count once per trait declaration, not per impl (impl-
   block context available syntactically).
3. Forwarding-gate: promote only when a clump is forwarded intact to another
   call (match call-site argument identifiers against enclosing parameters).
4. Bonus evidence: if the clump's field set is a subset of an existing
   struct's fields, surface it in the rationale.
5. Fingerprint: normalized clump member set (sorted name+type pairs) +
   detector id — survives parameter reordering.

### Acceptance Criteria
- A synthetic 3-signature clump that's forwarded intact through 2 call
  levels is reported; the same clump with no forwarding (each signature just
  uses the params locally) is not (per decision to drop non-traveling clumps
  in v1).

---

## Phase 7 — lspkit: JSON-RPC transport + `references`

**Deliverable**: `LspClient` performs a real initialize/initialized
handshake, runs a background reader thread dispatching responses by request
id, and `references` returns real data against `rust-analyzer`.

### Objectives

1. Transport: request/response framing over the LSP's `Content-Length`
   header protocol on the spawned process's stdio.
2. Handshake: `initialize` request with capabilities, `initialized`
   notification, per LSP spec minimum viable payload for rust-analyzer.
3. Background reader thread + request-id dispatch (replaces the `TODO` at
   `libs/lspkit/src/lib.rs:152-154`).
4. Wire `references` (`native.rs:18-24`) end-to-end.
5. Leave `hover`/`definition`/`implementations`/`document_symbols`/
   `workspace_symbols`/`prepare_call_hierarchy`/`incoming_calls`/
   `outgoing_calls` as `todo!()` — out of scope for this phase.

### Acceptance Criteria
- Integration test against a small fixture Rust project: `references` on a
  known symbol returns the known call sites, run against a real
  `rust-analyzer` process (mark `#[ignore]`d if CI lacks rust-analyzer,
  document the requirement).
- `LspClient::start` blocks until initialize completes or times out with a
  typed error (no silent hang).

---

## Phase 8 — Detector 3: dead exports and write-only code

**Deliverable**: `pub` symbol/field usage detector via `references`.

See `docs/patina-detect/spec.md:150-163`.

### Objectives

1. Enumerate `pub` symbols + struct fields from `SemanticTree`.
2. `references` per candidate; count external-crate vs same-crate,
   read vs write (fields).
3. Exclusion list: trait-impl methods, derive-heavy struct fields, bin entry
   points.
4. Test-only usage tagged, not dropped.
5. Fingerprint: qualified symbol path + detector id.

### Acceptance Criteria
- A genuinely dead `pub fn` in a low-traffic module is found; a trait-impl
  method with the same reference-count shape is excluded.

---

## Phase 9 — lspkit: call hierarchy

**Deliverable**: `prepare_call_hierarchy`, `incoming_calls`, `outgoing_calls`
wired end-to-end.

### Objectives

1. Wire the three methods (`native.rs:38-48`) against real `rust-analyzer`.
2. Extend the integration-test fixture from Phase 7 with a small call chain
   (A calls B calls C) to assert hierarchy resolution.

### Acceptance Criteria
- `incoming_calls`/`outgoing_calls` on the fixture's known chain return the
  expected callers/callees.

---

## Phase 10 — Detector 4: middleman delegation chains

**Deliverable**: Delegation-chain detector via the tree-sitter ∧ lspkit
conjunction.

See `docs/patina-detect/spec.md:165-177`.

### Objectives

1. Tree-sitter: functions whose body is a single delegating call.
2. Call hierarchy: confirm exactly one caller (same-crate requirement).
3. Skip trait methods (may satisfy an interface).
4. Chain composition when middlemen compose (A → B → C).
5. Fingerprint: normalized chain signature (sorted qualified names in the
   chain) + detector id.

### Acceptance Criteria
- A synthetic single-caller delegating wrapper is reported; the same shape
  with 2+ callers is not.

---

## Phase 11 — Detector 6: near-duplicate data structures

**Deliverable**: Struct-similarity clustering with the conversion-evidence
gate. Reuses `references` from Phase 7 — no new lspkit method.

See `docs/patina-detect/spec.md:194-210`.

### Objectives

1. Per-struct field multisets of normalized (name, type); Jaccard ≥0.7,
   ≥4 shared fields.
2. Conversion-evidence gate: `references` on both type symbols to find
   `impl From`/A→B signature conversion functions.
3. Evidence: overlap %, shared/total fields, conversion sites, footprint
   (files touching both).
4. Fingerprint: sorted pair of qualified struct names + detector id.

### Acceptance Criteria
- Two structs with ≥70% field overlap AND an existing hand-rolled
  conversion function are reported; the same overlap with zero conversion
  code is not (gate holds).

---

## Phase 12 — lspkit: `definition` + `hover`

**Deliverable**: Both wired end-to-end (`native.rs:10-16`).

### Objectives

1. Wire `definition`, `hover` against real `rust-analyzer`.
2. Extend the fixture with a `match`/`switch` site over a known enum to
   support Phase 13's scrutinee-type resolution.

### Acceptance Criteria
- `definition` on the fixture's match scrutinee resolves to the enum
  declaration; `hover` returns its signature.

---

## Phase 13 — Detector 7: parallel dispatch

**Deliverable**: Enum-match-site detector via definition/hover scrutinee
resolution.

See `docs/patina-detect/spec.md:212-224`.

### Objectives

1. Tree-sitter: collect `match`/`switch` sites.
2. Resolve scrutinee type via `definition`/`hover`; group by type.
3. Threshold: ≥3 sites across ≥2 files; exclude std types (`Option`,
   `Result`, per-language equivalents).
4. Evidence: site count, file count, arm-count per site.
5. Fingerprint: qualified enum name + detector id.

### Acceptance Criteria
- An enum matched in ≥3 sites across ≥2 files is reported; the same enum
  matched only twice, or only in one file, is not.

---

## Phase 14 — lspkit: `implementations`

**Deliverable**: Wired end-to-end (`native.rs:26-28`).

### Objectives

1. Wire `implementations` against real `rust-analyzer`.
2. Extend the fixture with a trait + single implementor.

### Acceptance Criteria
- `implementations` on the fixture's trait returns exactly its one
  implementor.

---

## Phase 15 — Detector 9: single-impl traits

**Deliverable**: Speculative-generality detector, last per the spec's own FP
rationale (test-double/DI pattern needs care).

See `docs/patina-detect/spec.md:250-259`.

### Objectives

1. Enumerate traits from `SemanticTree`; `implementations` count.
2. Exclusions: test-double impls, traits appearing only in `dyn`/generic
   bounds inside test code, marker/sealed traits.
3. Evidence: impl count, impl locations, whether test doubles exist.
4. Fingerprint: qualified trait name + detector id.

### Acceptance Criteria
- A trait with one production impl and one test-double impl is correctly
  excluded (DI pattern); a trait with genuinely one impl and no test double
  is reported.

---

## Phase 16 — Detector 8 revision: call-hierarchy-based forwarding-gate refinement

**Deliverable**: Data-clumps' forwarding-intact precision gate (Phase 6)
additionally excludes closed recursive/mutually-recursive clusters (e.g. a
private recursive-descent visitor's own helper functions forwarding
`(node, accumulator...)` state to each other) using lspkit's call hierarchy
— a false-positive shape found during Phase 6's own real-repo verification
(a subagent review classified `cognitive_complexity/detector.rs`'s
`score_node`/`score_if`/`score_match` `(node, nesting, max_nesting_depth)`
clump as a dismissible false positive of exactly this shape, and flagged it
as a systematic gap, not a one-off). See decision D011.

### Objectives

1. For each promoted clump group (Phase 6's existing threshold/forwarding
   checks still apply first), call `incoming_calls` (Phase 9,
   `lspkit::native::incoming_calls`) on every occurrence in the group.
2. Closed-cluster check: if every caller of every occurrence in the group is
   itself a member of the same group (no external caller anywhere), the
   clump is a self-contained recursive/mutually-recursive family — exclude
   it. If at least one occurrence has a caller outside the group, keep it
   (real independent call sites are passing the clump around, regardless of
   whether they all happen to live in the same file).
3. This supersedes any file/module-scope heuristic as the FP control for
   this shape — a same-file clump called from genuinely distinct external
   sites must not be excluded, which a module-scope gate would incorrectly
   do (this was the reason LSP was chosen over the cheaper heuristic,
   decision D011).
4. Land as a Phase 6 revision contribution
   (`NNN-phase-6-revision-code-[agent]`, per the standard revision pattern),
   not a new detector — `data_clumps`' existing entity shapes
   (`Evidence::DataClump`, `SiteRole::ForwardingSite`) are unchanged.

### Acceptance Criteria
- A synthetic closed recursive-helper family (mirroring
  `score_node`/`score_if`/`score_match`'s shape: private functions in one
  module, forwarding a clump to each other, no caller from outside the
  family) is excluded.
- A synthetic clump forwarded across genuinely distinct call sites that
  happen to share a file is still reported (proves the gate doesn't
  regress to the rejected file/module heuristic).
- Re-running detector 8 against this repo's own codebase no longer reports
  the `cognitive_complexity` scorer's clump (the known false positive this
  phase exists to fix).

---

## Deferred (explicitly out of scope — see spec + decision log)

- Feature envy, test theater, naming drift, arbitrary-block clones (spec's
  own "Deferred" section) — not mechanically decidable or too FP-prone yet.
- Churn hotspots as a standalone detector — ranking input only, not a
  symptom.
- Instruction cascade auto-resolve-vs-confirm semantics (decision D008) —
  revisit once `patina-detect-tui` verdict UI is mature.
- Any unified `patina` umbrella CLI (decision D009) — per-crate binaries for
  now.
