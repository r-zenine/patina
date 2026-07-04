# Design Document - Keybinding Registry Shape

> **Target**: < 100 lines total
> **Note**: Constraints were pre-pinned by plan decision D004 (plain static data,
> no proc-macros) and the Phase 0 characterization suite (behavior is frozen).

## Decision: One Static Table, Scope-Aware Matching, Doc-Only Catch-Alls

A single `static BINDINGS: &[Binding]` in `diffviz-review-tui/src/events/bindings.rs`.
Each row: `scope`, `keys` (alias list of `KeySpec { code, modifiers }`), `event`
(a concrete `UiEvent` value), `notation` (display strings, e.g. `["j", "<Down>"]`),
`description`. Matching semantics live in the lookup function and vary by scope,
not per row. Parametric behaviors are NOT dispatch rows — they are code fallbacks
plus doc-only entries so manifest/affordances still document them.

## Why This Design

**Constraints That Led Here:**
- D004: plain static data, no proc-macros, no LazyLock needed (all row parts are
  const-constructible: `UiEvent` variants, `KeyCode`, `KeyModifiers::NONE`)
- Phase 0 pins two quirks the table must reproduce: leader matching ignores
  modifiers, and unknown leader keys map to `DeactivateLeader`
- `InputChar(c)` and `EnterLeaderSubmenu(c)` look parametric, but only
  `InputChar` truly is — submenus are exactly `'a'` and `'t'`, i.e. two
  concrete rows

**Simplicity Rationale:**
Per-scope matching (exact `(code, modifiers)` for Navigation/Input; `code`-only
for Leader scopes) keeps rows dumb data. Encoding match strategy per row would
add a field every consumer must understand to reproduce two special cases.

## How It Works

**Key Interfaces (all in `events/bindings.rs`):**
- `BindingScope`: `Navigation | LeaderRoot | LeaderSubmenu(char) | Input` —
  computed from `(InputMode, leader_active, leader_submenu)` by `scope_of(...)`
- `lookup(scope, key) -> Option<&'static Binding>`: iterates `BINDINGS`,
  filters by scope, matches per-scope semantics
- `CATCH_ALLS: &[CatchAllDoc]`: doc-only rows — Input "type any character"
  (`InputChar`), Leader "any other key closes the menu" (`DeactivateLeader`)
- `SUBMENUS: &[SubmenuDoc]`: `('a', "Actions")`, `('t', "Toggles")` — titles for
  which-key and grouping in the help overlay

**Core Pattern:**
`handle_key_event` = `scope_of` → `lookup` → clone event, with two coded
fallbacks reproducing Phase 0 behavior: Leader scopes fall back to
`DeactivateLeader`; Input falls back to `InputChar(c)` for `Char` with
NONE/SHIFT only (table rows like Ctrl-c win because lookup runs first).

**Integration Points (the five consumers):**
- Dispatch: `events/input.rs::handle_key_event` becomes the ~15-line flow above
- Which-key: `ui/components/which_key.rs` renders `LeaderRoot`/`LeaderSubmenu`
  rows + submenu titles from `SUBMENUS`
- Help overlay: `ui/components/help_overlay.rs` renders rows grouped by scope
- Manifest: `app.rs::describe()` maps every row + catch-all to `KeyBindingDoc`
  (`event` label = `format!("{:?}", event)`)
- Affordances: `app.rs::affordances()` = rows for `scope_of(current ui_state)`
  + that scope's catch-alls

## What We're NOT Doing

**Rejected Alternatives:**
- **Per-row match-strategy field**: generalizes two quirks into a concept every
  consumer must handle; scope-based semantics is smaller
- **`HashMap`/`LazyLock` index**: ~40 rows; linear scan is measurement-free
  simplicity, and `find` preserves declaration-order precedence
- **Fixing the leader modifier-blindness quirk**: behavior change; Phase 0
  forbids it here (revisit post-Phase 3 with its own decision + test updates)
- **strum-style variant enumeration for coverage tests**: new dependency to
  automate what a duplicate-row + description invariant test covers well enough

**Out of Scope:**
- sam-tui registry (Phase 5 keeps hand-rolled affordances per D006)
- User-configurable rebinding (nothing consumes it; table centralization is the
  enabling step if it ever comes)

## Implementation Guidance

**For Next Contributor:**
- Build `bindings.rs` from the current match arms in `events/input.rs:90-336`,
  row for row — the Phase 0 suite is the oracle, do not edit it
- Convert `which_key.rs`/`help_overlay.rs` after dispatch is green, then
  `describe()`/`affordances()`, then flip the Phase 2 empty-bindings test

**Testing Strategy:**
- Phase 0 characterization suite passes unchanged (primary gate)
- New invariants: no duplicate `(scope, KeySpec)`; every description non-empty
- Affordance scope tests + which-key visual regression via `--test-full "<Space>"`

**Success Criteria:**
- `rg "KeyCode::Char" diffviz-review-tui/src/events/input.rs` shows only the
  scope routing and the two documented fallbacks — no binding arms left
