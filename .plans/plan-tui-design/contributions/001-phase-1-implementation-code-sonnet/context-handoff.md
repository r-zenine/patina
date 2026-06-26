# Context Handoff — Phase 1

## What was built

A `tui-design` workspace crate with:
- `palette.rs` — re-exports `catppuccin::PALETTE` and exposes `mocha()` helper
- `tokens.rs` — `SurfaceRamp([Color; 8])` (Index<usize> only), `AccentPalette` (14 catppuccin names), `Theme { surface, accents }` with `Theme::mocha()` constructor
- `stylesheet.rs` — 25 composable style functions (border, title_active, keybind_key, cursor, diff_added, file_renamed, etc.) all taking `&Theme`
- `icons.rs` — `Icons` constants moved from diffviz-review-tui/theme.rs

`diffviz-review-tui` migration:
- `theme.rs` deleted
- `theme_ext.rs` added: `DiffvizSurface` extension trait on `SurfaceRamp` (`panel_bg`, `diff_gutter_bg`, `annotation_text`, `inactive_border`, `focused_border`)
- All 9 component files migrated to `tui_design::{Theme, stylesheet, Icons}`
- The 4 bg-highlight violations fixed: modal `bg(Colors::BLACK)` removed, 3 cursor instances replaced with `stylesheet::cursor(theme)`

Infrastructure change:
- ratatui bumped 0.28 → 0.30 (catppuccin 2.8 ratatui feature requires ratatui-core Color, which only aligns with `ratatui::style::Color` from 0.30 onwards)
- `tui-harness` and `sam-tui` pinned versions updated to `{ workspace = true }`
- `tui-harness/src/error.rs` gained `Infallible` variant (ratatui 0.30 TestBackend is infallible)

## Key invariants for Phase 2 (sam-tui migration)

1. **Pattern is established**: `let theme = Theme::mocha();` at the top of each public render function, pass `&theme` to private helpers. Do the same in sam-tui.

2. **SamSurface extension trait**: create `sam-tui/src/theme_ext.rs` with `SamSurface` (surface indices) and `SamAccents` (accent semantics) as described in the context document. The context-document shows the exact mapping.

3. **UITheme replacement**: `sam-tui/src/modal_view/theme.rs` defines `UITheme` passed by `&UITheme` reference through UI builder structs. The plan says to replace it with a thin newtype or type alias over `tui_design::Theme`. Preserve the by-reference pattern (see D006 in plan decision-log).

4. **No breaking changes to sam-tui public API** — the migration is internal; existing callers pass `&UITheme` and that shape is preserved.

5. **Zero warnings rule** — run `cargo clippy --package sam-tui` after migration and fix all warnings before committing.

## What to watch for

- `sam-tui/src/modal_view/ui_options_mode.rs` uses inline `Style::default().fg(self.theme.borders)` patterns. Each one maps to a stylesheet function — consult the context-document's AccentPalette and SurfaceRamp tables for the right mapping.
- The `headless.rs` and `view.rs` files implement `ELMApp::draw` — these already compile correctly with ratatui 0.30 after the workspace version fix.
