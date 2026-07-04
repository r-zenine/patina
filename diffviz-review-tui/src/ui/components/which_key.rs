//! Which-key style overlay component for leader key menu
//!
//! Displays available commands when the leader key is active, with timeout
//! indicator. Content is rendered from the keybinding registry
//! (`events::bindings`) — the overlay cannot drift from dispatch.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};
use tui_design::{Theme, stylesheet};

use crate::events::UiEvent;
use crate::events::bindings::{self, BindingScope, SUBMENUS};
use crate::state::UiState;

/// Render the which-key overlay when leader is active (Spacemacs-style bottom panel)
pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.leader_active {
        return;
    }

    let theme = Theme::mocha();
    let area = bottom_panel(f.area());

    f.render_widget(Clear, area);

    let scope = match ui_state.leader_submenu {
        None => BindingScope::LeaderRoot,
        Some(c) => BindingScope::LeaderSubmenu(c),
    };

    let content = menu_lines(scope, &theme);
    let title = menu_title(scope);

    let timeout_text = if let Some(remaining) = ui_state.leader_timeout_remaining() {
        format!(" [{}s] ", remaining.as_secs())
    } else {
        " ".to_string()
    };

    let block = Block::default()
        .title(format!("{title}{timeout_text}"))
        .title_style(stylesheet::border_focused(&theme))
        .style(stylesheet::layer_elevated(&theme));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn menu_title(scope: BindingScope) -> String {
    match scope {
        BindingScope::LeaderSubmenu(c) => match SUBMENUS.iter().find(|s| s.key == c) {
            Some(submenu) => format!(" {} (Space + {c})", submenu.title),
            None => " Leader Menu".to_string(),
        },
        _ => " Leader Menu (Space)".to_string(),
    }
}

fn menu_lines(scope: BindingScope, theme: &Theme) -> Vec<Line<'static>> {
    let mut items: Vec<(String, &'static str)> = Vec::new();
    let mut esc_label = "cancel";

    for binding in bindings::bindings_for(scope) {
        if binding.event == UiEvent::DeactivateLeader {
            esc_label = if scope == BindingScope::LeaderRoot {
                "cancel"
            } else {
                "back"
            };
            continue;
        }
        items.push((binding.notation.join("/"), binding.description));
    }

    vec![
        Line::from(""),
        create_compact_line(items, theme),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", stylesheet::keybind_key(theme)),
            Span::styled(format!(" {esc_label}"), stylesheet::muted(theme)),
        ]),
    ]
}

fn create_compact_line(items: Vec<(String, &str)>, theme: &Theme) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];

    for (i, (key, description)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("    "));
        }

        spans.push(Span::styled(
            format!("[{key}]"),
            stylesheet::keybind_key(theme).add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            description.to_string(),
            stylesheet::body(theme),
        ));
    }

    Line::from(spans)
}

fn bottom_panel(r: Rect) -> Rect {
    let height = 10;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(height)])
        .split(r);

    layout[1]
}
