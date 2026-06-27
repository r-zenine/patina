//! Status bar component showing keybindings and current state

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
};
use tui_design::{Theme, stylesheet};

use crate::state::{FocusPanel, InputMode, UiState};
use diffviz_review::engines::ReviewEngine;

/// Render the status bar
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    let theme = Theme::mocha();
    match &ui_state.input_mode {
        InputMode::Navigation => render_navigation_status(f, area, ui_state, review_engine, &theme),
        InputMode::Instruction { .. } => render_input_status(
            f,
            area,
            "Instruction",
            "Enter to submit, Esc to cancel",
            &theme,
        ),
        InputMode::DecisionInstruction { .. } => render_input_status(
            f,
            area,
            "Decision Instruction",
            "Enter to submit, Esc to cancel",
            &theme,
        ),
    }
}

fn render_navigation_status(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(60)])
        .split(area);

    let state_info = create_state_info(ui_state, review_engine, theme);
    let bar_style = stylesheet::status_bar(theme).patch(stylesheet::layer_base(theme));
    let state_paragraph = Paragraph::new(state_info).style(bar_style);
    f.render_widget(state_paragraph, chunks[0]);

    let keybindings = match ui_state.focused_panel {
        FocusPanel::FileList => create_file_list_keybindings(theme),
        FocusPanel::DiffView => create_diff_view_keybindings(theme),
    };
    let keys_paragraph = Paragraph::new(keybindings)
        .style(bar_style)
        .alignment(ratatui::layout::Alignment::Right);
    f.render_widget(keys_paragraph, chunks[1]);
}

fn render_input_status(f: &mut Frame, area: Rect, mode_name: &str, help_text: &str, theme: &Theme) {
    let status_line = Line::from(vec![
        Span::styled(
            format!("{mode_name}: "),
            stylesheet::keybind_key(theme).add_modifier(Modifier::BOLD),
        ),
        Span::styled(help_text, stylesheet::muted(theme)),
    ]);

    let paragraph = Paragraph::new(status_line)
        .style(stylesheet::status_bar(theme).patch(stylesheet::layer_base(theme)));
    f.render_widget(paragraph, area);
}

fn create_state_info(
    _ui_state: &UiState,
    review_engine: &ReviewEngine,
    theme: &Theme,
) -> Line<'static> {
    let mut spans = Vec::new();

    let total_decisions = review_engine.get_all_decisions().len();
    let approved_decisions = review_engine.get_approved_decisions_count();

    spans.push(Span::styled(
        format!("Decision: {approved_decisions}/{total_decisions}"),
        if total_decisions == approved_decisions {
            stylesheet::success(theme)
        } else {
            stylesheet::body(theme)
        },
    ));

    let total_reviewables = review_engine.get_ordered_reviewable_ids().len();
    let total_approved = review_engine
        .get_ordered_reviewable_ids()
        .iter()
        .filter(|id| review_engine.state().is_approved(id))
        .count();

    spans.push(Span::styled(" | ", stylesheet::muted(theme)));
    spans.push(Span::styled(
        format!("Overall: {total_approved}/{total_reviewables}"),
        if total_approved == total_reviewables {
            stylesheet::success(theme)
        } else {
            stylesheet::body(theme)
        },
    ));

    Line::from(spans)
}

fn create_file_list_keybindings(theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("j/k", stylesheet::keybind_key(theme)),
        Span::styled(" nav  ", stylesheet::keybind_desc(theme)),
        Span::styled("Space", stylesheet::keybind_key(theme)),
        Span::styled(" leader  ", stylesheet::keybind_desc(theme)),
        Span::styled("?", stylesheet::keybind_key(theme)),
        Span::styled(" help  ", stylesheet::keybind_desc(theme)),
        Span::styled("q", stylesheet::keybind_key(theme)),
        Span::styled(" quit", stylesheet::keybind_desc(theme)),
    ])
}

fn create_diff_view_keybindings(theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("j/k", stylesheet::keybind_key(theme)),
        Span::styled(" nav  ", stylesheet::keybind_desc(theme)),
        Span::styled("v", stylesheet::keybind_key(theme)),
        Span::styled(" visual  ", stylesheet::keybind_desc(theme)),
        Span::styled("Space", stylesheet::keybind_key(theme)),
        Span::styled(" leader  ", stylesheet::keybind_desc(theme)),
        Span::styled("?", stylesheet::keybind_key(theme)),
        Span::styled(" help", stylesheet::keybind_desc(theme)),
    ])
}
