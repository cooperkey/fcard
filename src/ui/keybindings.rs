use crate::ui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(25)])
        .split(area);

    if app.mode == AppMode::Command {
        let cmd_line = Line::from(vec![
            Span::styled(&app.command_buffer, app.theme.accent_style()),
            Span::styled("█", app.theme.title_style()),
        ]);
        let para = Paragraph::new(cmd_line).style(app.theme.status_style());
        frame.render_widget(para, area);
        return;
    }
    if app.mode == AppMode::Search {
        let search_line = Line::from(vec![
            Span::styled("/", app.theme.accent_style()),
            Span::styled(&app.search_buffer, app.theme.accent_style()),
            Span::styled("█", app.theme.title_style()),
        ]);
        let para = Paragraph::new(search_line).style(app.theme.status_style());
        frame.render_widget(para, area);
        return;
    }

    let left_para = if let Some(msg) = &app.status_message {
        Paragraph::new(Line::from(Span::styled(
            format!(" {}", msg),
            app.theme.accent_style(),
        )))
        .style(app.theme.status_style())
    } else {
        let spans = match app.mode {
            AppMode::Browse => vec![
                Span::styled(" ↑↓/jk", app.theme.key_style()),
                Span::styled(": select  ", app.theme.muted_style()),
                Span::styled("gg/G", app.theme.key_style()),
                Span::styled(": top/bot  ", app.theme.muted_style()),
                Span::styled("^d/^u", app.theme.key_style()),
                Span::styled(": scroll  ", app.theme.muted_style()),
                Span::styled("/", app.theme.key_style()),
                Span::styled(": search  ", app.theme.muted_style()),
                Span::styled("n/N", app.theme.key_style()),
                Span::styled(": match  ", app.theme.muted_style()),
                Span::styled("tab", app.theme.key_style()),
                Span::styled(": pane  ", app.theme.muted_style()),
                Span::styled("enter", app.theme.key_style()),
                Span::styled(": flashcards  ", app.theme.muted_style()),
                Span::styled("q", app.theme.key_style()),
                Span::styled(": quit  ", app.theme.muted_style()),
            ],
            AppMode::Flashcard => vec![
                Span::styled("tab", app.theme.key_style()),
                Span::styled(": pane  ", app.theme.muted_style()),
                Span::styled(" ↑↓/jk", app.theme.key_style()),
                Span::styled(": card  ", app.theme.muted_style()),
                Span::styled("space", app.theme.key_style()),
                Span::styled(": flip  ", app.theme.muted_style()),
                Span::styled("1-4", app.theme.key_style()),
                Span::styled(": rate  ", app.theme.muted_style()),
                Span::styled("s", app.theme.key_style()),
                Span::styled(": shuffle  ", app.theme.muted_style()),
                Span::styled("esc/q", app.theme.key_style()),
                Span::styled(": back  ", app.theme.muted_style()),
            ],
            AppMode::Learn if app.learn_awaiting_next => vec![
                Span::styled(" enter/space", app.theme.key_style()),
                Span::styled(": next question  ", app.theme.muted_style()),
                Span::styled("q", app.theme.key_style()),
                Span::styled(": quit  ", app.theme.muted_style()),
            ],
            AppMode::Learn => vec![
                Span::styled("tab", app.theme.key_style()),
                Span::styled(": pane  ", app.theme.muted_style()),
                Span::styled("^w/alt+bs", app.theme.key_style()),
                Span::styled(": del word  ", app.theme.muted_style()),
                Span::styled("enter", app.theme.key_style()),
                Span::styled(": newline  ", app.theme.muted_style()),
                Span::styled("^enter/^s", app.theme.key_style()),
                Span::styled(": submit  ", app.theme.muted_style()),
                Span::styled("esc", app.theme.key_style()),
                Span::styled(": normal/quit  ", app.theme.muted_style()),
            ],
            AppMode::SpacedRep => vec![
                Span::styled(" space", app.theme.key_style()),
                Span::styled(": flip  ", app.theme.muted_style()),
                Span::styled("1-4", app.theme.key_style()),
                Span::styled(": rate  ", app.theme.muted_style()),
                Span::styled("esc/q", app.theme.key_style()),
                Span::styled(": back  ", app.theme.muted_style()),
            ],
            AppMode::ThemePicker => vec![
                Span::styled(" ↑↓", app.theme.key_style()),
                Span::styled(": navigate  ", app.theme.muted_style()),
                Span::styled("enter", app.theme.key_style()),
                Span::styled(": apply  ", app.theme.muted_style()),
                Span::styled("esc", app.theme.key_style()),
                Span::styled(": cancel  ", app.theme.muted_style()),
            ],
            AppMode::AddCard => vec![
                Span::styled(" tab", app.theme.key_style()),
                Span::styled(": switch field  ", app.theme.muted_style()),
                Span::styled("enter/^s", app.theme.key_style()),
                Span::styled(": save  ", app.theme.muted_style()),
                Span::styled("esc", app.theme.key_style()),
                Span::styled(": cancel  ", app.theme.muted_style()),
            ],
            AppMode::Command | AppMode::Search => vec![],
        };
        Paragraph::new(Line::from(spans)).style(app.theme.bg_style())
    };

    frame.render_widget(left_para, chunks[0]);
    let theme_line = Line::from(vec![
        Span::styled("theme: ", app.theme.muted_style()),
        Span::styled(app.theme.name.display_name(), app.theme.accent_style()),
    ]);
    let right_para = Paragraph::new(theme_line)
        .alignment(ratatui::layout::Alignment::Right)
        .style(app.theme.bg_style());
    frame.render_widget(right_para, chunks[1]);
}
