use std::{collections::HashMap, error::Error, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use diffviz_core::{
    ast_diff::strategies::ASTChangeType,
    ast_diff::{LineRange, RelevanceScore, BACKGROUND, ESSENTIAL, IMPORTANT},
    common::{ProgrammingLanguage, SemanticNodeKind},
    renderable_diff::{
        ChangeType, LineAnnotation, RenderableDiff, RenderableLine, RenderableMetadata,
        SemanticAnchor, SemanticAnchorType,
    },
};
use diffviz_review_tui::ui::components::renderable_diff_widget::RenderableDiffWidget;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

struct DemoState {
    show_all_context: bool,
    highlight_semantics: bool,
    scroll_offset: usize,
    cursor_index: usize,
    selection_anchor: Option<usize>,
    selection_range: Option<(usize, usize)>,
    view_height: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let diff = create_mock_renderable_diff();
    let result = run_demo(&mut terminal, diff);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("demo error: {err}");
    }

    Ok(())
}

fn run_demo(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    diff: RenderableDiff<'static>,
) -> Result<(), Box<dyn Error>> {
    let mut state = DemoState {
        show_all_context: false,
        highlight_semantics: true,
        scroll_offset: 0,
        cursor_index: 0,
        selection_anchor: None,
        selection_range: None,
        view_height: 0,
    };

    let total_lines = diff.lines.len();
    if total_lines == 0 {
        return Ok(());
    }

    let update_selection_range = |state: &mut DemoState| {
        if let Some(anchor) = state.selection_anchor {
            let start = anchor.min(state.cursor_index) + 1;
            let end = anchor.max(state.cursor_index) + 1;
            state.selection_range = Some((start, end));
        }
    };

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
                .split(frame.area());

            state.view_height = chunks[0].height as usize;
            let total_lines = diff.lines.len();
            if total_lines == 0 {
                return;
            }
            if state.cursor_index >= total_lines {
                state.cursor_index = total_lines - 1;
            }

            let max_offset = total_lines.saturating_sub(state.view_height.max(1));
            if state.scroll_offset > max_offset {
                state.scroll_offset = max_offset;
            }

            if state.cursor_index < state.scroll_offset {
                state.scroll_offset = state.cursor_index;
            } else if state.cursor_index >= state.scroll_offset + state.view_height.max(1) {
                state.scroll_offset = state.cursor_index + 1 - state.view_height.max(1);
            }

            let cursor_line_num = diff.lines[state.cursor_index].line_number;

            let widget = RenderableDiffWidget::new(&diff)
                .with_context(state.show_all_context)
                .highlight_semantics(state.highlight_semantics)
                .show_inline_old(true)
                .with_scroll_offset(state.scroll_offset)
                .with_selection(state.selection_range)
                .with_cursor_line(Some(cursor_line_num));

            frame.render_widget(widget, chunks[0]);

            let help_lines = vec![
                Line::from(vec![
                    Span::styled(
                        "q/Ctrl+c",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" quit  "),
                    Span::styled(
                        "z",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" toggle context  "),
                    Span::styled(
                        "s",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" toggle semantics  "),
                    Span::styled(
                        "v",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" visual select"),
                ]),
                Line::from(vec![Span::styled(
                    format!(
                        "cursor: L{}  context: {}  semantics: {}  inline-old: on  selection: {}  nav: ↑/↓ j/k  g/G  PgUp/PgDn Ctrl+u/d  Home/End  |  visual: v to mark/clear",
                        cursor_line_num,
                        if state.show_all_context {
                            "all"
                        } else {
                            "folded"
                        },
                        if state.highlight_semantics {
                            "on"
                        } else {
                            "off"
                        },
                        state
                            .selection_range
                            .map(|(start, end)| {
                                if start == end {
                                    format!("L{start}")
                                } else {
                                    format!("L{start}-L{end}")
                                }
                            })
                            .unwrap_or_else(|| "none".to_string())
                    ),
                    Style::default().fg(Color::Gray),
                )]),
            ];

            let help = Paragraph::new(help_lines)
                .block(Block::default().borders(Borders::TOP).title("controls"));

            frame.render_widget(help, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(150))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('z'),
                    kind: KeyEventKind::Press,
                    ..
                }) => state.show_all_context = !state.show_all_context,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('s'),
                    kind: KeyEventKind::Press,
                    ..
                }) => state.highlight_semantics = !state.highlight_semantics,
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    kind: KeyEventKind::Press,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if state.cursor_index + 1 < total_lines {
                        state.cursor_index += 1;
                        update_selection_range(&mut state);
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    kind: KeyEventKind::Press,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if state.cursor_index > 0 {
                        state.cursor_index -= 1;
                        update_selection_range(&mut state);
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::PageDown,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let step = state.view_height.max(1);
                    state.cursor_index = (state.cursor_index + step).min(total_lines - 1);
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::PageUp,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let step = state.view_height.max(1);
                    state.cursor_index = state.cursor_index.saturating_sub(step);
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    state.cursor_index = 0;
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::End,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    state.cursor_index = total_lines - 1;
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('g'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    state.cursor_index = 0;
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('G'),
                    modifiers: crossterm::event::KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    state.cursor_index = total_lines - 1;
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let step = state.view_height.max(1);
                    state.cursor_index = state.cursor_index.saturating_sub(step);
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let step = state.view_height.max(1);
                    state.cursor_index = (state.cursor_index + step).min(total_lines - 1);
                    update_selection_range(&mut state);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('v'),
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    // Vim-like visual select cycling: none → start → finalize → clear
                    if state.selection_anchor.is_some() {
                        // Finalize selection, clear anchor
                        state.selection_anchor = None;
                    } else if state.selection_range.is_some() {
                        // Clear selection entirely
                        state.selection_range = None;
                    } else {
                        // Start new selection
                        state.selection_anchor = Some(state.cursor_index);
                        state.selection_range =
                            Some((state.cursor_index + 1, state.cursor_index + 1));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn create_mock_renderable_diff() -> RenderableDiff<'static> {
    #[allow(clippy::too_many_arguments)]
    fn make_line(
        line_number: usize,
        content: &'static str,
        change: Option<ChangeType>,
        relevance: RelevanceScore,
        semantic_kind: SemanticNodeKind,
        anchor: Option<(SemanticAnchorType, &'static str)>,
        node_depth: usize,
        span: Option<(usize, usize)>,
    ) -> RenderableLine<'static> {
        let (start_col, end_col) = span.unwrap_or((0, content.len()));
        let annotations = vec![LineAnnotation {
            start_col,
            end_col,
            relevance,
            change_type: change.clone(),
            semantic_kind: semantic_kind.clone(),
            node_depth,
        }];

        RenderableLine {
            line_number,
            content,
            byte_range: (0, content.len()),
            annotations,
            semantic_anchor: anchor.map(|(anchor_type, identifier)| SemanticAnchor {
                anchor_type,
                identifier: identifier.to_string(),
            }),
        }
    }

    let lines = vec![
        make_line(
            1,
            "impl Cart {",
            None,
            IMPORTANT,
            SemanticNodeKind::ImplBlock,
            Some((SemanticAnchorType::StructDeclaration, "Cart")),
            0,
            None,
        ),
        make_line(
            2,
            "    pub fn total(&self) -> f64 {",
            Some(ChangeType::Deleted),
            ESSENTIAL,
            SemanticNodeKind::Function,
            Some((SemanticAnchorType::FunctionSignature, "Cart::total")),
            1,
            Some((24, 29)),
        ),
        make_line(
            3,
            "    pub fn total(&self) -> i32 {",
            Some(ChangeType::Modified),
            ESSENTIAL,
            SemanticNodeKind::Function,
            Some((SemanticAnchorType::FunctionSignature, "Cart::total")),
            1,
            Some((24, 29)),
        ),
        make_line(
            4,
            "        let mut sum = 0;",
            Some(ChangeType::Deleted),
            ESSENTIAL,
            SemanticNodeKind::Statement,
            None,
            2,
            None,
        ),
        make_line(
            5,
            "        for item in &self.items {",
            None,
            IMPORTANT,
            SemanticNodeKind::Statement,
            None,
            2,
            None,
        ),
        make_line(
            6,
            "            sum += item.price();",
            Some(ChangeType::Deleted),
            ESSENTIAL,
            SemanticNodeKind::Statement,
            None,
            3,
            Some((12, 33)),
        ),
        make_line(
            7,
            "            sum += item.total();",
            Some(ChangeType::Modified),
            ESSENTIAL,
            SemanticNodeKind::Statement,
            None,
            3,
            Some((23, 28)),
        ),
        make_line(
            8,
            "        }",
            None,
            IMPORTANT,
            SemanticNodeKind::Statement,
            None,
            2,
            None,
        ),
        make_line(
            9,
            "        self.items.iter().map(|item| item.total()).sum()",
            Some(ChangeType::Added),
            ESSENTIAL,
            SemanticNodeKind::Expression,
            None,
            2,
            Some((8, 60)),
        ),
        make_line(
            10,
            "    }",
            None,
            IMPORTANT,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            11,
            "",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            0,
            Some((0, 0)),
        ),
        make_line(
            12,
            "    pub fn add_discount(&mut self, discount: f32) {",
            Some(ChangeType::Added),
            ESSENTIAL,
            SemanticNodeKind::Function,
            Some((SemanticAnchorType::FunctionSignature, "Cart::add_discount")),
            1,
            None,
        ),
        make_line(
            13,
            "        self.discounts.push(discount);",
            Some(ChangeType::Added),
            ESSENTIAL,
            SemanticNodeKind::Statement,
            None,
            2,
            None,
        ),
        make_line(
            14,
            "    }",
            Some(ChangeType::Added),
            IMPORTANT,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            15,
            "    fn helper(&self) {",
            Some(ChangeType::Moved),
            BACKGROUND,
            SemanticNodeKind::Function,
            Some((SemanticAnchorType::FunctionSignature, "Cart::helper")),
            1,
            None,
        ),
        make_line(
            16,
            "        self.rebuild_cache();",
            Some(ChangeType::Reordered),
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            2,
            Some((8, 26)),
        ),
        make_line(
            17,
            "    }",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            18,
            "",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            0,
            None,
        ),
        make_line(
            19,
            "    // TODO: apply loyalty discounts",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            20,
            "    // Additional helper analytics",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            21,
            "    fn debug_items(&self) {",
            Some(ChangeType::Added),
            BACKGROUND,
            SemanticNodeKind::Function,
            Some((SemanticAnchorType::FunctionSignature, "Cart::debug_items")),
            1,
            None,
        ),
        make_line(
            22,
            "        for item in &self.items {",
            Some(ChangeType::Added),
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            2,
            None,
        ),
        make_line(
            23,
            "            tracing::debug!(?item);",
            Some(ChangeType::Added),
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            3,
            None,
        ),
        make_line(
            24,
            "        }",
            Some(ChangeType::Added),
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            2,
            None,
        ),
        make_line(
            25,
            "    }",
            Some(ChangeType::Added),
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            26,
            "",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            0,
            None,
        ),
        make_line(
            27,
            "    pub fn total_items(&self) -> usize {",
            None,
            BACKGROUND,
            SemanticNodeKind::Function,
            Some((SemanticAnchorType::FunctionSignature, "Cart::total_items")),
            1,
            None,
        ),
        make_line(
            28,
            "        self.items.len()",
            None,
            BACKGROUND,
            SemanticNodeKind::Expression,
            None,
            2,
            None,
        ),
        make_line(
            29,
            "    }",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            30,
            "    pub fn total_discounts(&self) -> usize {",
            None,
            BACKGROUND,
            SemanticNodeKind::Function,
            Some((
                SemanticAnchorType::FunctionSignature,
                "Cart::total_discounts",
            )),
            1,
            None,
        ),
        make_line(
            31,
            "        self.discounts.len()",
            None,
            BACKGROUND,
            SemanticNodeKind::Expression,
            None,
            2,
            None,
        ),
        make_line(
            32,
            "    }",
            None,
            BACKGROUND,
            SemanticNodeKind::Statement,
            None,
            1,
            None,
        ),
        make_line(
            33,
            "}",
            None,
            IMPORTANT,
            SemanticNodeKind::ImplBlock,
            None,
            0,
            None,
        ),
    ];

    let essential_line_count = lines
        .iter()
        .filter(|line| line.max_relevance() == ESSENTIAL)
        .count();

    let mut change_summary: HashMap<ASTChangeType, usize> = HashMap::new();
    for line in &lines {
        if let Some(change_type) = line.primary_change_type() {
            let entry = match change_type {
                ChangeType::Modified => ASTChangeType::Content,
                ChangeType::Added | ChangeType::Deleted | ChangeType::Moved => {
                    ASTChangeType::Structural
                }
                ChangeType::Reordered => ASTChangeType::Reorder,
            };
            *change_summary.entry(entry).or_insert(0) += 1;
        }
    }

    let total_changes = change_summary.values().sum();

    let changed_line_numbers = lines
        .iter()
        .filter(|line| line.has_changes())
        .map(|line| line.line_number)
        .collect::<Vec<_>>();

    let metadata = RenderableMetadata {
        total_changes,
        change_summary,
        essential_line_count,
        boundary_name: "Cart::total".to_string(),
        overall_line_range: LineRange {
            start_line: lines.first().map(|l| l.line_number).unwrap_or(1),
            end_line: lines.last().map(|l| l.line_number).unwrap_or(1),
            start_column: 0,
            end_column: 0,
        },
        changed_line_numbers,
    };

    RenderableDiff {
        lines,
        metadata,
        language: ProgrammingLanguage::Rust,
    }
}
