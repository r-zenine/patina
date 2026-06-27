//! Review Navigator — three-level hierarchical navigation prototype.
//!
//! L1  j/k navigate decisions  | Enter drill in | q quit
//! L2  j/k navigate impacts    | Enter enter code-scroll mode | i toggle instructions | Esc back
//! L2* j/k scroll code         | i toggle instructions | Esc exit code-scroll | q quit
//!
//! Surface ramp (dark theme, lighter = higher elevation):
//!   rationale    → layer_elevated (surface1)
//!   instructions → layer_raised   (surface0)
//!   code lines   → layer_base     (base)
//!   skip marker  → no bg          (terminal mantle floor)
//!
//! Layout: content capped at 120 columns, centered; surface bg fills full column width.

use std::io;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use tui_design::{Icons, Theme, stylesheet};

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
                                start: 12,
                                end: 15,
                                lines: vec![
                                    (LineKind::Context, "use crate::session::SessionStore;"),
                                    (
                                        LineKind::Removed,
                                        "pub fn validate_token(token: &str, store: &SessionStore) -> Result<Claims> {",
                                    ),
                                    (
                                        LineKind::Added,
                                        "pub fn validate_token<V: TokenValidator>(token: &str, v: &V) -> Result<Claims> {",
                                    ),
                                    (LineKind::Context, "    let raw = decode_jwt(token)?;"),
                                ],
                            },
                            LineRange {
                                start: 42,
                                end: 44,
                                lines: vec![
                                    (LineKind::Context, "impl Default for Middleware {"),
                                    (
                                        LineKind::Removed,
                                        "    fn default() -> Self { Self { store: SessionStore::new() } }",
                                    ),
                                    (
                                        LineKind::Added,
                                        "    fn default() -> Self { Self { validator: Arc::new(DefaultValidator::new()) } }",
                                    ),
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
                            start: 8,
                            end: 11,
                            lines: vec![
                                (LineKind::Context, "pub enum TokenType {"),
                                (LineKind::Context, "    Session,"),
                                (LineKind::Added, "    ApiKey,"),
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
                    line_ranges: vec![LineRange {
                        start: 1,
                        end: 6,
                        lines: vec![
                            (LineKind::Added, "use std::time::{Duration, Instant};"),
                            (LineKind::Added, ""),
                            (LineKind::Added, "pub struct SlidingWindowLimiter {"),
                            (LineKind::Added, "    window: Duration,"),
                            (LineKind::Added, "    max_requests: u32,"),
                            (LineKind::Added, "}"),
                        ],
                    }],
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
    // code_scroll: None  = impact-navigate mode
    // code_scroll: Some  = code-scroll mode (was a separate L3 variant)
    L2 {
        decision_idx: usize,
        focused_impact: usize,
        code_scroll: Option<u16>,
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
                        focused_impact: 0,
                        code_scroll: None,
                    };
                }
                _ => {}
            },

            NavLevel::L2 {
                decision_idx,
                focused_impact,
                code_scroll,
            } => match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some(s) = code_scroll.as_mut() {
                        *s = s.saturating_add(1);
                    } else {
                        let n = self.log.decisions[*decision_idx].code_impacts.len();
                        if *focused_impact + 1 < n {
                            *focused_impact += 1;
                        }
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if let Some(s) = code_scroll.as_mut() {
                        *s = s.saturating_sub(1);
                    } else if *focused_impact > 0 {
                        *focused_impact -= 1;
                    }
                }
                KeyCode::Enter => {
                    if code_scroll.is_none() {
                        *code_scroll = Some(0);
                    }
                }
                KeyCode::Esc => {
                    if code_scroll.is_some() {
                        *code_scroll = None;
                    } else {
                        let d = *decision_idx;
                        self.nav = NavLevel::L1 { selected: d };
                    }
                }
                _ => {}
            },
        }
    }
}

// ── Layout helpers ────────────────────────────────────────────────────────────

const CONTENT_WIDTH: u16 = 120;
const INDENT: usize = 2;

// Lines emitted by render_l2 before the impact loop (title + rationale lines + blank).
// Updated dynamically; this constant covers the fixed title + blank.
const L2_FIXED_HEADER_LINES: u16 = 2;
// Lines in the between-impact separator (two blank lines).
const IMPACT_SEPARATOR_LINES: u16 = 2;

fn content_rect(area: Rect) -> Rect {
    let w = CONTENT_WIDTH.min(area.width);
    let x = area.x + (area.width - w) / 2;
    Rect {
        x,
        width: w,
        ..area
    }
}

// Line indented by INDENT with bg painted on every cell up to col_width.
// `accent` draws a 1-char ▌ bar in the first column (focused indicator); None = blank.
// Explicit trailing span required — Line::style alone does not fill remaining cells.
fn padded_line<'a>(
    col_width: u16,
    spans: Vec<Span<'a>>,
    bg: Color,
    accent: Option<Color>,
) -> Line<'a> {
    let content_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let used = INDENT + content_len;
    let trailing = (col_width as usize).saturating_sub(used);
    let bar = match accent {
        Some(c) => Span::styled("▌ ", Style::default().fg(c).bg(bg)),
        None => Span::styled("  ", Style::default().bg(bg)),
    };
    let mut all = vec![bar];
    for s in spans {
        all.push(Span::styled(s.content, s.style.bg(bg)));
    }
    all.push(Span::styled(" ".repeat(trailing), Style::default().bg(bg)));
    Line::from(all)
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

fn skip_marker(col_width: u16, fg: Color, bg: Color, accent: Option<Color>) -> Line<'static> {
    let bar = match accent {
        Some(c) => Span::styled("▌ ", Style::default().fg(c).bg(bg)),
        None => Span::styled("  ", Style::default().bg(bg)),
    };
    // Center "···" in the full col_width (matching Alignment::Center), then subtract the 2-char bar.
    let left_pad = (col_width as usize).saturating_sub(3) / 2;
    let pad = left_pad.saturating_sub(2);
    let trailing = (col_width as usize).saturating_sub(2 + pad + 3);
    Line::from(vec![
        bar,
        Span::styled(" ".repeat(pad), Style::default().bg(bg)),
        Span::styled("···", Style::default().fg(fg).bg(bg)),
        Span::styled(" ".repeat(trailing), Style::default().bg(bg)),
    ])
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(theme.surface.mantle())),
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
            focused_impact,
            code_scroll: None,
        } => {
            render_l2(frame, content_area, app, *decision_idx, *focused_impact);
        }
        NavLevel::L2 {
            decision_idx,
            focused_impact,
            code_scroll: Some(scroll),
        } => {
            render_l3(
                frame,
                content_area,
                app,
                *decision_idx,
                *focused_impact,
                *scroll,
            );
        }
    }

    let nav_hint = match &app.nav {
        NavLevel::L1 { .. } => " j/k navigate    Enter drill in    q quit",
        NavLevel::L2 {
            code_scroll: None, ..
        } => " j/k navigate impacts    Enter scroll code    Esc back    q quit",
        NavLevel::L2 {
            code_scroll: Some(_),
            ..
        } => " j/k scroll code    Esc exit scroll    q quit",
    };
    let status = format!("commit {}  {}", app.log.commit, nav_hint);
    frame.render_widget(
        Paragraph::new(status).style(stylesheet::status_bar(theme)),
        status_area,
    );
}

// L1 — decision list

fn render_l1(frame: &mut Frame, area: Rect, app: &App, selected: usize) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - INDENT * 2;
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(padded_line(cr.width, vec![], theme.surface.mantle(), None));

    for (i, decision) in app.log.decisions.iter().enumerate() {
        let focused = i == selected;
        let bar = if focused {
            Some(theme.accents.lavender)
        } else {
            None
        };
        let header_bg = theme.surface.surface1();
        let header_fg = theme.surface.text();
        let n_files = decision.code_impacts.len();

        lines.push(padded_line(
            cr.width,
            vec![
                Span::styled(
                    format!("#{} {}", decision.number, decision.title),
                    Style::default().fg(header_fg).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} file{}", n_files, plural_s(n_files)),
                    Style::default().fg(theme.surface.overlay0()),
                ),
            ],
            header_bg,
            bar,
        ));

        if let Some(rationale) = decision.rationale {
            let rationale_bg = theme.surface.surface1();
            for text_line in wrap_text(rationale, text_width) {
                lines.push(padded_line(
                    cr.width,
                    vec![Span::styled(
                        format!("· {}", text_line),
                        Style::default().fg(theme.surface.subtext1()),
                    )],
                    rationale_bg,
                    bar,
                ));
            }
        }

        for impact in &decision.code_impacts {
            lines.push(padded_line(
                cr.width,
                vec![Span::styled(
                    format!("{} {}", Icons::FILE_MODIFIED, impact.file),
                    Style::default().fg(theme.surface.text()),
                )],
                theme.surface.surface0(),
                bar,
            ));
        }

        lines.push(padded_line(cr.width, vec![], theme.surface.mantle(), None));
    }

    frame.render_widget(Paragraph::new(lines), cr);
}

// L2 — all impacts shown at full detail; focused one uses elevated surfaces.

fn render_l2(frame: &mut Frame, area: Rect, app: &App, decision_idx: usize, focused_impact: usize) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - INDENT * 2;
    let decision = &app.log.decisions[decision_idx];
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(padded_line(
        cr.width,
        vec![Span::styled(
            format!("#{} {}", decision.number, decision.title),
            Style::default()
                .fg(theme.surface.text())
                .add_modifier(Modifier::BOLD),
        )],
        theme.surface.subtext0(),
        None,
    ));
    lines.push(padded_line(
        cr.width,
        vec![],
        theme.surface.surface0(),
        None,
    ));

    for (i, impact) in decision.code_impacts.iter().enumerate() {
        let focused = i == focused_impact;
        push_impact_detail(&mut lines, cr.width, theme, impact, text_width, focused);

        if i + 1 < decision.code_impacts.len() {
            lines.push(padded_line(cr.width, vec![], theme.surface.mantle(), None));
            lines.push(padded_line(cr.width, vec![], theme.surface.mantle(), None));
        }
    }

    let scroll = l2_focused_offset(decision, focused_impact, text_width);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), cr);
}

// Exact line count for one impact block (must mirror push_impact_detail).
fn impact_line_count(impact: &CodeImpact, text_width: usize) -> u16 {
    let mut h = 1u16; // file header
    h += 1; // reasoning label
    h += wrap_text(impact.reasoning, text_width).len() as u16;
    h += 1; // blank after reasoning
    for (idx, range) in impact.line_ranges.iter().enumerate() {
        h += range.lines.len() as u16;
        if idx + 1 < impact.line_ranges.len() {
            h += 1;
        } // intra-range skip marker
    }
    h
}

fn l2_focused_offset(decision: &Decision, focused_impact: usize, text_width: usize) -> u16 {
    let mut offset = L2_FIXED_HEADER_LINES;
    for i in 0..focused_impact {
        offset += impact_line_count(&decision.code_impacts[i], text_width);
        offset += IMPACT_SEPARATOR_LINES;
    }
    offset
}

// L2* (code-scroll mode) — focused impact fullscreen, j/k scrolls

fn render_l3(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    decision_idx: usize,
    impact_idx: usize,
    code_scroll: u16,
) {
    let theme = &app.theme;
    let cr = content_rect(area);
    let text_width = cr.width as usize - INDENT * 2;
    let impact = &app.log.decisions[decision_idx].code_impacts[impact_idx];
    let mut lines: Vec<Line<'static>> = Vec::new();

    push_impact_detail(&mut lines, cr.width, theme, impact, text_width, true);

    frame.render_widget(Paragraph::new(lines).scroll((code_scroll, 0)), cr);
}

// File header + reasoning + line ranges.
// `focused` draws a lavender accent bar spanning every line of this block.
fn push_impact_detail(
    lines: &mut Vec<Line<'static>>,
    col_width: u16,
    theme: &Theme,
    impact: &CodeImpact,
    text_width: usize,
    focused: bool,
) {
    let bar = if focused {
        Some(theme.accents.lavender)
    } else {
        None
    };
    let reasoning_bg = theme.surface.surface1();

    lines.push(padded_line(
        col_width,
        vec![Span::styled(
            format!("{} {}", Icons::FILE_MODIFIED, impact.file),
            Style::default()
                .fg(theme.surface.text())
                .add_modifier(Modifier::BOLD),
        )],
        reasoning_bg,
        bar,
    ));

    for text_line in wrap_text(impact.reasoning, text_width) {
        lines.push(padded_line(
            col_width,
            vec![Span::styled(
                text_line,
                Style::default().fg(theme.surface.subtext1()),
            )],
            reasoning_bg,
            bar,
        ));
    }
    lines.push(padded_line(col_width, vec![], reasoning_bg, bar));

    let code_bg = theme.surface.surface0();
    for (range_idx, range) in impact.line_ranges.iter().enumerate() {
        for (offset, (kind, content)) in range.lines.iter().enumerate() {
            let line_no = range.start + offset as u32;
            let (fg, sigil) = match kind {
                LineKind::Added => (theme.accents.green, "+"),
                LineKind::Removed => (theme.accents.red, "-"),
                LineKind::Context => (theme.surface.subtext0(), " "),
            };
            lines.push(padded_line(
                col_width,
                vec![
                    Span::styled(
                        format!("{:>4} ", line_no),
                        Style::default().fg(theme.surface.overlay0()),
                    ),
                    Span::styled(sigil, Style::default().fg(fg).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" {}", content), Style::default().fg(fg)),
                ],
                code_bg,
                bar,
            ));
        }
        if range_idx + 1 < impact.line_ranges.len() {
            lines.push(skip_marker(
                col_width,
                theme.surface.overlay0(),
                theme.surface.base(),
                bar,
            ));
        }
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

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
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
