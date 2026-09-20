use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeName {
    #[default]
    Terminal,
    CatppuccinMocha,
    Dracula,
    TokyoNight,
    SolarizedDark,
    Nord,
    HighContrast,
}

impl ThemeName {
    pub const ALL: &'static [ThemeName] = &[
        ThemeName::Terminal,
        ThemeName::CatppuccinMocha,
        ThemeName::Dracula,
        ThemeName::TokyoNight,
        ThemeName::SolarizedDark,
        ThemeName::Nord,
        ThemeName::HighContrast,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::CatppuccinMocha => "catppuccin mocha",
            Self::Dracula => "dracula",
            Self::TokyoNight => "tokyo night",
            Self::SolarizedDark => "solarize dark",
            Self::Nord => "nord",
            Self::HighContrast => "high contrast",
        }
    }

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|&t| t == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn from_cli_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "terminal" | "inherit" => Some(Self::Terminal),
            "catppuccin-mocha" | "catppuccin" => Some(Self::CatppuccinMocha),
            "dracula" => Some(Self::Dracula),
            "tokyo-night" | "tokyonight" => Some(Self::TokyoNight),
            "solarized-dark" | "solarized" => Some(Self::SolarizedDark),
            "nord" => Some(Self::Nord),
            "high-contrast" | "highcontrast" => Some(Self::HighContrast),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: ThemeName,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border: Color,
    pub border_focused: Color,
    pub highlight: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Terminal => Self::terminal(),
            ThemeName::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemeName::Dracula => Self::dracula(),
            ThemeName::TokyoNight => Self::tokyo_night(),
            ThemeName::SolarizedDark => Self::solarized_dark(),
            ThemeName::Nord => Self::nord(),
            ThemeName::HighContrast => Self::high_contrast(),
        }
    }

    fn terminal() -> Self {
        Self {
            name: ThemeName::Terminal,
            bg: Color::Reset,
            fg: Color::Reset,
            accent: Color::Green,
            border: Color::Reset,
            border_focused: Color::Green,
            highlight: Color::Reset,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::Reset,
        }
    }
    fn catppuccin_mocha() -> Self {
        Self {
            name: ThemeName::CatppuccinMocha,
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(137, 180, 250),
            border: Color::Rgb(88, 91, 112),
            border_focused: Color::Rgb(180, 190, 254),
            highlight: Color::Rgb(69, 71, 90),
            success: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            error: Color::Rgb(243, 139, 168),
            muted: Color::Rgb(127, 132, 156),
        }
    }
    fn dracula() -> Self {
        Self {
            name: ThemeName::Dracula,
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(189, 147, 249),
            border: Color::Rgb(68, 71, 90),
            border_focused: Color::Rgb(139, 233, 253),
            highlight: Color::Rgb(68, 71, 90),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(241, 250, 140),
            error: Color::Rgb(255, 85, 85),
            muted: Color::Rgb(98, 114, 164),
        }
    }
    fn tokyo_night() -> Self {
        Self {
            name: ThemeName::TokyoNight,
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(122, 162, 247),
            border: Color::Rgb(59, 66, 97),
            border_focused: Color::Rgb(187, 154, 247),
            highlight: Color::Rgb(41, 46, 66),
            success: Color::Rgb(158, 206, 106),
            warning: Color::Rgb(224, 175, 104),
            error: Color::Rgb(247, 118, 142),
            muted: Color::Rgb(86, 95, 137),
        }
    }
    fn solarized_dark() -> Self {
        Self {
            name: ThemeName::SolarizedDark,
            bg: Color::Rgb(0, 43, 54),
            fg: Color::Rgb(131, 148, 150),
            accent: Color::Rgb(38, 139, 210),
            border: Color::Rgb(7, 54, 66),
            border_focused: Color::Rgb(108, 113, 196),
            highlight: Color::Rgb(7, 54, 66),
            success: Color::Rgb(133, 154, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            muted: Color::Rgb(88, 110, 117),
        }
    }
    fn nord() -> Self {
        Self {
            name: ThemeName::Nord,
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(136, 192, 208),
            border: Color::Rgb(67, 76, 94),
            border_focused: Color::Rgb(129, 161, 193),
            highlight: Color::Rgb(59, 66, 82),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            muted: Color::Rgb(76, 86, 106),
        }
    }
    fn high_contrast() -> Self {
        Self {
            name: ThemeName::HighContrast,
            bg: Color::Rgb(0, 0, 0),
            fg: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(0, 255, 255),
            border: Color::Rgb(128, 128, 128),
            border_focused: Color::Rgb(255, 255, 0),
            highlight: Color::Rgb(40, 40, 40),
            success: Color::Rgb(0, 255, 0),
            warning: Color::Rgb(255, 255, 0),
            error: Color::Rgb(255, 0, 0),
            muted: Color::Rgb(160, 160, 160),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_name(ThemeName::default())
    }
}
