//! Review Navigator — three-level hierarchical navigation prototype.
//!
//! DrillNav pattern:
//!   Browse  j/k navigate decisions | a approve/unapprove decision (cascades to chunks)
//!           Enter drill in | q quit
//!   Drill   j/k navigate chunks | h/l cycle files (wraps, per-file state retained)
//!           Tab expand context | i expand/collapse note | a approve chunk
//!           Esc back | q quit
//!
//! Surface ramp (dark theme, lighter = higher elevation):
//!   rationale    → surface1   (CardTier::Header — pinnable)
//!   instructions → surface0   (CardTier::Body)
//!   code lines   → base       (CardTier::Content)
//!   pinned header container → mantle
//!   separator    → mantle     (widget floor — never touches crust/terminal)
//!
//! Layout: content capped at 120 columns, centered; surface bg fills full column width.

use std::collections::HashSet;
use std::io;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diffviz_core::RenderableLine;
use diffviz_core::renderable_diff::ChangeType;
use diffviz_review::{
    DecisionLog, ReviewEngine, ReviewableDiffId,
    entities::git_ref::{DiffQuery, GitRef},
    providers::mock_provider::MockDiffProvider,
    review_engine_builder::ReviewEngineBuilder,
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use tui_design::{
    CardTier, HierarchicalCard, Icons, Theme, render_drill_header, scroll_into_view,
    separator_line, stylesheet,
};

use diffviz_review_tui::events::input::UiEvent;

// ── Engine bootstrap ──────────────────────────────────────────────────────────

fn build_engine() -> Result<ReviewEngine> {
    let yaml = include_str!("resources/decision-log.yaml");
    let decision_log = DecisionLog::parse(yaml)?;

    let query = DiffQuery::head_to_unstaged();

    let mut provider = MockDiffProvider::new();

    provider.add_file_content(
        "src/auth/middleware.rs",
        &GitRef::Head,
        include_str!("resources/auth_middleware_old.rs"),
    );
    provider.add_file_content(
        "src/auth/middleware.rs",
        &GitRef::Unstaged,
        include_str!("resources/auth_middleware_new.rs"),
    );
    provider.add_file_content(
        "src/auth/token.rs",
        &GitRef::Head,
        include_str!("resources/auth_token_old.rs"),
    );
    provider.add_file_content(
        "src/auth/token.rs",
        &GitRef::Unstaged,
        include_str!("resources/auth_token_new.rs"),
    );
    // rate_limiter.rs is a new file — only provide new content
    provider.add_file_content(
        "src/rate_limiter.rs",
        &GitRef::Unstaged,
        include_str!("resources/rate_limiter_new.rs"),
    );

    let mut engine = ReviewEngineBuilder::new(Box::new(provider), "reviewer".into())
        .build_from_decisions(decision_log.decisions, query)?;

    seed_instructions(&mut engine)?;

    Ok(engine)
}

fn seed_instructions(engine: &mut ReviewEngine) -> Result<()> {
    let pairs = engine.get_decision_reviewable_diffs();

    // Note for the validate_token chunk in middleware.rs (decision 1).
    // A chunk holds a single note; bob's follow-up folds into alice's note
    // (single-note model: editing = appending, never a second instruction).
    if let Some(d) = pairs.iter().find(|d| {
        d.chunk_id.file_path() == "src/auth/middleware.rs"
            && d.chunk_id.line_range().start_line <= 20
            && d.decision_number == 1
    }) {
        engine.add_instruction(
            d.chunk_id.clone(),
            "Make sure TokenValidator is object-safe before merging — \
             we'll need Arc<dyn TokenValidator> in the Middleware struct below."
                .into(),
            "alice".into(),
        )?;
        engine.add_instruction(
            d.chunk_id.clone(),
            "Object safety confirmed on my branch — also add a #[deny(missing_docs)] \
             note for the trait."
                .into(),
            "bob".into(),
        )?;
    }

    // Instruction for the Middleware struct chunk (decision 1)
    if let Some(d) = pairs.iter().find(|d| {
        d.chunk_id.file_path() == "src/auth/middleware.rs"
            && d.chunk_id.line_range().start_line > 20
            && d.decision_number == 1
    }) {
        engine.add_instruction(
            d.chunk_id.clone(),
            "DefaultValidator::new() is not yet implemented — \
             this will panic at runtime if the default impl is exercised."
                .into(),
            "bob".into(),
        )?;
    }

    // Instruction for the rate limiter (decision 2)
    if let Some(d) = pairs
        .iter()
        .find(|d| d.chunk_id.file_path() == "src/rate_limiter.rs" && d.decision_number == 2)
    {
        engine.add_instruction(
            d.chunk_id.clone(),
            "Consider extracting retain + len check into a named helper — \
             check_and_record is doing two distinct things."
                .into(),
            "carol".into(),
        )?;
    }

    Ok(())
}

// ── Navigation helpers ────────────────────────────────────────────────────────

/// Sorted unique file paths affected by a decision (preserves sibling_idx mapping).
fn files_for_decision(engine: &ReviewEngine, decision_idx: usize) -> Vec<String> {
    let decisions = engine.get_all_decisions();
    let decision_number = decisions[decision_idx].number;
    let mut files: Vec<String> = engine
        .get_decision_reviewable_diffs()
        .into_iter()
        .filter(|d| d.decision_number == decision_number)
        .map(|d| d.chunk_id.file_path().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    files.sort();
    files
}

/// The chunk's single note. One note per chunk is a product invariant —
/// adding an instruction to an annotated chunk appends to the existing note.
fn note_for<'a>(
    engine: &'a ReviewEngine,
    chunk_id: &ReviewableDiffId,
) -> Option<&'a diffviz_review::Instruction> {
    engine
        .state()
        .get_instructions(chunk_id)
        .and_then(|v| v.first())
}

/// Chunk IDs for a specific decision × file, ordered by start line.
fn chunks_for_file(
    engine: &ReviewEngine,
    decision_idx: usize,
    file_path: &str,
) -> Vec<ReviewableDiffId> {
    let decisions = engine.get_all_decisions();
    let decision_number = decisions[decision_idx].number;
    let mut ids: Vec<ReviewableDiffId> = engine
        .get_decision_reviewable_diffs()
        .into_iter()
        .filter(|d| d.decision_number == decision_number && d.chunk_id.file_path() == file_path)
        .map(|d| d.chunk_id)
        .collect();
    ids.sort_by_key(|id| id.line_range().start_line);
    ids
}

// ── Navigation events ─────────────────────────────────────────────────────────

/// Pure key → event mapping; context-free so it stays testable.
fn handle_key_event(code: KeyCode) -> Option<UiEvent> {
    match code {
        KeyCode::Char('q') => Some(UiEvent::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(UiEvent::NavigateDown),
        KeyCode::Char('k') | KeyCode::Up => Some(UiEvent::NavigateUp),
        KeyCode::Char('h') | KeyCode::Left => Some(UiEvent::NavigateLeft),
        KeyCode::Char('l') | KeyCode::Right => Some(UiEvent::NavigateRight),
        KeyCode::Char('a') => Some(UiEvent::ToggleApprove),
        KeyCode::Char('i') => Some(UiEvent::ToggleInstructions),
        KeyCode::Enter => Some(UiEvent::SelectCurrent),
        KeyCode::Esc => Some(UiEvent::Back),
        KeyCode::Tab => Some(UiEvent::ToggleDecisionExpansion),
        _ => None,
    }
}

// ── Navigation state ──────────────────────────────────────────────────────────

/// Render-time snapshot passed into `render_drill`.
struct DrillContext<'a> {
    node_idx: usize,
    sibling_idx: usize,
    view: &'a FileView,
}

/// Per-file view state, retained while cycling siblings with h/l so a
/// round-trip doesn't lose the reviewer's place.
#[derive(Default, Clone)]
struct FileView {
    /// j/k cursor within the file's chunk list.
    cursor: usize,
    /// Chunks with expanded context (Tab toggles).
    expanded: HashSet<usize>,
    /// Chunks with expanded instruction text (i toggles).
    expanded_notes: HashSet<usize>,
}

/// State machine for the DrillNav pattern.
///
/// `Browse` — exploring top-level nodes as cards.
/// `Drill`  — inside a node: its label+summary is pinned, children scroll below.
enum DrillNavState {
    Browse {
        cursor: usize,
    },
    Drill {
        /// Which top-level node we're inside (Enter drills in, Esc backs out).
        node_idx: usize,
        /// h/l cycling among sibling child groups (files within a decision).
        sibling_idx: usize,
        /// One view state per sibling file, index-aligned with `files_for_decision`.
        views: Vec<FileView>,
    },
}

struct App {
    engine: ReviewEngine,
    commit: String,
    nav: DrillNavState,
    theme: Theme,
    quit: bool,
    /// One-shot error line for the status bar; cleared on the next keypress.
    message: Option<String>,
}

impl App {
    fn new() -> Result<Self> {
        let yaml = include_str!("resources/decision-log.yaml");
        let commit = DecisionLog::parse(yaml)?.commit;
        Ok(App {
            engine: build_engine()?,
            commit,
            nav: DrillNavState::Browse { cursor: 0 },
            theme: Theme::mocha(),
            quit: false,
            message: None,
        })
    }

    fn handle_nav_event(&mut self, event: UiEvent) {
        self.message = None;
        match event {
            UiEvent::Quit => self.quit = true,
            UiEvent::NavigateUp => self.navigate_up(),
            UiEvent::NavigateDown => self.navigate_down(),
            UiEvent::NavigateLeft => self.navigate_left(),
            UiEvent::NavigateRight => self.navigate_right(),
            UiEvent::ToggleApprove => self.toggle_approve(),
            UiEvent::SelectCurrent => self.drill_in(),
            UiEvent::Back => self.back(),
            UiEvent::ToggleDecisionExpansion => self.toggle_context(),
            UiEvent::ToggleInstructions => self.toggle_note(),
            _ => {}
        }
    }

    fn toggle_approve(&mut self) {
        // Read phase: extract values before any mutation so borrows don't overlap.
        enum Action {
            Decision(u32),
            Chunk {
                node_idx: usize,
                sibling_idx: usize,
                cursor: usize,
            },
        }
        let action = match &self.nav {
            DrillNavState::Browse { cursor } => {
                let decisions = self.engine.get_all_decisions();
                Action::Decision(decisions[*cursor].number)
            }
            DrillNavState::Drill {
                node_idx,
                sibling_idx,
                views,
            } => Action::Chunk {
                node_idx: *node_idx,
                sibling_idx: *sibling_idx,
                cursor: views[*sibling_idx].cursor,
            },
        };

        // Mutate phase
        let result = match action {
            Action::Decision(decision_num) => {
                if self.engine.is_decision_approved(decision_num) {
                    self.engine.reject_decision(decision_num).map(|_| ())
                } else {
                    self.engine
                        .approve_decision(decision_num, "reviewer".into())
                        .map(|_| ())
                }
            }
            Action::Chunk {
                node_idx,
                sibling_idx,
                cursor,
            } => {
                let files = files_for_decision(&self.engine, node_idx);
                let file_path = files[sibling_idx].clone();
                let chunk_ids = chunks_for_file(&self.engine, node_idx, &file_path);
                if cursor < chunk_ids.len() {
                    let chunk_id = chunk_ids[cursor].clone();
                    if self.engine.state().is_approved(&chunk_id) {
                        self.engine.reject(chunk_id)
                    } else {
                        self.engine.approve(chunk_id, "reviewer".into())
                    }
                } else {
                    Ok(())
                }
            }
        };
        if let Err(e) = result {
            self.message = Some(format!("Approval failed: {e}"));
        }
    }

    fn navigate_up(&mut self) {
        let cursor = match &mut self.nav {
            DrillNavState::Browse { cursor } => cursor,
            DrillNavState::Drill {
                sibling_idx, views, ..
            } => &mut views[*sibling_idx].cursor,
        };
        if *cursor > 0 {
            *cursor -= 1;
        }
    }

    fn navigate_down(&mut self) {
        let n = match &self.nav {
            DrillNavState::Browse { .. } => self.engine.get_all_decisions().len(),
            DrillNavState::Drill {
                node_idx,
                sibling_idx,
                ..
            } => {
                let files = files_for_decision(&self.engine, *node_idx);
                chunks_for_file(&self.engine, *node_idx, &files[*sibling_idx]).len()
            }
        };
        let cursor = match &mut self.nav {
            DrillNavState::Browse { cursor } => cursor,
            DrillNavState::Drill {
                sibling_idx, views, ..
            } => &mut views[*sibling_idx].cursor,
        };
        if *cursor + 1 < n {
            *cursor += 1;
        }
    }

    fn navigate_left(&mut self) {
        if let DrillNavState::Drill {
            sibling_idx, views, ..
        } = &mut self.nav
        {
            let n = views.len();
            *sibling_idx = sibling_idx.checked_sub(1).unwrap_or(n - 1);
        }
    }

    fn navigate_right(&mut self) {
        if let DrillNavState::Drill {
            sibling_idx, views, ..
        } = &mut self.nav
        {
            let n = views.len();
            *sibling_idx = (*sibling_idx + 1) % n;
        }
    }

    fn drill_in(&mut self) {
        if let DrillNavState::Browse { cursor } = &self.nav {
            let idx = *cursor;
            let n_files = files_for_decision(&self.engine, idx).len();
            self.nav = DrillNavState::Drill {
                node_idx: idx,
                sibling_idx: 0,
                views: vec![FileView::default(); n_files],
            };
        }
    }

    fn back(&mut self) {
        if let DrillNavState::Drill { node_idx, .. } = &self.nav {
            let d = *node_idx;
            self.nav = DrillNavState::Browse { cursor: d };
        }
    }

    fn current_view_mut(&mut self) -> Option<&mut FileView> {
        match &mut self.nav {
            DrillNavState::Drill {
                sibling_idx, views, ..
            } => Some(&mut views[*sibling_idx]),
            DrillNavState::Browse { .. } => None,
        }
    }

    fn toggle_context(&mut self) {
        if let Some(view) = self.current_view_mut() {
            let chunk = view.cursor;
            if !view.expanded.remove(&chunk) {
                view.expanded.insert(chunk);
            }
        }
    }

    fn toggle_note(&mut self) {
        if let Some(view) = self.current_view_mut() {
            let chunk = view.cursor;
            if !view.expanded_notes.remove(&chunk) {
                view.expanded_notes.insert(chunk);
            }
        }
    }
}

// ── Layout helpers ────────────────────────────────────────────────────────────

const CONTENT_WIDTH: u16 = 120;

fn content_rect(area: Rect) -> Rect {
    let w = CONTENT_WIDTH.min(area.width);
    let x = area.x + (area.width - w) / 2;
    Rect {
        x,
        width: w,
        ..area
    }
}

fn plural_s(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Hard-splits a word that can't fit on any line (long paths, URLs) into
/// `max_cols`-sized pieces; words that fit come back whole.
fn split_oversized(word: &str, max_cols: usize) -> Vec<String> {
    if word.chars().count() <= max_cols {
        return vec![word.to_string()];
    }
    let mut pieces = Vec::new();
    let mut chars = word.chars().peekable();
    while chars.peek().is_some() {
        pieces.push(chars.by_ref().take(max_cols).collect());
    }
    pieces
}

fn wrap_text(text: &str, max_cols: usize) -> Vec<String> {
    let max_cols = max_cols.max(1);
    let mut result = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        for piece in split_oversized(word, max_cols) {
            if current.is_empty() {
                current = piece;
            } else if current.chars().count() + 1 + piece.chars().count() <= max_cols {
                current.push(' ');
                current.push_str(&piece);
            } else {
                result.push(std::mem::take(&mut current));
                current = piece;
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

// ── Rendering helpers ─────────────────────────────────────────────────────────

/// Centered dot row on a mantle background — signals h/l sibling navigation.
/// Active dot uses the accent color; inactive dots use overlay0.
fn dot_pagination_line(
    col_width: u16,
    current: usize,
    total: usize,
    theme: &Theme,
) -> Line<'static> {
    let mantle = theme.surface.mantle();
    let active = theme.accents.lavender;
    let passive = theme.surface.overlay0();

    let dot_section = total * 2 - 1; // "● ○ ○" = n dots + (n-1) spaces
    let left_pad = (col_width as usize).saturating_sub(dot_section) / 2;
    let right_fill = (col_width as usize).saturating_sub(left_pad + dot_section);

    let mut spans: Vec<Span<'static>> = Vec::with_capacity(total * 2 + 2);
    spans.push(Span::styled(
        " ".repeat(left_pad),
        Style::default().bg(mantle),
    ));
    for i in 0..total {
        if i > 0 {
            spans.push(Span::styled(" ", Style::default().bg(mantle)));
        }
        let (dot, color) = if i == current {
            ("●", active)
        } else {
            ("○", passive)
        };
        spans.push(Span::styled(dot, Style::default().fg(color).bg(mantle)));
    }
    spans.push(Span::styled(
        " ".repeat(right_fill),
        Style::default().bg(mantle),
    ));
    Line::from(spans)
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    frame.render_widget(
        Paragraph::new("").style(stylesheet::terminal_floor(theme)),
        area,
    );

    let content_area = Rect {
        height: area.height.saturating_sub(1),
        ..area
    };
    let status_area = Rect {
        y: area.bottom().saturating_sub(1),
        height: 1,
        ..area
    };

    match &app.nav {
        DrillNavState::Browse { cursor } => render_browse(frame, content_area, app, *cursor),
        DrillNavState::Drill {
            node_idx,
            sibling_idx,
            views,
        } => render_drill(
            frame,
            content_area,
            app,
            DrillContext {
                node_idx: *node_idx,
                sibling_idx: *sibling_idx,
                view: &views[*sibling_idx],
            },
        ),
    }

    // An error message preempts the keybinding hints until the next keypress.
    let (status, status_style) = if let Some(msg) = &app.message {
        (msg.clone(), stylesheet::error(theme))
    } else {
        let hints = match &app.nav {
            DrillNavState::Browse { .. } => {
                let approved = app.engine.get_approved_decisions_count();
                let total = app.engine.get_all_decisions().len();
                format!(
                    "commit {}   j/k navigate    Enter drill in    a approve ({}/{})    q quit",
                    app.commit, approved, total,
                )
            }
            DrillNavState::Drill {
                node_idx,
                sibling_idx,
                views,
            } => {
                let files = files_for_decision(&app.engine, *node_idx);
                let total = files.len();
                let file_path = &files[*sibling_idx];
                let chunk_ids = chunks_for_file(&app.engine, *node_idx, file_path);
                let view = &views[*sibling_idx];
                let total_chunks = chunk_ids.len();
                let approved_count = chunk_ids
                    .iter()
                    .filter(|id| app.engine.state().is_approved(id))
                    .count();
                let ctx_label = if view.expanded.contains(&view.cursor) {
                    "collapse ctx"
                } else {
                    "expand ctx"
                };
                // Only advertise h/l when there is more than one file to cycle.
                let files_hint = if total > 1 {
                    format!("file {}/{}    h/l files    ", sibling_idx + 1, total)
                } else {
                    String::new()
                };
                // Only advertise the note toggle when the focused chunk has one.
                let note_hint = chunk_ids
                    .get(view.cursor)
                    .and_then(|id| note_for(&app.engine, id))
                    .map(|_| {
                        if view.expanded_notes.contains(&view.cursor) {
                            "i collapse note    "
                        } else {
                            "i expand note    "
                        }
                    })
                    .unwrap_or("");
                format!(
                    "commit {}   {}j/k chunks    Tab {}    {}a approve ({}/{})    Esc back    q quit",
                    app.commit, files_hint, ctx_label, note_hint, approved_count, total_chunks,
                )
            }
        };
        (hints, stylesheet::status_bar(theme))
    };
    frame.render_widget(Paragraph::new(status).style(status_style), status_area);
}

fn render_browse(frame: &mut Frame, area: Rect, app: &App, cursor: usize) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut heights: Vec<u16> = Vec::new();

    lines.push(separator_line(cr.width, theme.surface.mantle()));

    let decisions = app.engine.get_all_decisions();
    for (i, decision) in decisions.iter().enumerate() {
        let card_start = lines.len();
        let focused = i == cursor;
        let is_decision_approved = app.engine.is_decision_approved(decision.number);
        let card = make_card(cr.width, focused, theme.accents.lavender);
        let n_files = decision.code_impacts.len();
        let (chunks_approved, chunks_total) =
            app.engine.decision_approval_progress(decision.number);

        // label row — CardTier::Header (surface1)
        let label_card = if is_decision_approved {
            card.with_badge(Icons::APPROVED, theme.accents.green)
        } else {
            card
        };
        lines.push(label_card.at(
            CardTier::Header,
            vec![
                Span::styled(
                    format!("#{} {}", decision.number, decision.title),
                    Style::default().fg(theme.surface.text()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  {} file{} · {}/{} chunks approved",
                        n_files,
                        plural_s(n_files),
                        chunks_approved,
                        chunks_total,
                    ),
                    Style::default().fg(theme.surface.overlay0()),
                ),
            ],
            theme,
        ));

        // summary rows — CardTier::Header (pinnable block, same elevation as label)
        if let Some(rationale) = &decision.rationale {
            for text_line in wrap_text(rationale, text_width) {
                lines.push(card.at(
                    CardTier::Header,
                    vec![Span::styled(
                        format!("· {}", text_line),
                        Style::default().fg(theme.surface.subtext1()),
                    )],
                    theme,
                ));
            }
        }

        // children preview — CardTier::Body (surface0, lower elevation)
        for impact in &decision.code_impacts {
            lines.push(card.at(
                CardTier::Body,
                vec![Span::styled(
                    format!("{} {}", Icons::FILE_MODIFIED, impact.file),
                    Style::default().fg(theme.surface.text()),
                )],
                theme,
            ));
        }

        lines.push(separator_line(cr.width, theme.surface.mantle()));
        heights.push((lines.len() - card_start) as u16);
    }

    // The leading separator belongs to the first card's height so offsets align.
    if let Some(h) = heights.first_mut() {
        *h += 1;
    }
    let scroll = scroll_into_view(&heights, cursor, cr.height);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), cr);
}

fn render_drill(frame: &mut Frame, area: Rect, app: &App, s: DrillContext<'_>) {
    let DrillContext {
        node_idx,
        sibling_idx,
        view,
    } = s;
    let cursor = view.cursor;
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);

    let decisions = app.engine.get_all_decisions();
    let decision = decisions[node_idx];
    let files = files_for_decision(&app.engine, node_idx);
    let file_path = &files[sibling_idx];
    let chunk_ids = chunks_for_file(&app.engine, node_idx, file_path);

    // Find the code impact reasoning for this file
    let impact = decision
        .code_impacts
        .iter()
        .find(|ci| ci.file == *file_path)
        .expect("file path from engine should match decision log");

    // Build the anchored header: label + summary at CardTier::Header (surface1).
    let is_decision_approved = app.engine.is_decision_approved(decision.number);
    let header_card = HierarchicalCard::new(cr.width);
    let mut header_lines: Vec<Line<'static>> = Vec::new();
    let file_row_card = if is_decision_approved {
        header_card.with_badge(Icons::APPROVED, theme.accents.green)
    } else {
        header_card
    };
    header_lines.push(file_row_card.at(
        CardTier::Header,
        vec![Span::styled(
            format!("{} {}", Icons::FILE_MODIFIED, file_path),
            Style::default().fg(theme.surface.text()).add_modifier(Modifier::BOLD),
        )],
        theme,
    ));
    for text_line in wrap_text(&impact.reasoning, text_width) {
        header_lines.push(header_card.at(
            CardTier::Header,
            vec![Span::styled(
                text_line,
                Style::default().fg(theme.surface.subtext1()),
            )],
            theme,
        ));
    }
    let chunks_area = render_drill_header(frame, cr, header_lines, theme);
    let below_header_area = chunks_area;

    // Dot pagination — one mantle-level line between the strip and chunks.
    let total_siblings = files.len();
    let content_area = if total_siblings > 1 {
        let [dots_area, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(below_header_area);
        frame.render_widget(
            Paragraph::new(dot_pagination_line(
                cr.width,
                sibling_idx,
                total_siblings,
                theme,
            )),
            dots_area,
        );
        rest
    } else {
        below_header_area
    };

    let n = chunk_ids.len();
    let mut chunk_lines: Vec<Line<'static>> = Vec::new();

    for (i, chunk_id) in chunk_ids.iter().enumerate() {
        let is_approved = app.engine.state().is_approved(chunk_id);
        let card = make_card(cr.width, i == cursor, theme.accents.lavender);
        let chunk_expanded = view.expanded.contains(&i);
        let annot_expanded = view.expanded_notes.contains(&i);

        let note = note_for(&app.engine, chunk_id);

        if let Some(instr) = note {
            let wrap_width = text_width.saturating_sub(6);
            let mut rows = note_rows(instr, wrap_width);
            let has_more = rows.len() > 1;
            if !annot_expanded {
                let first = rows.into_iter().next().unwrap_or_default();
                rows = vec![if has_more {
                    format!("{}…", first.trim_end_matches(' '))
                } else {
                    first
                }];
            }
            for (row, text_line) in rows.into_iter().enumerate() {
                let icon_col = if row == 0 {
                    Span::styled(
                        Icons::HAS_INSTRUCTIONS,
                        Style::default().fg(theme.accents.yellow),
                    )
                } else {
                    Span::styled("  ", Style::default())
                };
                let row_card = if row == 0 && is_approved {
                    card.with_badge(Icons::APPROVED, theme.accents.green)
                } else {
                    card
                };
                chunk_lines.push(row_card.at(
                    CardTier::Body,
                    vec![
                        Span::styled(" ", Style::default()),
                        icon_col,
                        Span::styled(
                            format!("   {}", text_line),
                            Style::default().fg(theme.surface.subtext1()),
                        ),
                    ],
                    theme,
                ));
            }
        }

        if let Some(renderable) = app.engine.get_renderable_diff_object(chunk_id) {
            let has_note = note.is_some();
            let visible_lines: Vec<_> = renderable
                .lines
                .iter()
                .filter(|line| chunk_expanded || line_has_change(line))
                .collect();

            for (line_idx, line) in visible_lines.iter().enumerate() {
                let ct = line_change_type(line);
                let (fg, sigil) = match &ct {
                    Some(ChangeType::Added) => (theme.accents.green, "+"),
                    Some(ChangeType::Deleted) => (theme.accents.red, "-"),
                    _ => (theme.surface.subtext0(), " "),
                };
                let row_card = if !has_note && line_idx == 0 && is_approved {
                    card.with_badge(Icons::APPROVED, theme.accents.green)
                } else {
                    card
                };
                chunk_lines.push(row_card.at(
                    CardTier::Content,
                    vec![
                        Span::styled(
                            format!("{:>3} ", line.line_number),
                            Style::default().fg(theme.surface.overlay0()),
                        ),
                        Span::styled(sigil, Style::default().fg(fg).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" {}", line.content), Style::default().fg(fg)),
                    ],
                    theme,
                ));
            }
        }

        if i + 1 < n {
            chunk_lines.push(Line::styled(
                format!("{:^width$}", "···", width = cr.width as usize),
                Style::default()
                    .fg(theme.surface.overlay0())
                    .bg(theme.surface.mantle()),
            ));
        }
    }

    let heights: Vec<u16> = chunk_ids
        .iter()
        .enumerate()
        .map(|(i, chunk_id)| {
            let h = visible_line_count(
                &app.engine,
                chunk_id,
                note_for(&app.engine, chunk_id),
                view.expanded.contains(&i),
                view.expanded_notes.contains(&i),
                text_width,
            );
            if i + 1 < n { h + 1 } else { h }
        })
        .collect();
    let scroll = scroll_into_view(&heights, cursor, content_area.height);
    frame.render_widget(
        Paragraph::new(chunk_lines).scroll((scroll, 0)),
        content_area,
    );
}

fn line_change_type(line: &RenderableLine<'_>) -> Option<ChangeType> {
    line.annotations.first().and_then(|a| a.change_type.clone())
}

fn line_has_change(line: &RenderableLine<'_>) -> bool {
    line.annotations
        .first()
        .and_then(|a| a.change_type.as_ref())
        .is_some()
}

fn visible_line_count(
    engine: &ReviewEngine,
    chunk_id: &ReviewableDiffId,
    note: Option<&diffviz_review::Instruction>,
    expanded: bool,
    annot_expanded: bool,
    text_width: usize,
) -> u16 {
    let code_lines = if let Some(renderable) = engine.get_renderable_diff_object(chunk_id) {
        if expanded {
            renderable.lines.len() as u16
        } else {
            renderable
                .lines
                .iter()
                .filter(|l| line_has_change(l))
                .count() as u16
        }
    } else {
        0
    };
    let instr_lines = if let Some(instr) = note {
        if annot_expanded {
            note_rows(instr, text_width.saturating_sub(6)).len() as u16
        } else {
            1
        }
    } else {
        0
    };
    code_lines + instr_lines
}

/// Wrapped display rows for a note: authors prefix the first contribution;
/// each appended contribution starts on its own row (note content is
/// newline-separated under the single-note model).
fn note_rows(instr: &diffviz_review::Instruction, wrap_width: usize) -> Vec<String> {
    let text = format!("{}: {}", instr.author, instr.content);
    text.split('\n')
        .flat_map(|segment| wrap_text(segment, wrap_width))
        .collect()
}

fn make_card(col_width: u16, focused: bool, accent_color: Color) -> HierarchicalCard {
    if focused {
        HierarchicalCard::new(col_width).focused(accent_color)
    } else {
        HierarchicalCard::new(col_width)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new()?;
    let mut dirty = true;

    loop {
        // Redraw only when state changed — no busy re-rendering between keys.
        if dirty {
            terminal.draw(|f| render(f, &app))?;
            dirty = false;
        }

        if event::poll(std::time::Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if let Some(event) = handle_key_event(key.code) {
                        app.handle_nav_event(event);
                        dirty = true;
                    }
                }
                Event::Resize(_, _) => dirty = true,
                _ => {}
            }
        }

        if app.quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
