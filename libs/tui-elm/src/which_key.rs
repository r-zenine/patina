//! Which-key style overlay for the leader menu.
//!
//! Renders the keys available in the active leader scope, with a timeout
//! indicator, from the keybinding registry — the overlay cannot drift from
//! dispatch. Rows with [`BindingRole::DismissLeader`] become the Esc footer
//! ("cancel" at the root, "back" in a submenu) instead of menu items.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};
use tui_design::{Theme, stylesheet};

use crate::leader::LeaderState;
use crate::registry::{BindingRole, BindingScope, Registry};

/// Render the which-key overlay when the leader is active (bottom panel).
pub fn render<E>(f: &mut Frame, registry: &Registry<E>, leader: &LeaderState, theme: &Theme) {
    let Some(scope) = leader.scope() else {
        return;
    };

    let area = bottom_panel(f.area());

    f.render_widget(Clear, area);

    let content = menu_lines(registry, scope, theme);
    let title = menu_title(registry, scope);

    let timeout_text = if let Some(remaining) = leader.timeout_remaining() {
        format!(" [{}s] ", remaining.as_secs())
    } else {
        " ".to_string()
    };

    let block = Block::default()
        .title(format!("{title}{timeout_text}"))
        .title_style(stylesheet::border_focused(theme))
        .style(stylesheet::layer_elevated(theme));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn menu_title<E>(registry: &Registry<E>, scope: BindingScope) -> String {
    match scope {
        BindingScope::LeaderSubmenu(c) => match registry.submenus.iter().find(|s| s.key == c) {
            Some(submenu) => format!(" {} ({} + {c})", submenu.title, registry.leader_label),
            None => " Leader Menu".to_string(),
        },
        _ => format!(" Leader Menu ({})", registry.leader_label),
    }
}

fn menu_lines<E>(registry: &Registry<E>, scope: BindingScope, theme: &Theme) -> Vec<Line<'static>> {
    let mut items: Vec<(String, &'static str)> = Vec::new();
    let mut esc_label = "cancel";

    for binding in registry.bindings_for(scope) {
        if binding.role == BindingRole::DismissLeader {
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
