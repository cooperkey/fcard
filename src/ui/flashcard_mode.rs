use crate::ui::app::App;
use crate::ui::card_content;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" flashcard mode ")
        .title_style(app.theme.title_style())
        .border_style(app.theme.border_style(true))
        .style(app.theme.bg_style());
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    let deck = match app.current_deck() {
        Some(d) => d,
        None => return,
    };

    if deck.cards.is_empty() {
        let placeholder = Paragraph::new("deck is empty")
            .style(app.theme.muted_style())
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner_area);
        return;
    }

    let card_idx = app.current_card_index().unwrap_or(0);
    let card = match deck.cards.get(card_idx) {
        Some(c) => c,
        None => return,
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner_area);
    let progress_line = Line::from(vec![Span::styled(
        format!("card {}/{}", card_idx + 1, deck.cards.len()),
        app.theme.accent_style(),
    )]);
    frame.render_widget(
        Paragraph::new(progress_line).alignment(Alignment::Left),
        chunks[0],
    );

    let cards_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let front_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true))
        .title(" front ")
        .title_style(app.theme.title_style());
    let front_inner = front_block.inner(cards_chunks[0]);
    frame.render_widget(front_block, cards_chunks[0]);
    card_content::render_front(frame, front_inner, &card.front, app);

    let back_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(app.card_flipped))
        .title(" back ")
        .title_style(app.theme.title_style());
    let back_inner = back_block.inner(cards_chunks[1]);
    frame.render_widget(back_block, cards_chunks[1]);

    if app.card_flipped {
        card_content::render_back(frame, back_inner, &card.back, app.back_scroll, app)
    } else {
        let pad = (back_inner.height as usize).saturating_sub(1) / 2;
        let mut b_content: Vec<Line> = (0..pad).map(|_| Line::from("")).collect();
        b_content.push(Line::from(Span::styled(
            "[press space to flip]",
            app.theme.muted_style(),
        )));
        frame.render_widget(
            Paragraph::new(b_content).alignment(Alignment::Center),
            back_inner,
        );
    }

    let footer_line = if app.card_flipped {
        Line::from(vec![
            Span::styled("rate: ", app.theme.accent_style()),
            Span::styled("1", app.theme.key_style()),
            Span::styled(": again", app.theme.muted_style()),
            Span::styled("2", app.theme.key_style()),
            Span::styled(": hard", app.theme.muted_style()),
            Span::styled("3", app.theme.key_style()),
            Span::styled(": good", app.theme.muted_style()),
            Span::styled("4", app.theme.key_style()),
            Span::styled(": easy", app.theme.muted_style()),
            Span::styled("  |  ", app.theme.muted_style()),
            Span::styled("↑/↓", app.theme.key_style()),
            Span::styled(": scroll", app.theme.muted_style()),
            Span::styled("s", app.theme.key_style()),
            Span::styled(": shuffle", app.theme.muted_style()),
            Span::styled("esc/q", app.theme.key_style()),
            Span::styled(": back", app.theme.muted_style()),
        ])
    } else {
        Line::from(vec![
            Span::styled("space", app.theme.key_style()),
            Span::styled(": flip", app.theme.muted_style()),
            Span::styled("s", app.theme.key_style()),
            Span::styled(": shuffle", app.theme.muted_style()),
            Span::styled("esc/q", app.theme.key_style()),
            Span::styled(": back", app.theme.muted_style()),
        ])
    };
    frame.render_widget(
        Paragraph::new(footer_line).alignment(Alignment::Center),
        chunks[2],
    );
}
