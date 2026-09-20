use crate::ui::app::{App, FocusedPane};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focused_pane == FocusedPane::MainView;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .title(" preview ")
        .title_style(app.theme.title_style())
        .border_style(app.theme.border_style(focused))
        .style(app.theme.bg_style());
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if app.decks.is_empty() {
        let placeholder = Paragraph::new("no decks loaded.\n\nuse fcard init <path> to create one")
            .style(app.theme.muted_style())
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner_area);
        return;
    }

    let deck = &app.decks[app.selected_deck];
    if deck.cards.is_empty() {
        let placeholder = Paragraph::new("deck is empty.\n\nEdit the file or add cards")
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
            Constraint::Length(2),
        ])
        .split(inner_area);
    let header_text = if app.card_flipped {
        Span::styled("back", app.theme.accent_style())
    } else {
        Span::styled("front", app.theme.accent_style())
    };
    let progress_text = Span::styled(
        format!(" (card {}/{})", card_idx + 1, deck.cards.len()),
        app.theme.muted_style(),
    );
    let header =
        Paragraph::new(Line::from(vec![header_text, progress_text])).alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);
    let content_text = if app.card_flipped {
        card.back.as_str()
    } else {
        card.front.as_str()
    };
    let raw_lines: Vec<&str> = content_text.lines().collect();
    let vertical_space = chunks[1].height as usize;
    let lines_count = raw_lines.len();
    let pad_top = if vertical_space > lines_count {
        (vertical_space - lines_count) / 2
    } else {
        0
    };
    let mut content_lines = Vec::new();
    for _ in 0..pad_top {
        content_lines.push(Line::from(""));
    }
    for l in &raw_lines {
        let spans = crate::ui::card_content::highlight_spans(
            l,
            &app.search_query,
            app.theme.text_style(),
            app.theme.warning_style(),
        );
        content_lines.push(Line::from(spans));
    }

    let content = Paragraph::new(content_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(content, chunks[1]);

    let mut footer_spans = Vec::new();
    if !card.tags.is_empty() {
        footer_spans.push(Span::styled("tags: ", app.theme.accent_style()));
        let tags_str = card.tags.join(", ");
        let highlighted = crate::ui::card_content::highlight_spans(
            &tags_str,
            &app.search_query,
            app.theme.muted_style(),
            app.theme.warning_style(),
        );
        footer_spans.extend(highlighted);
    }

    if app.card_flipped && !card.hints.is_empty() {
        if !footer_spans.is_empty() {
            footer_spans.push(Span::styled("  |  ", app.theme.muted_style()));
        }
        footer_spans.push(Span::styled("hint: ", app.theme.warning_style()));
        let hints_str = card.hints.join(", ");
        let highlighted = crate::ui::card_content::highlight_spans(
            &hints_str,
            &app.search_query,
            app.theme.muted_style(),
            app.theme.warning_style(),
        );
        footer_spans.extend(highlighted);
    }

    let footer = Paragraph::new(Line::from(footer_spans))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[2]);
}
