use crate::models::theme::ThemeName;
use crate::ui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
};

pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let themes = ThemeName::ALL;
    let height = (themes.len() as u16) + 4;
    let popup_area = centered_rect(40, height, area);

    frame.render_widget(Clear, popup_area);
    let items: Vec<ListItem> = themes
        .iter()
        .map(|theme| {
            let is_active = theme == &app.theme.name;
            let prefix = if is_active { "● " } else { "  " };
            let style = if is_active {
                app.theme.accent_style()
            } else {
                app.theme.text_style()
            };
            let line = Line::from(vec![
                Span::styled(prefix, app.theme.accent_style()),
                Span::styled(theme.display_name(), style),
            ]);
            ListItem::new(line)
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title(" select theme ")
                .title_style(app.theme.title_style())
                .border_style(app.theme.border_style(true))
                .style(app.theme.popup_style()),
        )
        .highlight_style(app.theme.highlight_style());
    let mut state = ListState::default();
    state.select(Some(app.theme_picker_selected));
    frame.render_stateful_widget(list, popup_area, &mut state);
}
