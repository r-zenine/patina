//! Review Navigator — three-level hierarchical navigation prototype.
//!
//! L1  j/k navigate decisions  | Enter drill in | q quit
//! L2  j/k navigate chunks     | h/l cycle files (wraps) | Tab expand context | Esc back | q quit
//!
//! Surface ramp (dark theme, lighter = higher elevation):
//!   rationale    → surface1   (highest widget elevation)
//!   instructions → surface0
//!   code lines   → base
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
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use tui_design::{HierarchicalCard, Icons, Theme, separator_line, stylesheet};

// ── Mock data (mirrors the decision-log YAML template) ────────────────────────

struct DecisionLog {
    commit: &'static str,
    decisions: Vec<Decision>,
}

struct Decision {
    number: u32,
    title: &'static str,
    rationale: Option<&'static str>,
    code_impacts: Vec<CodeImpact>,
}

struct CodeImpact {
    file: &'static str,
    reasoning: &'static str,
    line_ranges: Vec<LineRange>,
}

struct LineRange {
    start: u32,
    #[allow(dead_code)] // present to match the YAML template schema; used by real deserializer
    end: u32,
    lines: Vec<(LineKind, &'static str)>,
}

#[derive(Clone, Copy)]
enum LineKind {
    Added,
    Removed,
    Context,
}

fn mock_data() -> DecisionLog {
    DecisionLog {
        commit: "a3f9c12",
        decisions: vec![
            Decision {
                number: 1,
                title: "Refactor authentication middleware",
                rationale: Some(
                    "Middleware was tightly coupled to the session store, \
                    making it impossible to swap implementations without touching call sites.",
                ),
                code_impacts: vec![
                    CodeImpact {
                        file: "src/auth/middleware.rs",
                        reasoning: "Extracting the validation logic into a trait allows \
                            injecting different backends. The TokenValidator trait replaces \
                            the concrete SessionStore parameter on validate_token.",
                        line_ranges: vec![
                            LineRange {
                                start: 8,
                                end: 26,
                                lines: vec![
                                    (LineKind::Context, "use anyhow::Result;"),
                                    (LineKind::Context, "use crate::claims::Claims;"),
                                    (LineKind::Context, "use crate::jwt::decode_jwt;"),
                                    (LineKind::Removed, "use crate::session::SessionStore;"),
                                    (LineKind::Added,   "use crate::session::TokenValidator;"),
                                    (LineKind::Context, ""),
                                    (LineKind::Removed, "pub fn validate_token(token: &str, store: &SessionStore) -> Result<Claims> {"),
                                    (LineKind::Added,   "pub fn validate_token<V: TokenValidator>(token: &str, v: &V) -> Result<Claims> {"),
                                    (LineKind::Context, "    let raw = decode_jwt(token)?;"),
                                    (LineKind::Context, "    let header = raw.header();"),
                                    (LineKind::Removed, "    store.validate(&raw.claims)?;"),
                                    (LineKind::Added,   "    v.validate(&raw.claims)?;"),
                                    (LineKind::Context, "    Ok(raw.into_claims())"),
                                    (LineKind::Context, "}"),
                                ],
                            },
                            LineRange {
                                start: 38,
                                end: 57,
                                lines: vec![
                                    (LineKind::Context, "pub struct Middleware {"),
                                    (LineKind::Removed, "    store: SessionStore,"),
                                    (LineKind::Added,   "    validator: Arc<dyn TokenValidator>,"),
                                    (LineKind::Context, "    timeout: Duration,"),
                                    (LineKind::Context, "}"),
                                    (LineKind::Context, ""),
                                    (LineKind::Context, "impl Default for Middleware {"),
                                    (LineKind::Removed, "    fn default() -> Self {"),
                                    (LineKind::Removed, "        Self { store: SessionStore::new(), timeout: Duration::from_secs(30) }"),
                                    (LineKind::Removed, "    }"),
                                    (LineKind::Added,   "    fn default() -> Self {"),
                                    (LineKind::Added,   "        Self {"),
                                    (LineKind::Added,   "            validator: Arc::new(DefaultValidator::new()),"),
                                    (LineKind::Added,   "            timeout: Duration::from_secs(30),"),
                                    (LineKind::Added,   "        }"),
                                    (LineKind::Added,   "    }"),
                                    (LineKind::Context, "}"),
                                ],
                            },
                        ],
                    },
                    CodeImpact {
                        file: "src/auth/token.rs",
                        reasoning: "Token type enum was missing the ApiKey variant, causing \
                            panics when API clients authenticated. Added the variant and wired \
                            it through the validator.",
                        line_ranges: vec![LineRange {
                            start: 4,
                            end: 22,
                            lines: vec![
                                (LineKind::Context, "/// Identifies the authentication mechanism used."),
                                (LineKind::Context, "/// Variants must stay in sync with the claims `typ` field."),
                                (LineKind::Context, "#[derive(Debug, Clone, PartialEq, Eq)]"),
                                (LineKind::Context, "pub enum TokenType {"),
                                (LineKind::Context, "    /// Browser session cookie — 30-minute sliding expiry."),
                                (LineKind::Context, "    Session,"),
                                (LineKind::Added,   "    /// Long-lived API key — expiry driven by key record in DB."),
                                (LineKind::Added,   "    ApiKey,"),
                                (LineKind::Context, "}"),
                                (LineKind::Context, ""),
                                (LineKind::Context, "impl TokenType {"),
                                (LineKind::Context, "    pub fn is_session(&self) -> bool {"),
                                (LineKind::Context, "        matches!(self, Self::Session)"),
                                (LineKind::Context, "    }"),
                                (LineKind::Removed, "    pub fn lifetime(&self) -> Duration {"),
                                (LineKind::Removed, "        Duration::from_secs(1800)"),
                                (LineKind::Added,   "    pub fn lifetime(&self) -> Duration {"),
                                (LineKind::Added,   "        match self {"),
                                (LineKind::Added,   "            Self::Session => Duration::from_secs(1800),"),
                                (LineKind::Added,   "            Self::ApiKey  => Duration::from_secs(86400 * 365),"),
                                (LineKind::Added,   "        }"),
                                (LineKind::Context, "    }"),
                                (LineKind::Context, "}"),
                            ],
                        }],
                    },
                ],
            },
            Decision {
                number: 2,
                title: "Introduce rate limiting on public endpoints",
                rationale: Some(
                    "No rate limiting existed on /api/public/* routes, \
                    exposing the service to trivial abuse before the next release.",
                ),
                code_impacts: vec![CodeImpact {
                    file: "src/rate_limiter.rs",
                    reasoning: "A sliding window limiter is the minimal viable fix. \
                            Redis TTL drives window expiry; the window size is configurable \
                            per environment via the SLA document.",
                    line_ranges: vec![
                        LineRange {
                            start: 1,
                            end: 16,
                            lines: vec![
                                (LineKind::Added, "use std::collections::VecDeque;"),
                                (LineKind::Added, "use std::time::{Duration, Instant};"),
                                (LineKind::Added, ""),
                                (LineKind::Added, "/// Sliding-window rate limiter backed by an in-process ring buffer."),
                                (LineKind::Added, "/// For multi-instance deployments, swap the inner store for Redis."),
                                (LineKind::Added, "pub struct SlidingWindowLimiter {"),
                                (LineKind::Added, "    window: Duration,"),
                                (LineKind::Added, "    max_requests: u32,"),
                                (LineKind::Added, "    timestamps: VecDeque<Instant>,"),
                                (LineKind::Added, "}"),
                            ],
                        },
                        LineRange {
                            start: 18,
                            end: 35,
                            lines: vec![
                                (LineKind::Added, "impl SlidingWindowLimiter {"),
                                (LineKind::Added, "    pub fn new(window: Duration, max_requests: u32) -> Self {"),
                                (LineKind::Added, "        Self { window, max_requests, timestamps: VecDeque::new() }"),
                                (LineKind::Added, "    }"),
                                (LineKind::Added, ""),
                                (LineKind::Added, "    /// Returns `true` if the request is allowed, `false` if rate-limited."),
                                (LineKind::Added, "    pub fn check_and_record(&mut self) -> bool {"),
                                (LineKind::Added, "        let now = Instant::now();"),
                                (LineKind::Added, "        let cutoff = now - self.window;"),
                                (LineKind::Added, "        self.timestamps.retain(|&t| t > cutoff);"),
                                (LineKind::Added, "        if self.timestamps.len() as u32 >= self.max_requests {"),
                                (LineKind::Added, "            return false;"),
                                (LineKind::Added, "        }"),
                                (LineKind::Added, "        self.timestamps.push_back(now);"),
                                (LineKind::Added, "        true"),
                                (LineKind::Added, "    }"),
                                (LineKind::Added, "}"),
                            ],
                        },
                    ],
                }],
            },
        ],
    }
}

// ── Navigation state ──────────────────────────────────────────────────────────

enum NavLevel {
    L1 {
        selected: usize,
    },
    L2 {
        decision_idx: usize,
        impact_idx: usize,
        focused_chunk: usize,
        expanded_chunks: HashSet<usize>,
    },
}

struct App {
    log: DecisionLog,
    nav: NavLevel,
    theme: Theme,
    quit: bool,
}

impl App {
    fn new() -> Self {
        App {
            nav: NavLevel::L1 { selected: 0 },
            log: mock_data(),
            theme: Theme::mocha(),
            quit: false,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        if matches!(code, KeyCode::Char('q')) {
            self.quit = true;
            return;
        }
        match &mut self.nav {
            NavLevel::L1 { selected } => match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if *selected + 1 < self.log.decisions.len() {
                        *selected += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                }
                KeyCode::Enter => {
                    let idx = *selected;
                    self.nav = NavLevel::L2 {
                        decision_idx: idx,
                        impact_idx: 0,
                        focused_chunk: 0,
                        expanded_chunks: HashSet::new(),
                    };
                }
                _ => {}
            },

            NavLevel::L2 {
                decision_idx,
                impact_idx,
                focused_chunk,
                expanded_chunks,
            } => match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    let n = self.log.decisions[*decision_idx].code_impacts[*impact_idx]
                        .line_ranges
                        .len();
                    if *focused_chunk + 1 < n {
                        *focused_chunk += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if *focused_chunk > 0 {
                        *focused_chunk -= 1;
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    let n = self.log.decisions[*decision_idx].code_impacts.len();
                    *impact_idx = (*impact_idx + 1) % n;
                    *focused_chunk = 0;
                    expanded_chunks.clear();
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    let n = self.log.decisions[*decision_idx].code_impacts.len();
                    *impact_idx = impact_idx.checked_sub(1).unwrap_or(n - 1);
                    *focused_chunk = 0;
                    expanded_chunks.clear();
                }
                KeyCode::Tab => {
                    if !expanded_chunks.remove(focused_chunk) {
                        expanded_chunks.insert(*focused_chunk);
                    }
                }
                KeyCode::Esc => {
                    let d = *decision_idx;
                    self.nav = NavLevel::L1 { selected: d };
                }
                _ => {}
            },
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
        NavLevel::L1 { selected } => {
            render_l1(frame, content_area, app, *selected);
        }
        NavLevel::L2 {
            decision_idx,
            impact_idx,
            focused_chunk,
            expanded_chunks,
        } => {
            render_l2(frame, content_area, app, *decision_idx, *impact_idx, *focused_chunk, expanded_chunks);
        }
    }

    let status = match &app.nav {
        NavLevel::L1 { .. } => {
            format!("commit {}   j/k navigate    Enter drill in    q quit", app.log.commit)
        }
        NavLevel::L2 {
            decision_idx,
            impact_idx,
            focused_chunk,
            expanded_chunks,
        } => {
            let total = app.log.decisions[*decision_idx].code_impacts.len();
            let ctx_label = if expanded_chunks.contains(focused_chunk) { "collapse ctx" } else { "expand ctx" };
            format!(
                "commit {}   file {}/{}    h/l files    j/k chunks    Tab {}    Esc back    q quit",
                app.log.commit,
                impact_idx + 1,
                total,
                ctx_label,
            )
        }
    };
    frame.render_widget(
        Paragraph::new(status).style(stylesheet::status_bar(theme)),
        status_area,
    );
}

// L1 — decision list

fn render_l1(frame: &mut Frame, area: Rect, app: &App, selected: usize) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - 4; // 2 bar + 2 inner indent
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(separator_line(cr.width, theme.surface.mantle()));

    for (i, decision) in app.log.decisions.iter().enumerate() {
        let focused = i == selected;
        let card = make_card(cr.width, focused, theme.accents.lavender);
        let n_files = decision.code_impacts.len();

        lines.push(card.line(
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
            theme.surface.surface1(),
        ));

        if let Some(rationale) = decision.rationale {
            for text_line in wrap_text(rationale, text_width) {
                lines.push(card.line(
                    vec![Span::styled(
                        format!("· {}", text_line),
                        Style::default().fg(theme.surface.subtext1()),
                    )],
                    theme.surface.surface1(),
                ));
            }
        }

        for impact in &decision.code_impacts {
            lines.push(card.line(
                vec![Span::styled(
                    format!("{} {}", Icons::FILE_MODIFIED, impact.file),
                    Style::default().fg(theme.surface.text()),
                )],
                theme.surface.surface0(),
            ));
        }

        lines.push(separator_line(cr.width, theme.surface.mantle()));
    }

    frame.render_widget(Paragraph::new(lines), cr);
}

// L2 — pinned file header + scrollable chunk cards; h/l cycles files, j/k navigates chunks.

fn render_l2(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    decision_idx: usize,
    impact_idx: usize,
    focused_chunk: usize,
    expanded_chunks: &HashSet<usize>,
) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - 4;
    let impact = &app.log.decisions[decision_idx].code_impacts[impact_idx];

    let reasoning_lines = wrap_text(impact.reasoning, text_width);
    // mantle fill + file line + reasoning lines + mantle separator
    let header_height = 1 + 1 + reasoning_lines.len() as u16 + 1;

    let [header_area, chunks_area] =
        Layout::vertical([Constraint::Length(header_height), Constraint::Fill(1)]).areas(cr);

    // Pinned header — mantle container bg with surface1 card content.
    let header_card = HierarchicalCard::new(cr.width);
    let mut header_lines: Vec<Line<'static>> = Vec::new();
    header_lines.push(separator_line(cr.width, theme.surface.mantle()));
    header_lines.push(header_card.line(
        vec![Span::styled(
            format!("{} {}", Icons::FILE_MODIFIED, impact.file),
            Style::default().fg(theme.surface.text()).add_modifier(Modifier::BOLD),
        )],
        theme.surface.surface1(),
    ));
    for text_line in reasoning_lines {
        header_lines.push(header_card.line(
            vec![Span::styled(text_line, Style::default().fg(theme.surface.subtext1()))],
            theme.surface.surface1(),
        ));
    }
    header_lines.push(separator_line(cr.width, theme.surface.mantle()));
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(theme.surface.mantle())),
        header_area,
    );
    frame.render_widget(Paragraph::new(header_lines), header_area);

    // Chunk cards — each LineRange is a card; focused one gets the accent bar.
    let mut chunk_lines: Vec<Line<'static>> = Vec::new();
    for (i, range) in impact.line_ranges.iter().enumerate() {
        let card = make_card(cr.width, i == focused_chunk, theme.accents.lavender);
        let chunk_expanded = expanded_chunks.contains(&i);
        let visible_lines: Vec<_> = range
            .lines
            .iter()
            .enumerate()
            .filter(|(_, (kind, _))| chunk_expanded || !matches!(kind, LineKind::Context))
            .collect();
        for (offset, (kind, content)) in &visible_lines {
            let line_no = range.start + *offset as u32;
            let (fg, sigil) = match kind {
                LineKind::Added => (theme.accents.green, "+"),
                LineKind::Removed => (theme.accents.red, "-"),
                LineKind::Context => (theme.surface.subtext0(), " "),
            };
            chunk_lines.push(card.line(
                vec![
                    Span::styled(
                        format!("{:>4} ", line_no),
                        Style::default().fg(theme.surface.overlay0()),
                    ),
                    Span::styled(sigil, Style::default().fg(fg).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" {}", content), Style::default().fg(fg)),
                ],
                theme.surface.base(),
            ));
        }
        if i + 1 < impact.line_ranges.len() {
            // Freestanding skip-marker between chunk cards — not owned by either card.
            chunk_lines.push(Line::styled(
                format!("{:>6}", "···"),
                Style::default()
                    .fg(theme.surface.overlay0())
                    .bg(theme.surface.mantle()),
            ));
        }
    }
    let scroll = chunks_scroll_offset(impact, focused_chunk, chunks_area.height, expanded_chunks);
    frame.render_widget(Paragraph::new(chunk_lines).scroll((scroll, 0)), chunks_area);
}

fn visible_line_count(range: &LineRange, expanded: bool) -> u16 {
    if expanded {
        range.lines.len() as u16
    } else {
        range.lines.iter().filter(|(k, _)| !matches!(k, LineKind::Context)).count() as u16
    }
}

fn chunks_scroll_offset(impact: &CodeImpact, focused_chunk: usize, viewport: u16, expanded_chunks: &HashSet<usize>) -> u16 {
    // Compute start line of the focused chunk.
    let mut chunk_start = 0u16;
    for i in 0..focused_chunk {
        chunk_start += visible_line_count(&impact.line_ranges[i], expanded_chunks.contains(&i));
        chunk_start += 1; // skip-marker between chunks
    }
    let chunk_height = visible_line_count(&impact.line_ranges[focused_chunk], expanded_chunks.contains(&focused_chunk));

    // If the focused chunk is visible from scroll=0, don't scroll.
    if chunk_start + chunk_height <= viewport {
        return 0;
    }
    // Otherwise scroll so the focused chunk starts at the top.
    chunk_start
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
        {
            app.handle_key(key.code);
        }

        if app.quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
