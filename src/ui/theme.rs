use crate::models::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

impl Theme {
    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.border_focused)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.border)
        }
    }

    pub fn text_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    pub fn highlight_style(&self) -> Style {
        if self.name == crate::models::theme::ThemeName::Terminal || self.highlight == Color::Reset
        {
            Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.fg)
                .bg(self.highlight)
                .add_modifier(Modifier::BOLD)
        }
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_style(&self) -> Style {
        if self.name == crate::models::theme::ThemeName::Terminal {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default().fg(self.bg).bg(self.accent)
        }
    }

    pub fn key_style(&self) -> Style {
        if self.name == crate::models::theme::ThemeName::Terminal {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.accent)
                .bg(self.bg)
                .add_modifier(Modifier::BOLD)
        }
    }

    pub fn muted_style(&self) -> Style {
        if self.name == crate::models::theme::ThemeName::Terminal {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(self.muted)
        }
    }

    pub fn bg_style(&self) -> Style {
        if self.bg == Color::Reset {
            Style::default()
        } else {
            Style::default().fg(self.bg)
        }
    }

    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    #[allow(dead_code)]
    pub fn gauge_style(&self) -> Style {
        Style::default().fg(self.accent).bg(self.highlight)
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn popup_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }
}

fn interpolate_color(c1: Color, c2: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (c1, c2) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f64 * (1.0 - t) + r2 as f64 * t).round() as u8;
            let g = (g1 as f64 * (1.0 - t) + g2 as f64 * t).round() as u8;
            let b = (b1 as f64 * (1.0 - t) + b2 as f64 * t).round() as u8;
            Color::Rgb(r, g, b)
        }
        _ => {
            if t < 0.33 {
                c1
            } else if t < 0.66 {
                Color::Cyan
            } else {
                c2
            }
        }
    }
}

pub fn render_gradient_gauge(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    ratio: f64,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let width = area.width as usize;
    let clamped_ratio = ratio.clamp(0.0, 1.0);
    let total_eights = (clamped_ratio * width as f64 * 8.0).round() as usize;
    let label_chars: Vec<char> = label.chars().collect();
    let label_len = label_chars.len();
    let label_start = if width > label_len {
        (width - label_len) / 2
    } else {
        0
    };
    let label_end = (label_start + label_len).min(width);
    let sub_blocks = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
    let mut spans = Vec::with_capacity(width);

    for x in 0..width {
        let col_eighths = total_eights.saturating_sub(x * 8);
        let t = if width > 1 {
            x as f64 / (width - 1) as f64
        } else {
            1.0
        };

        let col_color = if t < 0.5 {
            interpolate_color(theme.accent, theme.border_focused, t * 2.0)
        } else {
            interpolate_color(theme.border_focused, theme.success, (t - 0.5) * 2.0)
        };

        let is_label = x >= label_start && x < label_end;
        let label_char = if is_label {
            label_chars.get(x - label_start).copied()
        } else {
            None
        };

        if col_eighths >= 8 {
            if let Some(ch) = label_char {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(theme.bg)
                        .bg(col_color)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    "█",
                    Style::default().fg(col_color).bg(theme.bg),
                ));
            }
        } else if col_eighths > 0 {
            let frac_idx = col_eighths.min(7);
            let block_ch = sub_blocks[frac_idx];
            if let Some(ch) = label_char {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(col_color)
                        .bg(theme.bg)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    block_ch.to_string(),
                    Style::default().fg(col_color).bg(theme.bg),
                ));
            }
        } else {
            if let Some(ch) = label_char {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(theme.muted).bg(theme.bg),
                ));
            } else {
                spans.push(Span::styled(
                    "░",
                    Style::default().fg(theme.muted).bg(theme.bg),
                ));
            }
        }
    }

    let p = Paragraph::new(Line::from(spans)).style(theme.bg_style());
    frame.render_widget(p, area);
}
