use crate::ui::app::{AddCardFocus, App};
use crate::ui::theme_picker::centered_rect;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(64, 14, area);

    frame.render_widget(Clear, popup_area);
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title(" add new card ")
        .title_style(app.theme.title_style())
        .border_style(app.theme.border_style(true))
        .style(app.theme.bg_style());

    let inner = main_block.inner(popup_area);
    frame.render_widget(main_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);
    let is_front = app.add_card_focus == AddCardFocus::Front;
    let front_border = app.theme.border_style(is_front);
    let front_title = if is_front {
        " front (question) [editing] "
    } else {
        " front (question) "
    };
    let front_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_front {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(front_border)
        .title(front_title)
        .title_style(if is_front {
            app.theme.accent_style()
        } else {
            app.theme.muted_style()
        });

    let mut front_text = app.add_card_front.clone();
    if is_front {
        front_text.push('░');
    }
    let front_para = Paragraph::new(front_text)
        .block(front_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(front_para, chunks[0]);

    let is_back = app.add_card_focus == AddCardFocus::Back;
    let back_border = app.theme.border_style(is_back);
    let back_title = if is_back {
        " back (answer) [editing] "
    } else {
        " back (answer) "
    };
    let back_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_back {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(back_border)
        .title(back_title)
        .title_style(if is_back {
            app.theme.accent_style()
        } else {
            app.theme.muted_style()
        });

    let mut back_text = app.add_card_back.clone();
    if is_back {
        back_text.push('░');
    }
    let back_para = Paragraph::new(back_text)
        .block(back_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(back_para, chunks[1]);

    let footer_line = Line::from(vec![
        Span::styled("tab", app.theme.key_style()),
        Span::styled(": switch field ", app.theme.muted_style()),
        Span::styled("enter/ctrl+s", app.theme.key_style()),
        Span::styled(": save card  ", app.theme.muted_style()),
        Span::styled("esc", app.theme.key_style()),
        Span::styled(": cancel", app.theme.muted_style()),
    ]);
    let footer_para = Paragraph::new(footer_line);
    frame.render_widget(footer_para, chunks[2]);
}
