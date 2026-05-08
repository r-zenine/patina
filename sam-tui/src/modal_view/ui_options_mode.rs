use ratatui::{
    layout::Alignment,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::fmt::Write;

use super::state::OptionsState;
use super::theme::UITheme;

pub(super) struct UIOptionsMode<'a> {
    theme: &'a UITheme,
}

impl<'a> UIOptionsMode<'a> {
    pub(super) const fn new(theme: &'a UITheme) -> Self {
        UIOptionsMode { theme }
    }

    pub(super) fn draw(&self, f: &mut Frame, options: &'a OptionsState) {
        let options_widget = self.options_widget(options);
        f.render_widget(options_widget, f.area())
    }

    fn block(&self, title: &'static str) -> Block<'static> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.borders))
            .border_type(ratatui::widgets::BorderType::Rounded)
    }

    fn options_widget(&self, options: &OptionsState) -> Paragraph<'_> {
        let mut text = String::new();
        for opt in &options.options {
            let toggle = if opt.active { "⌘" } else { " " };
            writeln!(text, "➺ {} ({}) : {}", toggle, opt.key, opt.text)
                .expect("unexpectedly unable to write into a string, please open an issue with the associated stack trace.");
        }
        Paragraph::new(text)
            .block(self.block("Options"))
            .style(self.theme.style())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }
}
