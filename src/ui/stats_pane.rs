use crate::storage::stats_store::StatsStore;
use crate::ui::app::{App, FocusedPane};
use crate::ui::event::current_epoch;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App, stats: &StatsStore) {
    let focused = app.focused_pane == FocusedPane::Stats;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .title(" stats ")
        .title_style(app.theme.title_style())
        .border_style(app.theme.border_style(focused))
        .style(app.theme.bg_style());
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    let deck = match app.current_deck() {
        Some(d) => d,
        None => {
            let placeholder = Paragraph::new(" no active deck selected")
                .style(app.theme.muted_style())
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(placeholder, inner_area);
            return;
        }
    };

    if deck.cards.is_empty() {
        let placeholder = Paragraph::new("deck is empty")
            .style(app.theme.muted_style())
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(placeholder, inner_area);
        return;
    }

    let now = current_epoch();
    let due_ids = stats.due_ids(now);
    let mut total_correct = 0;
    let mut total_incorrect = 0;
    let mut reviewed_count = 0;
    let mut sum_ef = 0.0;
    let mut due_count = 0;

    for card in &deck.cards {
        let card_state = stats.get_or_default(&card.id);
        if card_state.repetitions > 0 {
            reviewed_count += 1;
            sum_ef += card_state.ease_factor;
        }
        total_correct += card_state.total_correct;
        total_incorrect += card_state.total_incorrect;
        if due_ids.contains(&card.id) {
            due_count += 1;
        }
    }

    let avg_ef = if reviewed_count > 0 {
        sum_ef / (reviewed_count as f32)
    } else {
        2.5
    };
    let total_reviews = total_correct + total_incorrect;
    let mastery_pct = if total_reviews > 0 {
        (total_correct as f64 / total_reviews as f64) * 100.0
    } else {
        0.0
    };
    let total_cards = deck.cards.len();
    let stats_line = vec![
        Line::from(vec![
            Span::styled("mastery:      ", app.theme.accent_style()),
            Span::styled(
                format!(
                    "{} of {} cards ({:.0}%)",
                    reviewed_count, total_cards, mastery_pct
                ),
                app.theme.success_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("total cards:  ", app.theme.accent_style()),
            Span::styled(format!("{}", total_cards), app.theme.text_style()),
            Span::styled("  reviewed: ", app.theme.accent_style()),
            Span::styled(format!("{}", reviewed_count), app.theme.text_style()),
        ]),
        Line::from(vec![
            Span::styled("due today:    ", app.theme.accent_style()),
            Span::styled(
                format!("{}", due_count),
                if due_count > 0 {
                    app.theme.warning_style()
                } else {
                    app.theme.success_style()
                },
            ),
            Span::styled("  avg ease: ", app.theme.accent_style()),
            Span::styled(format!("{:.2}", avg_ef), app.theme.text_style()),
        ]),
        Line::from(vec![
            Span::styled("total trials: ", app.theme.accent_style()),
            Span::styled(
                format!(
                    "{} ({} correct, {} incorrect)",
                    total_reviews, total_correct, total_incorrect
                ),
                app.theme.text_style(),
            ),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_line)
        .style(app.theme.text_style())
        .wrap(Wrap { trim: false });
    frame.render_widget(stats_paragraph, inner_area);
}
