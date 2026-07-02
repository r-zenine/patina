# Context Handoff - Phase 0 Test Design (TDD)

> Commit: `b221460` — code + stubs; all pre-existing suites green, fmt/clippy clean.

## 🎯 Core Result
**Built**: The DrillNav behavioral contract as 38 state-driven tests across `tests/drillnav_navigation_tests.rs` (21), `drillnav_approval_tests.rs` (9), `drillnav_note_tests.rs` (8), plus the compiling surface: `DrillNavState`/`FileView` types, 14 inert `UiState` methods (`state.rs:479-583`), and additive StateSnapshot v2 fields.
**Key insight**: `InputTestHarness` consumes the app, so engine-side effects (approvals, notes) can't be read back through it. The shared `tests/drillnav_common/mod.rs` module solves this: `drive_app` feeds parsed keys through public `ReviewTuiApp::process_key_event`, then `into_review_engine` reclaims the engine for assertions — zero new surface on frozen APIs. It also documents the audited decision→file→chunk map of the fixtures.

## 🚦 Current State
**✅ Solid foundation**: 29 tests fail behaviorally, 9 pass vacuously with inert stubs (clamp/no-op invariants — weak signal until Phase 1). Every failure is behavioral, none compile errors. All 12 pre-existing suites green; zero warnings.
**⚠️ Needs attention**: None blocking — Phase 1 can start immediately.
**⏸️ Deferred**: No visual assertions anywhere in the drillnav files (deliberate) — visual coverage arrives in Phase 2 when existing visual suites are rewritten. Old StateSnapshot fields retained; Phase 1 drops only those no surviving test reads.

## 👥 Next Agent Guidance
**Phase 1 implementer**:
- Fill the 14 stub methods **with their exact signatures** — they are the frozen contract (contribution decision #1). Success = all 38 drillnav tests green (they are all state-driven; nothing waits for Phase 2).
- Build the precomputed decision→files→chunks index in `ReviewTuiApp::initialize_ui_state` (plan D6); don't query the engine per keystroke.
- D7 error path: an unresolvable approve target (e.g. `a` over zero decisions) must surface via `set_status_message`, not silently no-op — `test_failed_approval_surfaces_status_message_and_next_key_clears_it` is the binding contract (decision #4).
- Paging methods are `drill_page_up/drill_page_down`, not the roadmap's `page_up/down` — the old names carry live scroll_offset semantics until Phase 4 (decision #2).
- `drill_nav` and `status_message` are **private** UiState fields (V4); keep access behind methods.
**Phase 4 cleanup agent**: may reclaim the `page_up/down` names after deleting the old scroll pair; fixtures needed no extension (decision #6), so nothing in `main.rs` hardcoded decisions is phase-0-specific.

## 🔗 Integration Points
**Expects**: mock fixtures + hardcoded decisions in `src/main.rs` unchanged (decision 1 spans 2 files, decision 3 spans 3, decision 2 is single-file/single-chunk); note tests target `calculator.rs` (file idx 1) because `reader.rs` has two chunks with identical start lines (ambiguous cursor order).
**Provides**: full key-table contract coverage (j/k/h/l, Enter, Esc, Tab, i, n, a, Ctrl-d/u, g/G, Space leader, ?, q), StateSnapshot v2 fields for state assertions, `drillnav_common::drive_app` for engine-state tests.

## 📋 Reference Links
- [decision-log.yaml](decision-log.yaml) — 6 decisions with code impacts
