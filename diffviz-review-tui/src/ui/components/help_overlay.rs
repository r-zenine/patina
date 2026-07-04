//! Help overlay component showing all keybindings
//!
//! Displays a comprehensive reference of all available keybindings when '?'
//! is pressed. Content is rendered from the keybinding registry
//! (`events::bindings`) — the overlay cannot drift from dispatch.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_design::{Theme, stylesheet};

use crate::events::UiEvent;
use crate::events::bindings::{self, BindingScope, INPUT_CATCH_ALL, SUBMENUS};
use crate::state::UiState;

pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.show_help {
        return;
    }

    let theme = Theme::mocha();
    let area = centered_popup(f.area(), 80, 80);

    f.render_widget(Clear, area);

    let content = create_help_content(&theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Keybindings Help (? to close)")
        .border_style(stylesheet::border_focused(&theme))
        .style(stylesheet::layer_elevated(&theme));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn create_help_content(theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(""), create_section_header("NAVIGATION", theme)];

    for binding in bindings::bindings_for(BindingScope::Navigation) {
        lines.push(create_help_line(
            &binding.notation.join("/"),
            binding.description,
            theme,
        ));
    }

    lines.push(Line::from(""));
    lines.push(create_section_header("LEADER MENU (Space + key)", theme));
    for submenu in SUBMENUS {
        for binding in bindings::bindings_for(BindingScope::LeaderSubmenu(submenu.key)) {
            if binding.event == UiEvent::DeactivateLeader {
                continue;
            }
            lines.push(create_help_line(
                &format!("Space {} {}", submenu.key, binding.notation.join("/")),
                binding.description,
                theme,
            ));
        }
    }

    lines.push(Line::from(""));
    lines.push(create_section_header("TEXT INPUT (note editing)", theme));
    for binding in bindings::bindings_for(BindingScope::Input) {
        lines.push(create_help_line(
            &binding.notation.join("/"),
            binding.description,
            theme,
        ));
    }
    lines.push(create_help_line(
        INPUT_CATCH_ALL.keys_label,
        INPUT_CATCH_ALL.description,
        theme,
    ));
    lines.push(Line::from(""));

    lines
}

fn create_section_header(text: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  {text}"),
        stylesheet::keybind_key(theme).add_modifier(Modifier::UNDERLINED),
    )])
}

fn create_help_line(keys: &str, description: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(format!("{keys:<20}"), stylesheet::warning(theme)),
        Span::styled(description.to_string(), stylesheet::body(theme)),
    ])
}

fn centered_popup(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
