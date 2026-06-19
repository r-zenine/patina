//! Status bar component showing keybindings and current state

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    state::{FocusPanel, InputMode, UiState},
    theme::{Colors, Styles},
};
use diffviz_review::engines::ReviewEngine;

/// Render the status bar
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, review_engine: &ReviewEngine) {
    match &ui_state.input_mode {
        InputMode::Navigation => render_navigation_status(f, area, ui_state, review_engine),
        InputMode::Instruction { .. } => {
            render_input_status(f, area, "Instruction", "Enter to submit, Esc to cancel")
        }
        InputMode::DecisionInstruction { .. } => render_input_status(
            f,
            area,
            "Decision Instruction",
            "Enter to submit, Esc to cancel",
        ),
    }
}

/// Render status bar in navigation mode
fn render_navigation_status(
    f: &mut Frame,
    area: Rect,
    ui_state: &UiState,
    review_engine: &ReviewEngine,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),    // Left section with current state
            Constraint::Length(60), // Right section with keybindings
        ])
        .split(area);

    // Left section: Current state info
    let state_info = create_state_info(ui_state, review_engine);
    let state_paragraph = Paragraph::new(state_info).style(Styles::status_bar());
    f.render_widget(state_paragraph, chunks[0]);

    // Right section: Key bindings based on focused panel
    let keybindings = match ui_state.focused_panel {
        FocusPanel::FileList => create_file_list_keybindings(),
        FocusPanel::DiffView => create_diff_view_keybindings(),
    };
    let keys_paragraph = Paragraph::new(keybindings).style(Styles::status_bar());
    f.render_widget(keys_paragraph, chunks[1]);
}

/// Render status bar in input modes
fn render_input_status(f: &mut Frame, area: Rect, mode_name: &str, help_text: &str) {
    let status_line = Line::from(vec![
        Span::styled(
            format!("{mode_name}: "),
            Style::default()
                .fg(Colors::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(help_text, Styles::muted()),
    ]);

    let paragraph = Paragraph::new(status_line).style(Styles::status_bar());
    f.render_widget(paragraph, area);
}

/// Create state information for status bar
fn create_state_info(ui_state: &UiState, review_engine: &ReviewEngine) -> Line<'static> {
    let mut spans = Vec::new();

    // Current file info
    if let Some(file_path) = ui_state.current_file_path() {
        let file_name = file_path.split('/').next_back().unwrap_or(&file_path);
        spans.push(Span::styled(
            format!("📄 {file_name}"),
            Style::default().fg(Colors::BLUE),
        ));
    }

    let total_decisions = review_engine.get_all_decisions().len();
    let approved_decisions = review_engine.get_approved_decisions_count();

    spans.push(Span::styled(" | ", Styles::muted()));
    spans.push(Span::styled(
        format!("Decision: {approved_decisions}/{total_decisions}"),
        if total_decisions == approved_decisions {
            Style::default().fg(Colors::GREEN)
        } else {
            Style::default().fg(Colors::WHITE)
        },
    ));

    // Overall progress
    //
    let total_reviewables = review_engine.get_ordered_reviewable_ids().len();
    let total_approved = review_engine
        .get_ordered_reviewable_ids()
        .iter()
        .filter(|id| review_engine.state().is_approved(id))
        .count();

    spans.push(Span::styled(" | ", Styles::muted()));
    spans.push(Span::styled(
        format!("Overall: {total_approved}/{total_reviewables}"),
        if total_approved == total_reviewables {
            Style::default().fg(Colors::GREEN)
        } else {
            Style::default().fg(Colors::WHITE)
        },
    ));

    // Context display mode
    spans.push(Span::styled(" | ", Styles::muted()));
    spans.push(Span::styled(
        if ui_state.show_all_context {
            "Full Context"
        } else {
            "Folded"
        },
        Style::default().fg(Colors::CYAN),
    ));

    Line::from(spans)
}

/// Create keybindings for file list panel
fn create_file_list_keybindings() -> Line<'static> {
    Line::from(vec![
        Span::styled("j/k", Style::default().fg(Colors::CYAN)),
        Span::styled(" nav  ", Styles::muted()),
        Span::styled("Space", Style::default().fg(Colors::CYAN)),
        Span::styled(" leader  ", Styles::muted()),
        Span::styled("?", Style::default().fg(Colors::CYAN)),
        Span::styled(" help  ", Styles::muted()),
        Span::styled("q", Style::default().fg(Colors::CYAN)),
        Span::styled(" quit", Styles::muted()),
    ])
}

/// Create keybindings for diff view panel
fn create_diff_view_keybindings() -> Line<'static> {
    Line::from(vec![
        Span::styled("j/k", Style::default().fg(Colors::CYAN)),
        Span::styled(" nav  ", Styles::muted()),
        Span::styled("v", Style::default().fg(Colors::CYAN)),
        Span::styled(" visual  ", Styles::muted()),
        Span::styled("Space", Style::default().fg(Colors::CYAN)),
        Span::styled(" leader  ", Styles::muted()),
        Span::styled("?", Style::default().fg(Colors::CYAN)),
        Span::styled(" help", Styles::muted()),
    ])
}
