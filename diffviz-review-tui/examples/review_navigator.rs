//! Review Navigator — three-level hierarchical navigation prototype.
//!
//! DrillNav pattern:
//!   Browse  j/k navigate nodes  | Enter drill in | q quit
//!   Drill   j/k navigate chunks | h/l cycle siblings (wraps) | Tab expand context | Esc back | q quit
//!
//! Surface ramp (dark theme, lighter = higher elevation):
//!   rationale    → surface1   (CardTier::Header — pinnable)
//!   instructions → surface0   (CardTier::Body)
//!   code lines   → base       (CardTier::Content)
//!   pinned header container → mantle
//!   separator    → mantle     (widget floor — never touches crust/terminal)
//!
//! Layout: content capped at 120 columns, centered; surface bg fills full column width.

use std::collections::{HashMap, HashSet};
use std::io;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use diffviz_core::{
    LineRange as CoreLineRange, ProgrammingLanguage, RenderableDiff, RenderableLine,
    RenderableMetadata, SemanticNodeKind,
    ast_diff::ESSENTIAL,
    renderable_diff::{ChangeType, LineAnnotation},
};
use diffviz_review::entities::{
    decision::{CodeImpact, Decision, DecisionLineRange, DecisionLog},
    instruction::{Instruction, InstructionStatus},
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use tui_design::{
    CardTier, HierarchicalCard, Icons, Theme, render_drill_header, scroll_into_view,
    separator_line, stylesheet,
};

use diffviz_review_tui::events::input::UiEvent;

// ── Mock display data (separate from decision metadata) ───────────────────────
//
// The real app sources this from RenderableDiff produced by the diff engine.
// These local types exist only to hold mock content until the example is promoted.

struct MockImpactDisplay {
    chunks: Vec<MockChunk>,
}

struct MockChunk {
    diff: RenderableDiff<'static>,
    instruction: Option<Instruction>,
}

// ── Mock data builder ─────────────────────────────────────────────────────────

/// Constructs a `RenderableDiff<'static>` from static line data.
/// Each row is `(content, change_type)` where `None` = context line.
fn mock_diff(
    start_line: usize,
    rows: &[(&'static str, Option<ChangeType>)],
) -> RenderableDiff<'static> {
    let lines: Vec<RenderableLine<'static>> = rows
        .iter()
        .enumerate()
        .map(|(i, (content, ct))| RenderableLine {
            line_number: start_line + i,
            content,
            byte_range: (0, content.len()),
            annotations: vec![LineAnnotation {
                start_col: 0,
                end_col: content.len(),
                relevance: ESSENTIAL,
                change_type: ct.clone(),
                semantic_kind: SemanticNodeKind::Function,
                node_depth: 0,
            }],
            semantic_anchor: None,
        })
        .collect();

    let changed_line_numbers = rows
        .iter()
        .enumerate()
        .filter(|(_, (_, ct))| ct.is_some())
        .map(|(i, _)| start_line + i)
        .collect();

    RenderableDiff {
        metadata: RenderableMetadata {
            total_changes: rows.iter().filter(|(_, ct)| ct.is_some()).count(),
            change_summary: HashMap::new(),
            essential_line_count: rows.len(),
            boundary_name: String::new(),
            overall_line_range: CoreLineRange {
                start_line,
                end_line: start_line + rows.len(),
                start_column: 0,
                end_column: 0,
            },
            changed_line_numbers,
        },
        lines,
        language: ProgrammingLanguage::Rust,
    }
}

fn instr(id: &str, author: &str, content: &str) -> Instruction {
    Instruction {
        id: id.to_string(),
        author: author.to_string(),
        timestamp: String::new(),
        content: content.to_string(),
        status: InstructionStatus::Active,
    }
}

fn mock_data() -> (DecisionLog, Vec<Vec<MockImpactDisplay>>) {
    use ChangeType::{Added, Deleted};

    let log = DecisionLog {
        commit: "a3f9c12".to_string(),
        decisions: vec![
            Decision {
                number: 1,
                title: "Refactor authentication middleware".to_string(),
                rationale: Some(
                    "Middleware was tightly coupled to the session store, \
                    making it impossible to swap implementations without touching call sites."
                        .to_string(),
                ),
                code_impacts: vec![
                    CodeImpact {
                        file: "src/auth/middleware.rs".to_string(),
                        reasoning: "Extracting the validation logic into a trait allows \
                            injecting different backends. The TokenValidator trait replaces \
                            the concrete SessionStore parameter on validate_token."
                            .to_string(),
                        line_ranges: vec![
                            DecisionLineRange { start: 8, end: 26 },
                            DecisionLineRange { start: 38, end: 57 },
                        ],
                    },
                    CodeImpact {
                        file: "src/auth/token.rs".to_string(),
                        reasoning: "Token type enum was missing the ApiKey variant, causing \
                            panics when API clients authenticated. Added the variant and wired \
                            it through the validator."
                            .to_string(),
                        line_ranges: vec![DecisionLineRange { start: 4, end: 22 }],
                    },
                ],
            },
            Decision {
                number: 2,
                title: "Introduce rate limiting on public endpoints".to_string(),
                rationale: Some(
                    "No rate limiting existed on /api/public/* routes, \
                    exposing the service to trivial abuse before the next release."
                        .to_string(),
                ),
                code_impacts: vec![CodeImpact {
                    file: "src/rate_limiter.rs".to_string(),
                    reasoning: "A sliding window limiter is the minimal viable fix. \
                            Redis TTL drives window expiry; the window size is configurable \
                            per environment via the SLA document."
                        .to_string(),
                    line_ranges: vec![
                        DecisionLineRange { start: 1, end: 16 },
                        DecisionLineRange { start: 18, end: 35 },
                    ],
                }],
            },
        ],
    };

    // Display data — line content that the real app will source from the diff engine.
    let display: Vec<Vec<MockImpactDisplay>> = vec![
        // Decision 1
        vec![
            // Impact 0: src/auth/middleware.rs
            MockImpactDisplay {
                chunks: vec![
                    MockChunk {
                        diff: mock_diff(
                            8,
                            &[
                                ("use anyhow::Result;", None),
                                ("use crate::claims::Claims;", None),
                                ("use crate::jwt::decode_jwt;", None),
                                ("use crate::session::SessionStore;", Some(Deleted)),
                                ("use crate::session::TokenValidator;", Some(Added)),
                                ("", None),
                                (
                                    "pub fn validate_token(token: &str, store: &SessionStore) -> Result<Claims> {",
                                    Some(Deleted),
                                ),
                                (
                                    "pub fn validate_token<V: TokenValidator>(token: &str, v: &V) -> Result<Claims> {",
                                    Some(Added),
                                ),
                                ("    let raw = decode_jwt(token)?;", None),
                                ("    let header = raw.header();", None),
                                ("    store.validate(&raw.claims)?;", Some(Deleted)),
                                ("    v.validate(&raw.claims)?;", Some(Added)),
                                ("    Ok(raw.into_claims())", None),
                                ("}", None),
                            ],
                        ),
                        instruction: Some(instr(
                            "i1",
                            "alice",
                            "Make sure TokenValidator is object-safe before merging — \
                            we'll need Arc<dyn TokenValidator> in the Middleware struct below.",
                        )),
                    },
                    MockChunk {
                        diff: mock_diff(
                            38,
                            &[
                                ("pub struct Middleware {", None),
                                ("    store: SessionStore,", Some(Deleted)),
                                ("    validator: Arc<dyn TokenValidator>,", Some(Added)),
                                ("    timeout: Duration,", None),
                                ("}", None),
                                ("", None),
                                ("impl Default for Middleware {", None),
                                ("    fn default() -> Self {", Some(Deleted)),
                                (
                                    "        Self { store: SessionStore::new(), timeout: Duration::from_secs(30) }",
                                    Some(Deleted),
                                ),
                                ("    }", Some(Deleted)),
                                ("    fn default() -> Self {", Some(Added)),
                                ("        Self {", Some(Added)),
                                (
                                    "            validator: Arc::new(DefaultValidator::new()),",
                                    Some(Added),
                                ),
                                ("            timeout: Duration::from_secs(30),", Some(Added)),
                                ("        }", Some(Added)),
                                ("    }", Some(Added)),
                                ("}", None),
                            ],
                        ),
                        instruction: Some(instr(
                            "i2",
                            "bob",
                            "DefaultValidator::new() is not yet implemented — \
                            this will panic at runtime if the default impl is exercised.",
                        )),
                    },
                ],
            },
            // Impact 1: src/auth/token.rs
            MockImpactDisplay {
                chunks: vec![MockChunk {
                    diff: mock_diff(
                        4,
                        &[
                            ("/// Identifies the authentication mechanism used.", None),
                            (
                                "/// Variants must stay in sync with the claims `typ` field.",
                                None,
                            ),
                            ("#[derive(Debug, Clone, PartialEq, Eq)]", None),
                            ("pub enum TokenType {", None),
                            (
                                "    /// Browser session cookie — 30-minute sliding expiry.",
                                None,
                            ),
                            ("    Session,", None),
                            (
                                "    /// Long-lived API key — expiry driven by key record in DB.",
                                Some(Added),
                            ),
                            ("    ApiKey,", Some(Added)),
                            ("}", None),
                            ("", None),
                            ("impl TokenType {", None),
                            ("    pub fn is_session(&self) -> bool {", None),
                            ("        matches!(self, Self::Session)", None),
                            ("    }", None),
                            ("    pub fn lifetime(&self) -> Duration {", Some(Deleted)),
                            ("        Duration::from_secs(1800)", Some(Deleted)),
                            ("    pub fn lifetime(&self) -> Duration {", Some(Added)),
                            ("        match self {", Some(Added)),
                            (
                                "            Self::Session => Duration::from_secs(1800),",
                                Some(Added),
                            ),
                            (
                                "            Self::ApiKey  => Duration::from_secs(86400 * 365),",
                                Some(Added),
                            ),
                            ("        }", Some(Added)),
                            ("    }", None),
                            ("}", None),
                        ],
                    ),
                    instruction: None,
                }],
            },
        ],
        // Decision 2
        vec![
            // Impact 0: src/rate_limiter.rs
            MockImpactDisplay {
                chunks: vec![
                    MockChunk {
                        diff: mock_diff(
                            1,
                            &[
                                ("use std::collections::VecDeque;", Some(Added)),
                                ("use std::time::{Duration, Instant};", Some(Added)),
                                ("", Some(Added)),
                                (
                                    "/// Sliding-window rate limiter backed by an in-process ring buffer.",
                                    Some(Added),
                                ),
                                (
                                    "/// For multi-instance deployments, swap the inner store for Redis.",
                                    Some(Added),
                                ),
                                ("pub struct SlidingWindowLimiter {", Some(Added)),
                                ("    window: Duration,", Some(Added)),
                                ("    max_requests: u32,", Some(Added)),
                                ("    timestamps: VecDeque<Instant>,", Some(Added)),
                                ("}", Some(Added)),
                            ],
                        ),
                        instruction: None,
                    },
                    MockChunk {
                        diff: mock_diff(
                            18,
                            &[
                                ("impl SlidingWindowLimiter {", Some(Added)),
                                (
                                    "    pub fn new(window: Duration, max_requests: u32) -> Self {",
                                    Some(Added),
                                ),
                                (
                                    "        Self { window, max_requests, timestamps: VecDeque::new() }",
                                    Some(Added),
                                ),
                                ("    }", Some(Added)),
                                ("", Some(Added)),
                                (
                                    "    /// Returns `true` if the request is allowed, `false` if rate-limited.",
                                    Some(Added),
                                ),
                                (
                                    "    pub fn check_and_record(&mut self) -> bool {",
                                    Some(Added),
                                ),
                                ("        let now = Instant::now();", Some(Added)),
                                ("        let cutoff = now - self.window;", Some(Added)),
                                (
                                    "        self.timestamps.retain(|&t| t > cutoff);",
                                    Some(Added),
                                ),
                                (
                                    "        if self.timestamps.len() as u32 >= self.max_requests {",
                                    Some(Added),
                                ),
                                ("            return false;", Some(Added)),
                                ("        }", Some(Added)),
                                ("        self.timestamps.push_back(now);", Some(Added)),
                                ("        true", Some(Added)),
                                ("    }", Some(Added)),
                                ("}", Some(Added)),
                            ],
                        ),
                        instruction: Some(instr(
                            "i3",
                            "carol",
                            "Consider extracting retain + len check into a named helper — \
                            check_and_record is doing two distinct things.",
                        )),
                    },
                ],
            },
        ],
    ];

    (log, display)
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
    cursor: usize,
    expanded: &'a HashSet<usize>,
    expanded_annotations: &'a HashSet<usize>,
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
        /// j/k cursor within the visible children list.
        cursor: usize,
        /// Children with expanded context (Tab toggles).
        expanded: HashSet<usize>,
        /// Children with expanded annotation text (Tab toggles).
        expanded_annotations: HashSet<usize>,
    },
}

struct App {
    log: DecisionLog,
    display: Vec<Vec<MockImpactDisplay>>,
    nav: DrillNavState,
    /// Approved chunks: (node_idx, sibling_idx, chunk_idx).
    approved: HashSet<(usize, usize, usize)>,
    /// Approved decisions: node_idx. Mirrors ReviewEngine's DecisionApprovals —
    /// decision approval is tracked independently and cascades to chunks on toggle.
    approved_decisions: HashSet<usize>,
    theme: Theme,
    quit: bool,
}

impl App {
    fn new() -> Self {
        let (log, display) = mock_data();
        App {
            log,
            display,
            nav: DrillNavState::Browse { cursor: 0 },
            approved: HashSet::new(),
            approved_decisions: HashSet::new(),
            theme: Theme::mocha(),
            quit: false,
        }
    }

    fn handle_nav_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::Quit => self.quit = true,
            UiEvent::NavigateUp => self.navigate_up(),
            UiEvent::NavigateDown => self.navigate_down(),
            UiEvent::NavigateLeft => self.navigate_left(),
            UiEvent::NavigateRight => self.navigate_right(),
            UiEvent::ToggleApprove => self.toggle_approve(),
            UiEvent::SelectCurrent => self.drill_in(),
            UiEvent::Back => self.back(),
            UiEvent::ToggleDecisionExpansion => self.toggle_expansion(),
            _ => {}
        }
    }

    fn toggle_approve(&mut self) {
        match &self.nav {
            DrillNavState::Browse { cursor } => {
                let node_idx = *cursor;
                if self.approved_decisions.remove(&node_idx) {
                    // Unapprove decision → cascade unapprove all its chunks.
                    for sibling_idx in 0..self.display[node_idx].len() {
                        for chunk_idx in 0..self.display[node_idx][sibling_idx].chunks.len() {
                            self.approved.remove(&(node_idx, sibling_idx, chunk_idx));
                        }
                    }
                } else {
                    // Approve decision → cascade approve all its chunks.
                    self.approved_decisions.insert(node_idx);
                    for sibling_idx in 0..self.display[node_idx].len() {
                        for chunk_idx in 0..self.display[node_idx][sibling_idx].chunks.len() {
                            self.approved.insert((node_idx, sibling_idx, chunk_idx));
                        }
                    }
                }
            }
            DrillNavState::Drill {
                node_idx,
                sibling_idx,
                cursor,
                ..
            } => {
                let key = (*node_idx, *sibling_idx, *cursor);
                if !self.approved.remove(&key) {
                    self.approved.insert(key);
                }
            }
        }
    }

    fn navigate_up(&mut self) {
        match &mut self.nav {
            DrillNavState::Browse { cursor } => {
                if *cursor > 0 {
                    *cursor -= 1;
                }
            }
            DrillNavState::Drill { cursor, .. } => {
                if *cursor > 0 {
                    *cursor -= 1;
                }
            }
        }
    }

    fn navigate_down(&mut self) {
        match &mut self.nav {
            DrillNavState::Browse { cursor } => {
                if *cursor + 1 < self.log.decisions.len() {
                    *cursor += 1;
                }
            }
            DrillNavState::Drill {
                node_idx,
                sibling_idx,
                cursor,
                ..
            } => {
                let n = self.display[*node_idx][*sibling_idx].chunks.len();
                if *cursor + 1 < n {
                    *cursor += 1;
                }
            }
        }
    }

    fn navigate_left(&mut self) {
        if let DrillNavState::Drill {
            node_idx,
            sibling_idx,
            cursor,
            expanded,
            expanded_annotations,
        } = &mut self.nav
        {
            let n = self.log.decisions[*node_idx].code_impacts.len();
            *sibling_idx = sibling_idx.checked_sub(1).unwrap_or(n - 1);
            *cursor = 0;
            expanded.clear();
            expanded_annotations.clear();
        }
    }

    fn navigate_right(&mut self) {
        if let DrillNavState::Drill {
            node_idx,
            sibling_idx,
            cursor,
            expanded,
            expanded_annotations,
        } = &mut self.nav
        {
            let n = self.log.decisions[*node_idx].code_impacts.len();
            *sibling_idx = (*sibling_idx + 1) % n;
            *cursor = 0;
            expanded.clear();
            expanded_annotations.clear();
        }
    }

    fn drill_in(&mut self) {
        if let DrillNavState::Browse { cursor } = &self.nav {
            let idx = *cursor;
            self.nav = DrillNavState::Drill {
                node_idx: idx,
                sibling_idx: 0,
                cursor: 0,
                expanded: HashSet::new(),
                expanded_annotations: HashSet::from([0]),
            };
        }
    }

    fn back(&mut self) {
        if let DrillNavState::Drill { node_idx, .. } = &self.nav {
            let d = *node_idx;
            self.nav = DrillNavState::Browse { cursor: d };
        }
    }

    fn toggle_expansion(&mut self) {
        if let DrillNavState::Drill {
            cursor, expanded, ..
        } = &mut self.nav
        {
            let chunk = *cursor;
            if !expanded.remove(&chunk) {
                expanded.insert(chunk);
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

fn wrap_text(text: &str, max_cols: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_cols {
            current.push(' ');
            current.push_str(word);
        } else {
            result.push(std::mem::take(&mut current));
            current = word.to_string();
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
        Paragraph::new("").style(Style::default().bg(theme.surface.crust())),
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
            cursor,
            expanded,
            expanded_annotations,
        } => render_drill(
            frame,
            content_area,
            app,
            DrillContext {
                node_idx: *node_idx,
                sibling_idx: *sibling_idx,
                cursor: *cursor,
                expanded,
                expanded_annotations,
            },
            &app.approved,
        ),
    }

    let status = match &app.nav {
        DrillNavState::Browse { .. } => {
            let approved = app.approved_decisions.len();
            let total = app.log.decisions.len();
            format!(
                "commit {}   j/k navigate    Enter drill in    a approve ({}/{})    q quit",
                app.log.commit, approved, total,
            )
        }
        DrillNavState::Drill {
            node_idx,
            sibling_idx,
            cursor,
            expanded,
            ..
        } => {
            let total = app.log.decisions[*node_idx].code_impacts.len();
            let ctx_label = if expanded.contains(cursor) {
                "collapse ctx"
            } else {
                "expand ctx"
            };
            let approved_count = app.display[*node_idx][*sibling_idx]
                .chunks
                .iter()
                .enumerate()
                .filter(|(i, _)| app.approved.contains(&(*node_idx, *sibling_idx, *i)))
                .count();
            let total_chunks = app.display[*node_idx][*sibling_idx].chunks.len();
            format!(
                "commit {}   file {}/{}    h/l files    j/k chunks    Tab {}    a approve ({}/{})    Esc back    q quit",
                app.log.commit,
                sibling_idx + 1,
                total,
                ctx_label,
                approved_count,
                total_chunks,
            )
        }
    };
    frame.render_widget(
        Paragraph::new(status).style(stylesheet::status_bar(theme)),
        status_area,
    );
}

fn render_browse(frame: &mut Frame, area: Rect, app: &App, cursor: usize) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - 4;
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(separator_line(cr.width, theme.surface.mantle()));

    for (i, decision) in app.log.decisions.iter().enumerate() {
        let focused = i == cursor;
        let is_decision_approved = app.approved_decisions.contains(&i);
            let card = make_card(cr.width, focused, theme.accents.lavender);
            let n_files = decision.code_impacts.len();

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
                    format!("  {} file{}", n_files, plural_s(n_files)),
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
    }

    frame.render_widget(Paragraph::new(lines), cr);
}

fn render_drill(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    s: DrillContext<'_>,
    approved: &HashSet<(usize, usize, usize)>,
) {
    let DrillContext {
        node_idx,
        sibling_idx,
        cursor,
        expanded,
        expanded_annotations,
    } = s;
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - 4;
    let impact = &app.log.decisions[node_idx].code_impacts[sibling_idx];
    let display = &app.display[node_idx][sibling_idx];

    // Build the anchored header: label + summary at CardTier::Header (surface1).
    // render_drill_header handles the mantle fill, separators, and layout split.
    let is_decision_approved = app.approved_decisions.contains(&node_idx);
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
            format!("{} {}", Icons::FILE_MODIFIED, impact.file),
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
    let total_siblings = app.log.decisions[node_idx].code_impacts.len();
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

    let mut chunk_lines: Vec<Line<'static>> = Vec::new();
    for (i, chunk) in display.chunks.iter().enumerate() {
        let is_approved = approved.contains(&(node_idx, sibling_idx, i));
        let card = make_card(cr.width, i == cursor, theme.accents.lavender);
        let chunk_expanded = expanded.contains(&i);
        let annot_expanded = expanded_annotations.contains(&i);

        if let Some(instr) = &chunk.instruction {
            let instr_text = format!("{}: {}", instr.author, instr.content);
            let wrap_width = text_width.saturating_sub(6);
            let mut rows = wrap_text(&instr_text, wrap_width);
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

        let visible_lines: Vec<_> = chunk
            .diff
            .lines
            .iter()
            .filter(|line| chunk_expanded || line_has_change(line))
            .collect();

        let has_instr = chunk.instruction.is_some();
        for (line_idx, line) in visible_lines.iter().enumerate() {
            let ct = line_change_type(line);
            let (fg, sigil) = match &ct {
                Some(ChangeType::Added) => (theme.accents.green, "+"),
                Some(ChangeType::Deleted) => (theme.accents.red, "-"),
                _ => (theme.surface.subtext0(), " "),
            };
            let row_card = if !has_instr && line_idx == 0 && is_approved {
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

        if i + 1 < display.chunks.len() {
            chunk_lines.push(Line::styled(
                format!("{:^width$}", "···", width = cr.width as usize),
                Style::default()
                    .fg(theme.surface.overlay0())
                    .bg(theme.surface.mantle()),
            ));
        }
    }

    let n = display.chunks.len();
    let heights: Vec<u16> = (0..n)
        .map(|i| {
            let h = visible_line_count(
                &display.chunks[i],
                expanded.contains(&i),
                expanded_annotations.contains(&i),
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
    chunk: &MockChunk,
    expanded: bool,
    annot_expanded: bool,
    text_width: usize,
) -> u16 {
    let code_lines = if expanded {
        chunk.diff.lines.len() as u16
    } else {
        chunk
            .diff
            .lines
            .iter()
            .filter(|l| line_has_change(l))
            .count() as u16
    };
    let instr_lines = if let Some(instr) = &chunk.instruction {
        if annot_expanded {
            let text = format!("{}: {}", instr.author, instr.content);
            wrap_text(&text, text_width.saturating_sub(6)).len() as u16
        } else {
            1
        }
    } else {
        0
    };
    code_lines + instr_lines
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

    let mut app = App::new();

    loop {
        terminal.draw(|f| render(f, &app))?;

        if event::poll(std::time::Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && let Some(event) = handle_key_event(key.code)
        {
            app.handle_nav_event(event);
        }

        if app.quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
