//! Help overlay listing every keybinding, rendered from the registry so it
//! cannot drift from dispatch.
//!
//! Sections: navigation rows, leader submenu rows (spelled as full chords,
//! e.g. "Space a f"), then text-input rows plus the input catch-all. Rows
//! with [`BindingRole::DismissLeader`] are omitted — Esc handling is
//! explained by the which-key overlay itself.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_design::{Theme, stylesheet};

use crate::registry::{BindingRole, BindingScope, Registry};

/// App-authored strings for the help overlay.
pub struct HelpText<'a> {
    /// Overlay block title, e.g. "Keybindings Help (? to close)".
    pub title: &'a str,
    /// Heading of the text-input section, e.g. "TEXT INPUT (note editing)".
    pub input_section: &'a str,
}

/// Render the help overlay (centered popup over the whole frame).
pub fn render<E>(f: &mut Frame, registry: &Registry<E>, theme: &Theme, text: &HelpText) {
    let area = centered_popup(f.area(), 80, 80);

    f.render_widget(Clear, area);

    let content = create_help_content(registry, theme, text);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(text.title.to_string())
        .border_style(stylesheet::border_focused(theme))
        .style(stylesheet::layer_elevated(theme));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn create_help_content<E>(
    registry: &Registry<E>,
    theme: &Theme,
    text: &HelpText,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(""), create_section_header("NAVIGATION", theme)];

    for binding in registry.bindings_for(BindingScope::Navigation) {
        lines.push(create_help_line(
            &binding.notation.join("/"),
            binding.description,
            theme,
        ));
    }

    lines.push(Line::from(""));
    lines.push(create_section_header(
        &format!("LEADER MENU ({} + key)", registry.leader_label),
        theme,
    ));
    for submenu in registry.submenus {
        for binding in registry.bindings_for(BindingScope::LeaderSubmenu(submenu.key)) {
            if binding.role == BindingRole::DismissLeader {
                continue;
            }
            lines.push(create_help_line(
                &format!(
                    "{} {} {}",
                    registry.leader_label,
                    submenu.key,
                    binding.notation.join("/")
                ),
                binding.description,
                theme,
            ));
        }
    }

    lines.push(Line::from(""));
    lines.push(create_section_header(text.input_section, theme));
    for binding in registry.bindings_for(BindingScope::Input) {
        lines.push(create_help_line(
            &binding.notation.join("/"),
            binding.description,
            theme,
        ));
    }
    if let Some(catch_all) = registry.input_catch_all {
        lines.push(create_help_line(
            catch_all.keys_label,
            catch_all.description,
            theme,
        ));
    }
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
