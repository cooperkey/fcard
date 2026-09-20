use crate::storage::stats_store::StatsStore;
use crate::ui::app::{App, FocusedPane};
use crate::ui::event::current_epoch;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App, stats: &StatsStore) {
    let focused = app.focused_pane == FocusedPane::DeckList;
    let deck_count = app.decks.len();
    let items: Vec<ListItem> = (0..deck_count)
        .map(|idx| {
            let deck = &app.decks[idx];
            let due_count = deck
                .cards
                .iter()
                .filter(|c| stats.due_ids(current_epoch()).contains(&c.id))
                .count();
            let title = &deck.title;
            let card_count = deck.cards.len();
            let due_span = if due_count > 0 {
                Span::styled(format!(" {} due", due_count), app.theme.warning_style())
            } else {
                Span::styled("".to_string(), app.theme.muted_style())
            };

            let line = Line::from(vec![
                Span::styled(format!(" {}", title), app.theme.text_style()),
                Span::styled(format!(" ({})", card_count), app.theme.muted_style()),
                due_span,
            ]);
            ListItem::new(line)
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(if focused {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .title(" decks ")
                .title_style(app.theme.title_style())
                .border_style(app.theme.border_style(focused))
                .style(app.theme.bg_style()),
        )
        .highlight_style(app.theme.highlight_style());
    let mut state = ListState::default();
    if !app.decks.is_empty() {
        state.select(Some(app.selected_deck));
    }
    frame.render_stateful_widget(list, area, &mut state);
}
