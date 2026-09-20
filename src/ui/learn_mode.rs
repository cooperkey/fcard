use std::vec;

use crate::engine::matcher;
use crate::ui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" learn mode ")
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

    if app.learn_remaining.is_empty() && !app.learn_awaiting_next {
        let completion_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner_area);
        let congrats = Paragraph::new("all cards mastered")
            .alignment(Alignment::Center)
            .style(app.theme.muted_style());
        let hint = Paragraph::new("press esc or q to return to deck list")
            .alignment(Alignment::Center)
            .style(app.theme.muted_style());

        frame.render_widget(congrats, completion_layout[1]);
        frame.render_widget(hint, completion_layout[2]);
        return;
    }

    if app.learn_awaiting_next {
        draw_feedback_screen(frame, inner_area, app);
        return;
    }

    let current_idx = app.learn_remaining[0];
    let card = match deck.cards.get(current_idx) {
        Some(c) => c,
        None => return,
    };
    let total = deck.cards.len();
    let remaining = app.learn_remaining.len();
    let completed = total.saturating_sub(remaining);
    let is_multiline = card.back.contains('\n');
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Percentage(30),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(inner_area);

    let ratio = if total > 0 {
        completed as f64 / total as f64
    } else {
        0.0
    };
    let label = format!("progress {}/{} learned", completed, total);
    crate::ui::theme::render_gradient_gauge(frame, chunks[0], &label, ratio, &app.theme);
    let question_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true))
        .title(" question ")
        .title_style(app.theme.title_style());
    let q_inner = question_block.inner(chunks[1]);
    frame.render_widget(question_block, chunks[1]);
    let q_text = card.front.as_str();
    let q_lines_count = q_text.lines().count();
    let q_pad = if q_inner.height as usize > q_lines_count {
        (q_inner.height as usize - q_lines_count) / 2
    } else {
        0
    };
    let mut q_content = Vec::new();
    for _ in 0..q_pad {
        q_content.push(Line::from(""));
    }
    for line in q_text.lines() {
        q_content.push(Line::from(Span::styled(line, app.theme.text_style())));
    }
    frame.render_widget(
        Paragraph::new(q_content)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        q_inner,
    );

    let (line_idx, col_idx) = app.cursor_line_col();
    let mode_badge = match app.input_edit_mode {
        crate::ui::app::InputEditMode::Insert => "--INSERT--",
        crate::ui::app::InputEditMode::Normal => "--NORMAL--",
    };
    let input_title = format!(
        " your answer {} [Ln {}, Col{}]",
        mode_badge,
        line_idx + 1,
        col_idx + 1
    );
    let title_style = match app.input_edit_mode {
        crate::ui::app::InputEditMode::Insert => app.theme.accent_style(),
        crate::ui::app::InputEditMode::Normal => app.theme.warning_style(),
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true))
        .title(input_title)
        .title_style(title_style);

    let input_inner = input_block.inner(chunks[2]);
    frame.render_widget(input_block, chunks[2]);

    let text = &app.input_buffer;
    let cursor_pos = app.input_cursor.min(text.len());
    let safe_cursor = text
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
        .find(|&i| i >= cursor_pos)
        .unwrap_or(text.len());
    let before_cursor = &text[..safe_cursor];
    let after_cursor = &text[safe_cursor..];
    let full_rendered = format!("{before_cursor}█{after_cursor}");
    let raw_lines: Vec<&str> = full_rendered.split('\n').collect();
    let mut input_lines: Vec<Line> = Vec::new();
    let mut cursor_line_idx = 0;
    for (line_idx, line) in raw_lines.iter().enumerate() {
        if line.contains('█') {
            cursor_line_idx = line_idx;
        }
        let mut spans = Vec::new();
        let parts: Vec<&str> = line.split('█').collect();
        for (p_idx, part) in parts.iter().enumerate() {
            spans.push(Span::styled(part.to_string(), app.theme.text_style()));
            if p_idx < parts.len() - 1 {
                spans.push(Span::styled("█", app.theme.accent_style()));
            }
        }
        input_lines.push(Line::from(spans));
    }

    if input_lines.is_empty() {
        input_lines.push(Line::from(vec![
            Span::styled(" > ", app.theme.accent_style()),
            Span::styled("█", app.theme.accent_style()),
        ]));
    }

    let inner_height = input_inner.height as usize;
    let scroll_y = if inner_height > 0 && cursor_line_idx >= inner_height {
        (cursor_line_idx + 1 - inner_height) as u16
    } else {
        0
    };

    frame.render_widget(
        Paragraph::new(input_lines)
            .scroll((scroll_y, 0))
            .wrap(Wrap { trim: false }),
        input_inner,
    );

    let mut footer_spans = Vec::new();
    if is_multiline {
        footer_spans.push(Span::styled("enter", app.theme.key_style()));
        footer_spans.push(Span::styled(": newlne", app.theme.muted_style()));
        footer_spans.push(Span::styled("^enter/^s", app.theme.key_style()));
        footer_spans.push(Span::styled(": submit", app.theme.muted_style()));
    } else {
        footer_spans.push(Span::styled("enter", app.theme.key_style()));
        footer_spans.push(Span::styled(": submit", app.theme.muted_style()));
    }

    footer_spans.push(Span::styled("esc", app.theme.key_style()));
    footer_spans.push(Span::styled(": skip", app.theme.muted_style()));
    footer_spans.push(Span::styled("q", app.theme.key_style()));
    footer_spans.push(Span::styled(": quit", app.theme.muted_style()));

    let footer = Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}

fn draw_feedback_screen(frame: &mut Frame, area: Rect, app: &App) {
    let score = app.learn_last_score.unwrap_or(0.0);
    let f_msg = matcher::feedback(score);
    let is_perfect = score >= 0.95;
    let is_pass = score >= matcher::MATCH_THRESHOLD;
    let (header_title, header_style) = if is_perfect {
        (
            format!(" [passed] {}! (100% match) ", f_msg),
            app.theme.success_style(),
        )
    } else if is_pass {
        (
            format!(" [close] {}! ({:.0}% match)", f_msg, score * 100.0),
            app.theme.warning_style(),
        )
    } else {
        (
            format!(" [failed] {}! ({:.0}% match)", f_msg, score * 100.0),
            app.theme.error_style(),
        )
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(header_style)
        .title(header_title)
        .title_style(header_style);
    let header_para = Paragraph::new(Line::from(Span::styled(
        if is_pass {
            "Passed. Proceeding to next card"
        } else {
            "Review the solution before moving on"
        },
        app.theme.text_style(),
    )))
    .alignment(Alignment::Center)
    .block(header_block);

    frame.render_widget(header_para, chunks[0]);

    let gauge_label = format!("match accuracy: {:.0}%", score * 100.0);
    crate::ui::theme::render_gradient_gauge(
        frame,
        chunks[1],
        &gauge_label,
        score.clamp(0.0, 1.0),
        &app.theme,
    );

    let compare_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true))
        .title(" answer comparison ")
        .title_style(app.theme.title_style());
    let c_inner = compare_block.inner(chunks[2]);
    frame.render_widget(compare_block, chunks[2]);
    let expected = app.learn_last_expected.as_deref().unwrap_or("");
    let user_ans = app.learn_last_answer.as_deref().unwrap_or("");
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "expected answer:",
        app.theme.title_style(),
    )));
    for line in expected.lines() {
        lines.push(Line::from(vec![
            Span::styled("  • ", app.theme.accent_style()),
            Span::styled(line, app.theme.text_style()),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "your answer diff: ",
        app.theme.title_style(),
    )]));

    let diff_lines = matcher::diff_user_answer_multiline(
        user_ans,
        expected,
        app.theme.text_style(),
        app.theme.success_style(),
        app.theme.error_style(),
    );
    lines.extend(diff_lines);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), c_inner);

    let continue_spans = vec![
        Span::styled(" [press ", app.theme.muted_style()),
        Span::styled("ENTER", app.theme.accent_style()),
        Span::styled(" or o", app.theme.muted_style()),
        Span::styled("SPACE", app.theme.accent_style()),
        Span::styled(" to continue to next question ]", app.theme.muted_style()),
    ];
    let footer = Paragraph::new(Line::from(continue_spans)).alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}
