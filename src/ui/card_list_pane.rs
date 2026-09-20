use crate::storage::stats_store::StatsStore;
use crate::ui::app::{App, AppMode, FocusedPane};
use crate::ui::event::current_epoch;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App, stats: &StatsStore) {
    let focused = app.focused_pane == FocusedPane::CardList;
    let epoch = current_epoch();
    let due_ids = stats.due_ids(epoch);

    let deck = match app.current_deck() {
        Some(d) => d,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(if focused {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .title(" Cards ")
                .title_style(app.theme.title_style())
                .border_style(app.theme.border_style(focused))
                .style(app.theme.bg_style());
            frame.render_widget(block, area);
            return;
        }
    };

    let card_count = deck.cards.len();

    let items: Vec<ListItem> = (0..card_count)
        .filter_map(|pos| {
            let real_idx = if app.shuffle && !app.shuffled_order.is_empty() {
                *app.shuffled_order.get(pos)?
            } else {
                pos
            };
            let card = deck.cards.get(real_idx)?;
            let total_chars = card.front.chars().count();
            let snippet: String = if total_chars > 24 {
                format!("{}...", card.front.chars().take(24).collect::<String>())
            } else {
                card.front.clone()
            };
            let is_completed = match app.mode {
                AppMode::Learn => !app.learn_remaining.contains(&real_idx),
                _ => {
                    let card_state = stats.get_or_default(&card.id);
                    card_state.repetitions > 0
                }
            };
            let is_due = due_ids.contains(&card.id);
            let text_style = if is_completed {
                app.theme.success_style()
            } else {
                app.theme.text_style()
            };
            let status_span = if is_completed {
                Span::styled(" [done", app.theme.success_style())
            } else if is_due {
                Span::styled(" [due]", app.theme.warning_style())
            } else {
                Span::styled("", app.theme.muted_style())
            };
            let num_span = Span::styled(format!(" {:2}. ", pos + 1), app.theme.accent_style());
            let mut line_spans = vec![num_span];
            let highlighted_snippet = crate::ui::card_content::highlight_spans(
                &snippet,
                &app.search_query,
                text_style,
                app.theme.warning_style(),
            );
            line_spans.extend(highlighted_snippet);
            line_spans.push(status_span);
            let line = Line::from(line_spans);
            Some(ListItem::new(line))
        })
        .collect();

    let title = if app.shuffle {
        format!(" cards ({}) [shuffled] ", card_count)
    } else {
        format!(" cards ({}) ", card_count)
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(if focused {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                })
                .title(title)
                .title_style(app.theme.title_style())
                .border_style(app.theme.border_style(focused))
                .style(app.theme.bg_style()),
        )
        .highlight_style(app.theme.highlight_style());
    let mut state = ListState::default();

    if !deck.cards.is_empty() {
        let active_pos = if app.mode == AppMode::Learn {
            if let Some(&active_real_idx) = app.learn_remaining.first() {
                if app.shuffle && !app.shuffled_order.is_empty() {
                    app.shuffled_order
                        .iter()
                        .position(|&r| r == active_real_idx)
                        .unwrap_or(app.selected_card)
                } else {
                    active_real_idx
                }
            } else {
                app.selected_card
            }
        } else {
            app.selected_card
        };

        let active_pos = active_pos.min(card_count.saturating_sub(1));
        state.select(Some(active_pos));

        const SCROLLOFF: usize = 3;
        let visible_rows = (area.height as usize).saturating_sub(2).max(1);
        let offset = app.card_list_scroll;
        let new_offset = if active_pos < offset + SCROLLOFF {
            active_pos.saturating_sub(SCROLLOFF)
        } else if active_pos + 1 + SCROLLOFF > offset + visible_rows {
            (active_pos + 1 + SCROLLOFF).saturating_sub(visible_rows)
        } else {
            offset
        };

        app.card_list_scroll = new_offset.min(card_count.saturating_sub(visible_rows));
        *state.offset_mut() = app.card_list_scroll;
    }
    frame.render_stateful_widget(list, area, &mut state);
}
