use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::ui::app::App;
const MAX_COLS: usize = 6;

pub fn highlight_spans(
    text: &str,
    query: &str,
    normal_style: ratatui::style::Style,
    highlight_style: ratatui::style::Style,
) -> Vec<Span<'static>> {
    let query_trimmed = query.trim();
    if query_trimmed.is_empty() {
        return vec![Span::styled(text.to_string(), normal_style)];
    }

    let mut spans = Vec::new();
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let query_bytes_len = query_lower.len();
    let mut last_idx = 0;

    while last_idx < text.len() {
        if let Some(match_offset) = text_lower[last_idx..].find(&query_lower) {
            let start = last_idx + match_offset;
            let end = start + query_bytes_len;

            if text.is_char_boundary(start) && text.is_char_boundary(end) {
                if start > last_idx {
                    spans.push(Span::styled(
                        text[last_idx..start].to_string(),
                        normal_style,
                    ));
                }
                spans.push(Span::styled(text[start..end].to_string(), highlight_style));
                last_idx = end;
            } else {
                let next_char_len = text[last_idx..].chars().next().map_or(1, |c| c.len_utf8());
                last_idx += next_char_len;
            }
        } else {
            break;
        }
    }

    if last_idx < text.len() {
        spans.push(Span::styled(text[last_idx..].to_string(), normal_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), normal_style));
    }
    spans
}

pub fn render_front(frame: &mut Frame, area: Rect, text: &str, app: &App) {
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len();
    let pad = if area.height as usize > n {
        (area.height as usize - n) / 2
    } else {
        0
    };
    let mut content: Vec<Line> = (0..pad).map(|_| Line::from("")).collect();
    for l in &lines {
        let spans = highlight_spans(
            l,
            &app.search_query,
            app.theme.text_style(),
            app.theme.warning_style(),
        );
        content.push(Line::from(spans));
    }
    frame.render_widget(
        Paragraph::new(content)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub fn render_back(frame: &mut Frame, area: Rect, text: &str, scroll_offset: u16, app: &App) {
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len();
    let h = area.height as usize;
    let w = area.width as usize;
    if n == 0 {
        return;
    }

    let mut best_cols = None;
    for ncols in 1..=MAX_COLS {
        let rows_per_col = n.div_ceil(ncols);
        let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let col_w = w / ncols;
        if rows_per_col <= h && (ncols == 1 || col_w >= max_line_len.min(col_w).max(4)) {
            best_cols = Some(ncols);
            break;
        }
    }

    match best_cols {
        None => {
            let content: Vec<Line> = lines
                .iter()
                .map(|l| {
                    Line::from(highlight_spans(
                        l,
                        &app.search_query,
                        app.theme.text_style(),
                        app.theme.warning_style(),
                    ))
                })
                .collect();
            frame.render_widget(
                Paragraph::new(content)
                    .scroll((scroll_offset, 0))
                    .wrap(Wrap { trim: false }),
                area,
            );
        }
        Some(1) if n > h => {
            let content: Vec<Line> = lines
                .iter()
                .map(|l| {
                    Line::from(highlight_spans(
                        l,
                        &app.search_query,
                        app.theme.text_style(),
                        app.theme.warning_style(),
                    ))
                })
                .collect();
            frame.render_widget(
                Paragraph::new(content)
                    .scroll((scroll_offset, 0))
                    .wrap(Wrap { trim: false }),
                area,
            );
        }

        Some(1) => {
            let pad = (h.saturating_sub(n)) / 2;
            let mut content: Vec<Line> = (0..pad).map(|_| Line::from("")).collect();
            for l in &lines {
                let spans = highlight_spans(
                    l,
                    &app.search_query,
                    app.theme.text_style(),
                    app.theme.warning_style(),
                );
                content.push(Line::from(spans));
            }
            frame.render_widget(
                Paragraph::new(content)
                    .alignment(ratatui::layout::Alignment::Center)
                    .wrap(Wrap { trim: false }),
                area,
            );
        }

        Some(ncols) => {
            let rows_per_col = n.div_ceil(ncols);
            let constraints: Vec<Constraint> = (0..ncols)
                .map(|_| Constraint::Ratio(1, ncols as u32))
                .collect();
            let col_areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(area);

            for col_idx in 0..ncols {
                let start = col_idx & rows_per_col;
                if start >= n {
                    break;
                }
                let end = (start + rows_per_col).min(n);
                let col_lines = &lines[start..end];
                let pad = (h.saturating_div(col_lines.len())) / 2;
                let mut content: Vec<Line> = (0..pad).map(|_| Line::from("")).collect();
                for l in col_lines {
                    let spans = highlight_spans(
                        l,
                        &app.search_query,
                        app.theme.text_style(),
                        app.theme.warning_style(),
                    );
                    content.push(Line::from(spans));
                }
                frame.render_widget(
                    Paragraph::new(content).wrap(Wrap { trim: false }),
                    col_areas[col_idx],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Style};

    #[test]
    fn test_highlight_spans() {
        let normal = Style::default().fg(Color::White);
        let highlight = Style::default().fg(Color::Yellow);
        let spans = highlight_spans("hello world", "", normal, highlight);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "hello world");
        assert_eq!(spans[0].style, normal);

        let spans = highlight_spans("Hello World", "world", normal, highlight);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content, "Hello ");
        assert_eq!(spans[0].style, normal);
        assert_eq!(spans[1].content, "World  ");
        assert_eq!(spans[1].style, highlight);

        let spans = highlight_spans("banana", "a", normal, highlight);
        assert_eq!(spans.len(), 6);
    }
}
