use ratatui::layout::Direction;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use ratatui::layout::{Alignment, Constraint};
use ratatui::layout::{Layout, Rect};

use ratatui::Frame;

use super::state::Value;
use super::state::ViewState;
use super::theme::UITheme;

pub(super) struct UIInsertMode<'a> {
    filter_chunk: Rect,
    preview_chunk: Rect,
    list_chunk: Rect,
    theme: &'a UITheme,
}

impl<'a> UIInsertMode<'a> {
    pub(super) fn new(area: Rect, theme: &'a UITheme) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let chunk_list_input = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(92), Constraint::Min(8)])
            .split(chunks[0]);

        Self {
            filter_chunk: chunk_list_input[1],
            preview_chunk: chunks[1],
            list_chunk: chunk_list_input[0],
            theme,
        }
    }

    fn list_widget(&self, items: Vec<ListItem<'a>>) -> List<'a> {
        List::new(items)
            .block(self.block("Choices"))
            .style(self.theme.style())
            .highlight_style(self.theme.highlight_style())
            .highlight_symbol("➺ ")
    }

    fn filter_widget(&self, filter_query: &'a str) -> Paragraph<'a> {
        Paragraph::new(filter_query)
            .block(self.block("Filter"))
            .style(self.theme.style())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }

    fn preview_widget(&self, preview: &'a str) -> Paragraph<'a> {
        Paragraph::new(preview)
            .block(self.block("Preview"))
            .style(self.theme.style())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }

    fn block(&self, title: &'static str) -> Block<'static> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.borders))
            .border_type(ratatui::widgets::BorderType::Rounded)
    }

    pub(super) fn draw(
        &self,
        f: &mut Frame,
        mut list_items: ListItems<'a>,
        filter: &'a str,
        preview: &'a str,
    ) {
        let list_widget = self.list_widget(list_items.items);
        let filter_widget = self.filter_widget(filter);
        let preview_widget = self.preview_widget(preview);
        f.render_stateful_widget(list_widget, self.list_chunk, &mut list_items.state);
        f.render_widget(filter_widget, self.filter_chunk);
        f.render_widget(preview_widget, self.preview_chunk);
    }
}

pub(super) struct ListItems<'a> {
    items: Vec<ListItem<'a>>,
    state: ListState,
}

impl<'a, V: Value> From<&'a ViewState<V>> for ListItems<'a> {
    fn from(state: &'a ViewState<V>) -> Self {
        let items = state
            .list
            .displayed_values()
            .iter()
            .map(|e| {
                if e.0 {
                    ListItem::new(format!("❄ {}", e.1.text()))
                } else {
                    ListItem::new(format!("  {}", e.1.text()))
                }
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(state.list.highlighted_line);

        ListItems {
            items,
            state: list_state,
        }
    }
}
