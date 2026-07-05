# Patina Detect — Symptom Catalog and Design Spec

Deterministic detectors that surface review-worthy *symptoms* in agent-generated
code. No LLM anywhere in the detection path: detectors produce findings with
raw evidence, a human triages them, and the existing review tooling turns
triage into remediation instructions.

Status: design spec. Nothing here is implemented yet.

## Concept

A **Symptom** borrows the `Decision` (diffviz-review) UX concept with the
semantics reversed: decisions flow author → reviewer ("here is why I did
this"), symptoms flow detector → triager ("here is what looks wrong"). Both
are annotations over line ranges, and the review UX transfers directly:
navigate finding → drill into sites → read code → verdict + instruction.

Symptoms are **modeled first-class** — not shoehorned into Decision entities.
What Decision expresses loosely (freeform `rationale`, undifferentiated
`code_impacts`), Symptom types properly:

```
Symptom {
    id: SymptomId,            // content-addressed: detector id + stable
                              // fingerprint (e.g. clone subtree hash), NOT
                              // ordinal — survives re-runs and rebases
    detector: DetectorId,
    title: String,            // "Near-duplicate structs: A / B (87% overlap)"
    evidence: Evidence,       // typed per detector: scores, counts, rule id
    sites: Vec<Site>,
}
Site {
    file, line_ranges,
    role: SiteRole,           // CloneMember, ConversionSite, MatchSite,
                              // Definition, Caller, ... — per-detector roles
    note: String,             // what to scrutinize at these lines
}
TriageVerdict = Dismissed { reason: FalsePositive | Intentional | AcceptedDebt }
              | Fix { instruction }   // single-note model, as in diffviz-review
```

Typed `evidence` and `SiteRole` are what the Decision format could not
express: the TUI can render clone members side by side, sort by score, and
show "conversion site" vs "definition" differently — the enrichment path.

### Delivery: dedicated TUI on the framework crates

Two new workspace crates:

- **`patina-detect`** — the domain crate: Symptom/Evidence/Triage entities,
  detector engine, persistence (triage state keyed by `SymptomId`), symptom-
  log serialization. Consumes diffviz-core and lspkit. Mirrors the
  entities/engines layout of diffviz-review.
- **`patina-detect-tui`** — the triage UI, bootstrapped on `tui-harness`
  (ELM runtime, headless testing, `--describe`/`--agent-repl`), `tui-elm`
  (keybinding registry, leader state, which-key/help overlays, affordances),
  and `tui-design` (theme, cards, scroll). The framework provides the entire
  interaction and agent-testability layer; the crate adds only Symptom state,
  events, and views. Code display at symptom sites reuses diffviz-core's
  range-based rendering pipeline.

```
patina detect --against <commit> | --audit   →   symptoms + evidence
patina triage                                →   patina-detect-tui
     verdicts: dismiss (reason) / fix (instruction)
patina export                                →   instructions JSON → agent
```

The decision-log YAML export stays available as a cheap interop/debug format,
but it is an export, not the model.

## Cross-cutting design rules

1. **Tree-sitter surfaces, LSP confirms.** Candidates are enumerated cheaply
   from `SemanticTree` (diffviz-core); targeted lspkit queries
   (`references`, `callHierarchy`, `implementations`) fire only at candidates.
   Never sweep the repo with LSP requests — request volume stays proportional
   to findings, not repo size.
2. **Evidence in the rationale.** Every symptom carries the numbers that
   produced it (node count, similarity, caller count, matched rule id) so
   triage is a glance, not an investigation.
3. **Confidence tiers.** Detectors with a corroborating-evidence gate report
   gated findings as symptoms; ungated ones go to a low-confidence appendix or
   are dropped (v1: dropped).
4. **Test code is tagged, not dropped.** Findings whose members are all in
   test code are reported under a separate category.
5. **Baseline ratchet.** Persisted triage verdicts, keyed by content-addressed
   `SymptomId`, are the suppression baseline; re-runs surface only symptoms
   absent from the baseline. Fingerprint design per detector is therefore a
   day-one requirement, not a retrofit (see Open questions).
6. **Layering.** Detection lives in `patina-detect`, triage UI in
   `patina-detect-tui`. `patina-detect` consumes diffviz-core
   (`SemanticTree`) and lspkit; nothing detector-related enters diffviz-core.
   The ast-grep runtime never touches diffviz-core (its tree-sitter version
   differs from ours; the two node worlds do not mix).

## Foundations

- **diffviz-core** — `SemanticTree` (byte-coverage invariant, 11 languages),
  node normalization (`is_semantically_identical`), `LanguageDescriptor` kind
  tables, range-based rendering (`create_reviewable_diff_from_range()`).
- **lspkit** (`libs/lspkit`) — references, call hierarchy, implementations,
  hover, document symbols, `BlastRadius`.
- **TUI framework** (`libs/tui-harness`, `libs/tui-elm`, `libs/tui-design`) —
  ELM runtime, headless agent testing (`--describe` / `--test-input` /
  `--agent-repl`), single-registry keybindings with which-key/help/affordances,
  theme and card components. The triage TUI is a thin domain layer on top.
- **ast-grep crates** — `ast-grep-core` + `ast-grep-config` +
  `ast-grep-language` (0.x, lockstep versions, pin exactly). Rules stay as
  YAML files loaded at runtime; the same files work with the ast-grep CLI in
  editors/CI.
- **git** — churn signals (ranking input only, not a detector).

## The nine symptoms (build order)

### 1. House-rule violations

- **Foundation:** ast-grep (embedded crates)
- **Signal:** code that violates written project law (CLAUDE.md rules), so a
  hit is a violation by definition — near-zero FP.
- **Initial rule pack:**
  - fallbacks in diffviz-core: `unwrap_or_default()`, `.ok()` discarding
    errors, silent `else` defaults, catch-all `_ =>` arms returning defaults
  - stringly-typed errors: `map_err(|e| e.to_string())`, `format!` into error
    values
  - `let _ = <Result>` swallowing errors
  - `#[allow(dead_code)]`, `todo!` / `unimplemented!` outside tests
- **FP control:** none needed beyond rule scoping (some rules apply only to
  diffviz-core paths).
- **Evidence:** rule id, matched snippet.
- **Note:** this is also the accumulation point — every future "agents keep
  doing X" observation becomes one more YAML rule for free.

### 2. Type-2 clones (re-implemented helpers)

- **Foundation:** tree-sitter only
- **Signal:** an agent didn't find the existing helper and wrote it again —
  the signature agent smell.
- **Method:** hash each function-sized `SemanticNode` subtree with identifiers
  and literals replaced by placeholders (structure + node kinds only); bucket
  by hash; report groups. Builds on the normalization already in
  `is_semantically_identical`.
- **FP controls:** minimum size ≥ ~30 semantic nodes (tiny functions collide
  constantly); whole-function granularity only in v1 (arbitrary-block clones
  are a noisier later extension); cross-file groups ranked above same-file;
  all-test-code groups tagged separately.
- **Evidence:** group size, node count, member locations.
- **Symptom shape:** one symptom per clone group, one `CodeImpact` per member.

### 3. Dead exports and write-only code

- **Foundation:** lspkit (references), tree-sitter to enumerate candidates
- **Signal:** helpers written "for the future" — `pub` items nothing outside
  the defining crate references; fields written but never read. In a closed
  workspace (no external API consumers) this is near-zero FP.
- **Method:** enumerate `pub` symbols and struct fields from `SemanticTree`;
  `references` per candidate; count external/read references.
- **FP controls (mechanical exclusion list):** trait-impl methods (referenced
  through the trait), fields on derive-heavy structs (serde uses them
  invisibly), bin entry points. Usage that exists only in tests is tagged
  "test-only", not dropped — production code only tests exercise is its own
  finding.
- **Evidence:** reference count by crate, read/write split for fields.

### 4. Middleman delegation chains

- **Foundation:** tree-sitter ∧ lspkit (the conjunction is the point)
- **Signal:** pointless wrapper layers agents introduce; expected highest hit
  rate after clones.
- **Method:** tree-sitter finds functions whose body is a single delegating
  call; call hierarchy confirms exactly one caller. Either signal alone is
  noisy; together almost always real.
- **FP controls:** the conjunction itself; same-crate requirement; skip trait
  methods (delegation may satisfy an interface).
- **Evidence:** body shape, caller count, the chain (A → B → C) when middlemen
  compose.
- **Symptom shape:** one symptom per chain, impacts at each link.

### 5. Cognitive complexity extremes

- **Foundation:** tree-sitter only
- **Signal:** long functions with heavy nesting — missing helpers/abstractions.
- **Method:** Sonar cognitive-complexity spec (+1 per branch, +1 extra per
  nesting level) over tree-sitter node kinds; per-language increment /
  nesting-increment tables fit the `LanguageDescriptor` pattern.
- **FP controls:** threshold **≥ 25** (Sonar's default 15 flags fine code;
  Rust match-heavy dispatch inflates scores); report ranked by score so triage
  reads top-down.
- **Deliberately excluded from v1:** extraction-candidate analysis
  (live-in/live-out block detection) — doubles the work; raw score plus human
  eyes gets ~90% of the value.
- **Evidence:** score, function length, max nesting depth.

### 6. Near-duplicate data structures

- **Foundation:** tree-sitter (surface) + lspkit (evidence)
- **Signal:** almost-alike structs that fragment logic — the *almost* is the
  problem; 100% duplicates at boundaries are often fine.
- **Method:** per-struct field multisets of normalized (name, type); cluster
  by Jaccard similarity ≥ ~0.7 with ≥ 4 shared fields. Then lspkit gathers
  evidence: references on both type symbols → conversion functions
  (`impl From`, A→B signatures), functions accepting either type, spread
  footprint (files touching both).
- **FP controls:** **conversion-evidence gate** — only pairs with actual
  conversion code become symptoms. Legitimate pairs (serde request/response
  types, intentional persistence mirrors of domain entities — this repo does
  that by design) mostly lack hand-rolled conversion churn or get dismissed
  once into the baseline.
- **Evidence:** overlap %, shared/total fields, conversion sites, footprint.
- **Symptom shape:** impacts at both structs plus each conversion site.

### 7. Parallel dispatch

- **Foundation:** tree-sitter (match sites) + lspkit (scrutinee type)
- **Signal:** the same enum matched everywhere — logic spread that suggests
  missing polymorphism or a missing method on the enum. Covers the
  "logic spread" smell even with a single data structure.
- **Method:** collect `match`/`switch` sites; resolve scrutinee type via LSP
  definition/hover; group by type; flag enums matched in **≥ 3 sites across
  ≥ 2 files**.
- **FP controls:** the site/file thresholds; exclude std types
  (`Option`, `Result` and per-language equivalents).
- **Evidence:** site count, file count, arm-count per site.
- **Symptom shape:** one symptom per enum, one impact per match site.

### 8. Data clumps

- **Foundation:** tree-sitter only
- **Signal:** the missing struct *before* it exists — precursor to symptom 6
  (which is the missing struct after it got created twice).
- **Method:** groups of **≥ 3 parameters** (normalized name + type, stripping
  `&`/`mut`) co-occurring in **≥ 3 distinct signatures**. Pairs are hopeless
  (`(path, line)` recurs legitimately everywhere).
- **Key exclusion:** trait-impl methods mirror their trait declaration by
  necessity — count a signature once per trait, not once per impl
  (impl-block context is available syntactically). Same for interface
  implementations in TS/Java/Go.
- **Precision gate (same philosophy as symptom 4):** promote to symptom only
  when the clump is *forwarded intact* — a function receives the group and
  passes all of it to another call. A clump threaded through ≥ 2 signature
  levels is a struct being passed by hand; detectable by matching call-site
  argument identifiers against enclosing parameters. Non-traveling clumps:
  dropped in v1.
- **Bonus evidence (cheap, check always):** if the clump's name/type set is a
  subset of an existing struct's fields, the abstraction already exists —
  rationale says "consider passing `X` instead" and triage is instant.
- **Evidence:** clump members, occurrence count, forwarding chain, subset-of
  struct if any.

### 9. Single-impl traits

- **Foundation:** lspkit (implementations)
- **Signal:** speculative generality — a trait with exactly one implementor.
- **Method:** enumerate traits from `SemanticTree`; `implementations` count.
- **FP controls (why it ranks last):** exclude traits with test-double impls
  and traits appearing in `dyn`/generic bounds inside test code — the
  Environment/DI pattern legitimately has one production impl plus test
  doubles. Also exclude marker/sealed traits.
- **Evidence:** impl count, impl locations, whether test doubles exist.

## Deferred (and why)

- **Feature envy** (function referencing another module's symbols more than
  its own) — higher FP, needs tuning.
- **Test theater** (assertion-free tests, over-mocked tests) — judgment-heavy;
  mechanical versions are noisy.
- **Naming drift** (`fetch_`/`get_`/`retrieve_` for one concept) — not
  mechanically decidable.
- **Arbitrary-block clones** (type-2 below function granularity) — noisier
  extension of symptom 2.
- **Churn hotspots** — not a defect claim, so not a symptom; use later as a
  *ranking input* (a clone in high-churn code outranks one in frozen code).

## Open design questions

1. **Fingerprint design per detector.** `SymptomId` must survive re-runs,
   rebases, and code motion, so it hashes detector id + a *normalized*
   fingerprint — never line numbers. Some are obvious (a clone group's subtree
   hash, a rule id + normalized match), some need care (what identifies a data
   clump when a parameter is renamed? a complexity finding when the function
   is split?). Per-detector decision, made when each detector is built; a
   changed fingerprint means the old verdict is dropped and the symptom
   resurfaces, which errs on the safe side.
2. **Audit vs diff-scoped runs.** Since the triage TUI is not anchored to a
   git diff, whole-repo audit is the natural base mode; `--against <commit>`
   is a filter keeping symptoms whose sites overlap the change. Confirm
   diffviz-core's range-based rendering behaves when displaying current-tree
   code with no "old side".
3. **Instruction cascade.** diffviz-review tracks instructions whose target
   code changed (`InstructionStatus`). The equivalent here: a `Fix` verdict
   whose symptom disappears on re-run (fixed, or fingerprint drifted) —
   decide whether it auto-resolves or asks for confirmation.

## Sequencing rationale

Build order = value per false positive, constrained by foundation readiness:
the ast-grep pack pays off immediately while everything else is built; clones
and complexity are pure diffviz-core and give `LanguageDescriptor` its first
consumers outside diff-building (a pressure test of the descriptor
abstraction); the LSP detectors follow lspkit's maturity, cheapest-safest
first (references → call hierarchy → implementations).

The TUI bootstraps in parallel with detectors 1–2: two detectors' worth of
real findings (rule violations + clone groups) is enough to design the triage
views against, and the framework crates make the shell cheap. The Symptom
entity model in `patina-detect` comes first, since both the detectors and the
TUI depend on it.

One thing worth noting for when you start: the diffviz-tui-contribution skill and its rejection criteria are written for diffviz-review-tui specifically, but everything it enforces (pure views, command system, registry-only key matching) is now framework law — the new TUI should adopt the same rules from commit one, and a sibling skill or a generalization of that skill would be worth creating alongside the bootstrap.
