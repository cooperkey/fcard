use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Panes {
    pub deck_list: Rect,
    pub card_list: Rect,
    pub main_view: Rect,
    pub stats: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(area: Rect) -> Panes {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let main_area = main_layout[0];
    let status_bar = main_layout[1];
    let horiz_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main_area);
    let left_area = horiz_layout[0];
    let main_view = horiz_layout[1];
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(45),
        ])
        .split(left_area);

    let deck_list = left_layout[0];
    let stats = left_layout[1];
    let card_list = left_layout[2];

    Panes {
        deck_list,
        card_list,
        main_view,
        stats,
        status_bar,
    }
}
