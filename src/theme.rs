use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Theme color palette for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    // Semantic colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // UI colors
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub border: Color,
    pub border_focused: Color,

    // Status colors
    pub status_new: Color,
    pub status_progress: Color,
    pub status_feedback: Color,
    pub status_resolved: Color,
    pub status_closed: Color,

    // Priority colors
    pub priority_urgent: Color,
    pub priority_high: Color,
    pub priority_normal: Color,
    pub priority_low: Color,
}

/// Available theme names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeName {
    #[default]
    Default,
    CatppuccinMocha,
    CatppuccinMacchiato,
    CatppuccinFrappe,
    CatppuccinLatte,
    GruvboxDark,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    Nord,
    Dracula,
    OneDark,
    SolarizedDark,
    SolarizedLight,
    Monokai,
}

impl ThemeName {
    pub fn all() -> Vec<ThemeName> {
        vec![
            ThemeName::Default,
            ThemeName::CatppuccinMocha,
            ThemeName::CatppuccinMacchiato,
            ThemeName::CatppuccinFrappe,
            ThemeName::CatppuccinLatte,
            ThemeName::GruvboxDark,
            ThemeName::TokyoNight,
            ThemeName::TokyoNightStorm,
            ThemeName::TokyoNightLight,
            ThemeName::Nord,
            ThemeName::Dracula,
            ThemeName::OneDark,
            ThemeName::SolarizedDark,
            ThemeName::SolarizedLight,
            ThemeName::Monokai,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            ThemeName::Default => "Default",
            ThemeName::CatppuccinMocha => "Catppuccin Mocha",
            ThemeName::CatppuccinMacchiato => "Catppuccin Macchiato",
            ThemeName::CatppuccinFrappe => "Catppuccin Frappe",
            ThemeName::CatppuccinLatte => "Catppuccin Latte",
            ThemeName::GruvboxDark => "Gruvbox Dark",
            ThemeName::TokyoNight => "Tokyo Night",
            ThemeName::TokyoNightStorm => "Tokyo Night Storm",
            ThemeName::TokyoNightLight => "Tokyo Night Light",
            ThemeName::Nord => "Nord",
            ThemeName::Dracula => "Dracula",
            ThemeName::OneDark => "One Dark",
            ThemeName::SolarizedDark => "Solarized Dark",
            ThemeName::SolarizedLight => "Solarized Light",
            ThemeName::Monokai => "Monokai",
        }
    }
}

impl std::fmt::Display for ThemeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Theme {
    /// Get theme by name
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Default => Self::default(),
            ThemeName::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemeName::CatppuccinMacchiato => Self::catppuccin_macchiato(),
            ThemeName::CatppuccinFrappe => Self::catppuccin_frappe(),
            ThemeName::CatppuccinLatte => Self::catppuccin_latte(),
            ThemeName::GruvboxDark => Self::gruvbox_dark(),
            ThemeName::TokyoNight => Self::tokyo_night(),
            ThemeName::TokyoNightStorm => Self::tokyo_night_storm(),
            ThemeName::TokyoNightLight => Self::tokyo_night_light(),
            ThemeName::Nord => Self::nord(),
            ThemeName::Dracula => Self::dracula(),
            ThemeName::OneDark => Self::one_dark(),
            ThemeName::SolarizedDark => Self::solarized_dark(),
            ThemeName::SolarizedLight => Self::solarized_light(),
            ThemeName::Monokai => Self::monokai(),
        }
    }

    /// Create a new theme with default colors
    pub fn new_default() -> Self {
        Self {
            // Semantic colors
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,

            // UI colors
            background: Color::Reset,
            surface: Color::Reset,
            text: Color::White,
            text_secondary: Color::Gray,
            text_muted: Color::DarkGray,
            border: Color::Gray,
            border_focused: Color::Cyan,

            // Status colors
            status_new: Color::Cyan,
            status_progress: Color::Yellow,
            status_feedback: Color::Magenta,
            status_resolved: Color::Green,
            status_closed: Color::DarkGray,

            // Priority colors
            priority_urgent: Color::Red,
            priority_high: Color::LightRed,
            priority_normal: Color::White,
            priority_low: Color::DarkGray,
        }
    }

    /// Catppuccin Mocha theme (dark, warm)
    pub fn catppuccin_mocha() -> Self {
        Self {
            // Semantic colors - using Catppuccin Mocha palette
            primary: Color::Rgb(137, 180, 250),   // Blue
            secondary: Color::Rgb(180, 190, 254), // Lavender
            accent: Color::Rgb(203, 166, 247),    // Mauve
            success: Color::Rgb(166, 227, 161),   // Green
            warning: Color::Rgb(249, 226, 175),   // Yellow
            error: Color::Rgb(243, 139, 168),     // Red
            info: Color::Rgb(148, 226, 213),      // Teal

            // UI colors
            background: Color::Rgb(30, 30, 46),        // Base
            surface: Color::Rgb(49, 50, 68),           // Surface0
            text: Color::Rgb(205, 214, 244),           // Text
            text_secondary: Color::Rgb(186, 194, 222), // Subtext1
            text_muted: Color::Rgb(166, 173, 200),     // Subtext0
            border: Color::Rgb(88, 91, 112),           // Surface2
            border_focused: Color::Rgb(137, 180, 250), // Blue

            // Status colors
            status_new: Color::Rgb(137, 180, 250),      // Blue
            status_progress: Color::Rgb(249, 226, 175), // Yellow
            status_feedback: Color::Rgb(203, 166, 247), // Mauve
            status_resolved: Color::Rgb(166, 227, 161), // Green
            status_closed: Color::Rgb(127, 132, 156),   // Overlay0

            // Priority colors
            priority_urgent: Color::Rgb(243, 139, 168), // Red
            priority_high: Color::Rgb(235, 160, 172),   // Maroon
            priority_normal: Color::Rgb(205, 214, 244), // Text
            priority_low: Color::Rgb(166, 173, 200),    // Subtext0
        }
    }

    /// Catppuccin Macchiato theme (dark, cool)
    pub fn catppuccin_macchiato() -> Self {
        Self {
            // Semantic colors
            primary: Color::Rgb(138, 173, 244),   // Blue
            secondary: Color::Rgb(183, 189, 248), // Lavender
            accent: Color::Rgb(198, 160, 246),    // Mauve
            success: Color::Rgb(166, 218, 149),   // Green
            warning: Color::Rgb(238, 212, 159),   // Yellow
            error: Color::Rgb(237, 135, 150),     // Red
            info: Color::Rgb(139, 213, 202),      // Teal

            // UI colors
            background: Color::Rgb(36, 39, 58),        // Base
            surface: Color::Rgb(54, 58, 79),           // Surface0
            text: Color::Rgb(202, 211, 245),           // Text
            text_secondary: Color::Rgb(184, 192, 224), // Subtext1
            text_muted: Color::Rgb(165, 173, 203),     // Subtext0
            border: Color::Rgb(91, 96, 120),           // Surface2
            border_focused: Color::Rgb(138, 173, 244), // Blue

            // Status colors
            status_new: Color::Rgb(138, 173, 244),      // Blue
            status_progress: Color::Rgb(238, 212, 159), // Yellow
            status_feedback: Color::Rgb(198, 160, 246), // Mauve
            status_resolved: Color::Rgb(166, 218, 149), // Green
            status_closed: Color::Rgb(128, 135, 162),   // Overlay0

            // Priority colors
            priority_urgent: Color::Rgb(237, 135, 150), // Red
            priority_high: Color::Rgb(238, 153, 160),   // Maroon
            priority_normal: Color::Rgb(202, 211, 245), // Text
            priority_low: Color::Rgb(165, 173, 203),    // Subtext0
        }
    }

    /// Catppuccin Frappe theme (dark, neutral)
    pub fn catppuccin_frappe() -> Self {
        Self {
            // Semantic colors
            primary: Color::Rgb(140, 170, 238),   // Blue
            secondary: Color::Rgb(186, 187, 241), // Lavender
            accent: Color::Rgb(202, 158, 230),    // Mauve
            success: Color::Rgb(166, 209, 137),   // Green
            warning: Color::Rgb(229, 200, 144),   // Yellow
            error: Color::Rgb(231, 130, 132),     // Red
            info: Color::Rgb(129, 200, 190),      // Teal

            // UI colors
            background: Color::Rgb(48, 52, 70),        // Base
            surface: Color::Rgb(65, 69, 89),           // Surface0
            text: Color::Rgb(198, 208, 245),           // Text
            text_secondary: Color::Rgb(181, 191, 226), // Subtext1
            text_muted: Color::Rgb(163, 173, 200),     // Subtext0
            border: Color::Rgb(98, 104, 128),          // Surface2
            border_focused: Color::Rgb(140, 170, 238), // Blue

            // Status colors
            status_new: Color::Rgb(140, 170, 238),      // Blue
            status_progress: Color::Rgb(229, 200, 144), // Yellow
            status_feedback: Color::Rgb(202, 158, 230), // Mauve
            status_resolved: Color::Rgb(166, 209, 137), // Green
            status_closed: Color::Rgb(131, 139, 167),   // Overlay0

            // Priority colors
            priority_urgent: Color::Rgb(231, 130, 132), // Red
            priority_high: Color::Rgb(234, 153, 156),   // Maroon
            priority_normal: Color::Rgb(198, 208, 245), // Text
            priority_low: Color::Rgb(163, 173, 200),    // Subtext0
        }
    }

    /// Catppuccin Latte theme (light)
    pub fn catppuccin_latte() -> Self {
        Self {
            // Semantic colors
            primary: Color::Rgb(30, 102, 245),    // Blue
            secondary: Color::Rgb(114, 135, 253), // Lavender
            accent: Color::Rgb(136, 57, 239),     // Mauve
            success: Color::Rgb(64, 160, 43),     // Green
            warning: Color::Rgb(223, 142, 29),    // Yellow
            error: Color::Rgb(210, 15, 57),       // Red
            info: Color::Rgb(23, 146, 153),       // Teal

            // UI colors
            background: Color::Rgb(239, 241, 245),    // Base
            surface: Color::Rgb(230, 233, 239),       // Surface0
            text: Color::Rgb(76, 79, 105),            // Text
            text_secondary: Color::Rgb(92, 95, 119),  // Subtext1
            text_muted: Color::Rgb(108, 111, 133),    // Subtext0
            border: Color::Rgb(188, 192, 204),        // Surface2
            border_focused: Color::Rgb(30, 102, 245), // Blue

            // Status colors
            status_new: Color::Rgb(30, 102, 245),      // Blue
            status_progress: Color::Rgb(223, 142, 29), // Yellow
            status_feedback: Color::Rgb(136, 57, 239), // Mauve
            status_resolved: Color::Rgb(64, 160, 43),  // Green
            status_closed: Color::Rgb(140, 143, 161),  // Overlay0

            // Priority colors
            priority_urgent: Color::Rgb(210, 15, 57), // Red
            priority_high: Color::Rgb(230, 69, 83),   // Maroon
            priority_normal: Color::Rgb(76, 79, 105), // Text
            priority_low: Color::Rgb(108, 111, 133),  // Subtext0
        }
    }

    /// Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        Self {
            primary: Color::Rgb(254, 128, 25),    // Orange
            secondary: Color::Rgb(184, 187, 38),  // Green
            accent: Color::Rgb(211, 134, 155),    // Purple
            success: Color::Rgb(184, 187, 38),    // Green
            warning: Color::Rgb(250, 189, 47),    // Yellow
            error: Color::Rgb(251, 73, 52),       // Red
            info: Color::Rgb(131, 165, 152),      // Aqua
            background: Color::Rgb(40, 40, 40),   // bg0
            surface: Color::Rgb(60, 56, 54),      // bg1
            text: Color::Rgb(235, 219, 178),      // fg0
            text_secondary: Color::Rgb(213, 196, 161), // fg1
            text_muted: Color::Rgb(189, 174, 147),     // fg2
            border: Color::Rgb(102, 92, 84),      // bg3
            border_focused: Color::Rgb(254, 128, 25), // Orange
            status_new: Color::Rgb(131, 165, 152),      // Aqua
            status_progress: Color::Rgb(250, 189, 47),  // Yellow
            status_feedback: Color::Rgb(211, 134, 155), // Purple
            status_resolved: Color::Rgb(184, 187, 38),  // Green
            status_closed: Color::Rgb(146, 131, 116),   // fg4
            priority_urgent: Color::Rgb(251, 73, 52),   // Red
            priority_high: Color::Rgb(254, 128, 25),    // Orange
            priority_normal: Color::Rgb(235, 219, 178), // fg0
            priority_low: Color::Rgb(189, 174, 147),    // fg2
        }
    }

    /// Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        Self {
            primary: Color::Rgb(175, 58, 3),      // Orange
            secondary: Color::Rgb(121, 116, 14),  // Green
            accent: Color::Rgb(143, 63, 113),     // Purple
            success: Color::Rgb(121, 116, 14),    // Green
            warning: Color::Rgb(181, 118, 20),    // Yellow
            error: Color::Rgb(204, 36, 29),       // Red
            info: Color::Rgb(66, 123, 88),        // Aqua
            background: Color::Rgb(251, 241, 199), // bg0
            surface: Color::Rgb(235, 219, 178),    // bg1
            text: Color::Rgb(40, 40, 40),          // fg0
            text_secondary: Color::Rgb(60, 56, 54), // fg1
            text_muted: Color::Rgb(102, 92, 84),    // fg2
            border: Color::Rgb(213, 196, 161),      // bg3
            border_focused: Color::Rgb(175, 58, 3), // Orange
            status_new: Color::Rgb(66, 123, 88),     // Aqua
            status_progress: Color::Rgb(181, 118, 20), // Yellow
            status_feedback: Color::Rgb(143, 63, 113), // Purple
            status_resolved: Color::Rgb(121, 116, 14), // Green
            status_closed: Color::Rgb(146, 131, 116),  // fg4
            priority_urgent: Color::Rgb(204, 36, 29),  // Red
            priority_high: Color::Rgb(175, 58, 3),     // Orange
            priority_normal: Color::Rgb(40, 40, 40),   // fg0
            priority_low: Color::Rgb(102, 92, 84),     // fg2
        }
    }

    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            primary: Color::Rgb(125, 207, 255),   // Blue
            secondary: Color::Rgb(187, 154, 247), // Purple
            accent: Color::Rgb(255, 158, 100),    // Orange
            success: Color::Rgb(158, 206, 106),   // Green
            warning: Color::Rgb(224, 175, 104),   // Yellow
            error: Color::Rgb(247, 118, 142),     // Red
            info: Color::Rgb(125, 207, 255),      // Blue
            background: Color::Rgb(26, 27, 38),   // bg
            surface: Color::Rgb(36, 40, 59),      // bg_dark
            text: Color::Rgb(192, 202, 245),      // fg
            text_secondary: Color::Rgb(169, 177, 214), // fg_dark
            text_muted: Color::Rgb(86, 95, 137),       // comment
            border: Color::Rgb(41, 46, 66),       // bg_highlight
            border_focused: Color::Rgb(125, 207, 255), // Blue
            status_new: Color::Rgb(125, 207, 255),      // Blue
            status_progress: Color::Rgb(224, 175, 104), // Yellow
            status_feedback: Color::Rgb(187, 154, 247), // Purple
            status_resolved: Color::Rgb(158, 206, 106), // Green
            status_closed: Color::Rgb(86, 95, 137),     // comment
            priority_urgent: Color::Rgb(247, 118, 142), // Red
            priority_high: Color::Rgb(255, 158, 100),   // Orange
            priority_normal: Color::Rgb(192, 202, 245), // fg
            priority_low: Color::Rgb(86, 95, 137),      // comment
        }
    }

    /// Tokyo Night Storm theme
    pub fn tokyo_night_storm() -> Self {
        Self {
            primary: Color::Rgb(130, 170, 255),   // Blue
            secondary: Color::Rgb(187, 154, 247), // Purple
            accent: Color::Rgb(255, 158, 100),    // Orange
            success: Color::Rgb(158, 206, 106),   // Green
            warning: Color::Rgb(224, 175, 104),   // Yellow
            error: Color::Rgb(247, 118, 142),     // Red
            info: Color::Rgb(125, 207, 255),      // Cyan
            background: Color::Rgb(36, 40, 59),   // bg
            surface: Color::Rgb(41, 46, 66),      // bg_dark
            text: Color::Rgb(166, 173, 200),      // fg
            text_secondary: Color::Rgb(134, 142, 177), // fg_dark
            text_muted: Color::Rgb(86, 95, 137),       // comment
            border: Color::Rgb(52, 59, 88),       // bg_highlight
            border_focused: Color::Rgb(130, 170, 255), // Blue
            status_new: Color::Rgb(130, 170, 255),      // Blue
            status_progress: Color::Rgb(224, 175, 104), // Yellow
            status_feedback: Color::Rgb(187, 154, 247), // Purple
            status_resolved: Color::Rgb(158, 206, 106), // Green
            status_closed: Color::Rgb(86, 95, 137),     // comment
            priority_urgent: Color::Rgb(247, 118, 142), // Red
            priority_high: Color::Rgb(255, 158, 100),   // Orange
            priority_normal: Color::Rgb(166, 173, 200), // fg
            priority_low: Color::Rgb(86, 95, 137),      // comment
        }
    }

    /// Tokyo Night Light theme
    pub fn tokyo_night_light() -> Self {
        Self {
            primary: Color::Rgb(52, 84, 138),     // Blue
            secondary: Color::Rgb(136, 57, 239),  // Purple
            accent: Color::Rgb(150, 80, 39),      // Orange
            success: Color::Rgb(51, 110, 23),     // Green
            warning: Color::Rgb(143, 94, 21),     // Yellow
            error: Color::Rgb(186, 23, 39),       // Red
            info: Color::Rgb(15, 75, 110),        // Cyan
            background: Color::Rgb(213, 214, 219), // bg
            surface: Color::Rgb(232, 232, 234),    // bg_dark
            text: Color::Rgb(52, 59, 88),          // fg
            text_secondary: Color::Rgb(86, 95, 137), // fg_dark
            text_muted: Color::Rgb(154, 160, 177),   // comment
            border: Color::Rgb(232, 232, 234),       // bg_highlight
            border_focused: Color::Rgb(52, 84, 138), // Blue
            status_new: Color::Rgb(52, 84, 138),      // Blue
            status_progress: Color::Rgb(143, 94, 21), // Yellow
            status_feedback: Color::Rgb(136, 57, 239), // Purple
            status_resolved: Color::Rgb(51, 110, 23),  // Green
            status_closed: Color::Rgb(154, 160, 177),  // comment
            priority_urgent: Color::Rgb(186, 23, 39),  // Red
            priority_high: Color::Rgb(150, 80, 39),    // Orange
            priority_normal: Color::Rgb(52, 59, 88),   // fg
            priority_low: Color::Rgb(154, 160, 177),   // comment
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            primary: Color::Rgb(136, 192, 208),   // nord8 (Frost)
            secondary: Color::Rgb(129, 161, 193), // nord9 (Frost)
            accent: Color::Rgb(143, 188, 187),    // nord7 (Frost)
            success: Color::Rgb(163, 190, 140),   // nord14 (Aurora)
            warning: Color::Rgb(235, 203, 139),   // nord13 (Aurora)
            error: Color::Rgb(191, 97, 106),      // nord11 (Aurora)
            info: Color::Rgb(136, 192, 208),      // nord8
            background: Color::Rgb(46, 52, 64),   // nord0
            surface: Color::Rgb(59, 66, 82),      // nord1
            text: Color::Rgb(236, 239, 244),      // nord6
            text_secondary: Color::Rgb(229, 233, 240), // nord5
            text_muted: Color::Rgb(216, 222, 233),     // nord4
            border: Color::Rgb(76, 86, 106),      // nord3
            border_focused: Color::Rgb(136, 192, 208), // nord8
            status_new: Color::Rgb(136, 192, 208),      // nord8
            status_progress: Color::Rgb(235, 203, 139), // nord13
            status_feedback: Color::Rgb(180, 142, 173), // nord15
            status_resolved: Color::Rgb(163, 190, 140), // nord14
            status_closed: Color::Rgb(216, 222, 233),   // nord4
            priority_urgent: Color::Rgb(191, 97, 106),  // nord11
            priority_high: Color::Rgb(208, 135, 112),   // nord12
            priority_normal: Color::Rgb(236, 239, 244), // nord6
            priority_low: Color::Rgb(216, 222, 233),    // nord4
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            primary: Color::Rgb(139, 233, 253),   // Cyan
            secondary: Color::Rgb(189, 147, 249), // Purple
            accent: Color::Rgb(255, 121, 198),    // Pink
            success: Color::Rgb(80, 250, 123),    // Green
            warning: Color::Rgb(241, 250, 140),   // Yellow
            error: Color::Rgb(255, 85, 85),       // Red
            info: Color::Rgb(139, 233, 253),      // Cyan
            background: Color::Rgb(40, 42, 54),   // Background
            surface: Color::Rgb(68, 71, 90),      // Current Line
            text: Color::Rgb(248, 248, 242),      // Foreground
            text_secondary: Color::Rgb(248, 248, 242), // Foreground
            text_muted: Color::Rgb(98, 114, 164),      // Comment
            border: Color::Rgb(68, 71, 90),       // Current Line
            border_focused: Color::Rgb(139, 233, 253), // Cyan
            status_new: Color::Rgb(139, 233, 253),      // Cyan
            status_progress: Color::Rgb(241, 250, 140), // Yellow
            status_feedback: Color::Rgb(189, 147, 249), // Purple
            status_resolved: Color::Rgb(80, 250, 123),  // Green
            status_closed: Color::Rgb(98, 114, 164),    // Comment
            priority_urgent: Color::Rgb(255, 85, 85),   // Red
            priority_high: Color::Rgb(255, 184, 108),   // Orange
            priority_normal: Color::Rgb(248, 248, 242), // Foreground
            priority_low: Color::Rgb(98, 114, 164),     // Comment
        }
    }

    /// One Dark theme
    pub fn one_dark() -> Self {
        Self {
            primary: Color::Rgb(97, 175, 239),    // Blue
            secondary: Color::Rgb(198, 120, 221), // Purple
            accent: Color::Rgb(209, 154, 102),    // Orange
            success: Color::Rgb(152, 195, 121),   // Green
            warning: Color::Rgb(229, 192, 123),   // Yellow
            error: Color::Rgb(224, 108, 117),     // Red
            info: Color::Rgb(86, 182, 194),       // Cyan
            background: Color::Rgb(40, 44, 52),   // Background
            surface: Color::Rgb(53, 59, 69),      // Black
            text: Color::Rgb(171, 178, 191),      // Foreground
            text_secondary: Color::Rgb(171, 178, 191), // Foreground
            text_muted: Color::Rgb(92, 99, 112),       // Gray
            border: Color::Rgb(76, 82, 99),       // Gutter gray
            border_focused: Color::Rgb(97, 175, 239), // Blue
            status_new: Color::Rgb(97, 175, 239),      // Blue
            status_progress: Color::Rgb(229, 192, 123), // Yellow
            status_feedback: Color::Rgb(198, 120, 221), // Purple
            status_resolved: Color::Rgb(152, 195, 121), // Green
            status_closed: Color::Rgb(92, 99, 112),     // Gray
            priority_urgent: Color::Rgb(224, 108, 117), // Red
            priority_high: Color::Rgb(209, 154, 102),   // Orange
            priority_normal: Color::Rgb(171, 178, 191), // Foreground
            priority_low: Color::Rgb(92, 99, 112),      // Gray
        }
    }

    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),    // Blue
            secondary: Color::Rgb(108, 113, 196), // Violet
            accent: Color::Rgb(211, 54, 130),     // Magenta
            success: Color::Rgb(133, 153, 0),     // Green
            warning: Color::Rgb(181, 137, 0),     // Yellow
            error: Color::Rgb(220, 50, 47),       // Red
            info: Color::Rgb(42, 161, 152),       // Cyan
            background: Color::Rgb(0, 43, 54),    // base03
            surface: Color::Rgb(7, 54, 66),       // base02
            text: Color::Rgb(131, 148, 150),      // base0
            text_secondary: Color::Rgb(147, 161, 161), // base1
            text_muted: Color::Rgb(88, 110, 117),      // base01
            border: Color::Rgb(7, 54, 66),        // base02
            border_focused: Color::Rgb(38, 139, 210), // Blue
            status_new: Color::Rgb(38, 139, 210),      // Blue
            status_progress: Color::Rgb(181, 137, 0),  // Yellow
            status_feedback: Color::Rgb(108, 113, 196), // Violet
            status_resolved: Color::Rgb(133, 153, 0),  // Green
            status_closed: Color::Rgb(88, 110, 117),   // base01
            priority_urgent: Color::Rgb(220, 50, 47),  // Red
            priority_high: Color::Rgb(203, 75, 22),    // Orange
            priority_normal: Color::Rgb(131, 148, 150), // base0
            priority_low: Color::Rgb(88, 110, 117),     // base01
        }
    }

    /// Solarized Light theme
    pub fn solarized_light() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),    // Blue
            secondary: Color::Rgb(108, 113, 196), // Violet
            accent: Color::Rgb(211, 54, 130),     // Magenta
            success: Color::Rgb(133, 153, 0),     // Green
            warning: Color::Rgb(181, 137, 0),     // Yellow
            error: Color::Rgb(220, 50, 47),       // Red
            info: Color::Rgb(42, 161, 152),       // Cyan
            background: Color::Rgb(253, 246, 227), // base3
            surface: Color::Rgb(238, 232, 213),    // base2
            text: Color::Rgb(101, 123, 131),       // base00
            text_secondary: Color::Rgb(88, 110, 117),  // base01
            text_muted: Color::Rgb(147, 161, 161),     // base1
            border: Color::Rgb(238, 232, 213),         // base2
            border_focused: Color::Rgb(38, 139, 210),  // Blue
            status_new: Color::Rgb(38, 139, 210),      // Blue
            status_progress: Color::Rgb(181, 137, 0),  // Yellow
            status_feedback: Color::Rgb(108, 113, 196), // Violet
            status_resolved: Color::Rgb(133, 153, 0),  // Green
            status_closed: Color::Rgb(147, 161, 161),  // base1
            priority_urgent: Color::Rgb(220, 50, 47),  // Red
            priority_high: Color::Rgb(203, 75, 22),    // Orange
            priority_normal: Color::Rgb(101, 123, 131), // base00
            priority_low: Color::Rgb(147, 161, 161),    // base1
        }
    }

    /// Monokai theme
    pub fn monokai() -> Self {
        Self {
            primary: Color::Rgb(102, 217, 239),   // Blue
            secondary: Color::Rgb(171, 157, 242), // Purple
            accent: Color::Rgb(249, 38, 114),     // Pink
            success: Color::Rgb(166, 226, 46),    // Green
            warning: Color::Rgb(244, 191, 117),   // Yellow
            error: Color::Rgb(249, 38, 114),      // Red
            info: Color::Rgb(102, 217, 239),      // Blue
            background: Color::Rgb(39, 40, 34),   // Background
            surface: Color::Rgb(73, 72, 62),      // Highlight
            text: Color::Rgb(248, 248, 242),      // Foreground
            text_secondary: Color::Rgb(248, 248, 242), // Foreground
            text_muted: Color::Rgb(117, 113, 94),      // Comment
            border: Color::Rgb(73, 72, 62),       // Highlight
            border_focused: Color::Rgb(102, 217, 239), // Blue
            status_new: Color::Rgb(102, 217, 239),      // Blue
            status_progress: Color::Rgb(244, 191, 117), // Yellow
            status_feedback: Color::Rgb(171, 157, 242), // Purple
            status_resolved: Color::Rgb(166, 226, 46),  // Green
            status_closed: Color::Rgb(117, 113, 94),    // Comment
            priority_urgent: Color::Rgb(249, 38, 114),  // Red
            priority_high: Color::Rgb(253, 151, 31),    // Orange
            priority_normal: Color::Rgb(248, 248, 242), // Foreground
            priority_low: Color::Rgb(117, 113, 94),     // Comment
        }
    }

    /// Get color for status name (backward compatibility)
    pub fn get_status_color(&self, status: &str) -> Color {
        let status_lower = status.to_lowercase();
        match status_lower.as_str() {
            s if s.contains("new") => self.status_new,
            s if s.contains("progress") => self.status_progress,
            s if s.contains("feedback") => self.status_feedback,
            s if s.contains("resolved") => self.status_resolved,
            s if s.contains("closed") => self.status_closed,
            _ => self.text,
        }
    }

    /// Get color for priority name (backward compatibility)
    pub fn get_priority_color(&self, priority: &str) -> Color {
        let priority_lower = priority.to_lowercase();
        match priority_lower.as_str() {
            s if s.contains("urgent") || s.contains("immediate") => self.priority_urgent,
            s if s.contains("high") => self.priority_high,
            s if s.contains("normal") => self.priority_normal,
            s if s.contains("low") => self.priority_low,
            _ => self.text,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_name_display() {
        assert_eq!(ThemeName::Default.to_string(), "Default");
        assert_eq!(ThemeName::CatppuccinMocha.to_string(), "Catppuccin Mocha");
    }

    #[test]
    fn test_all_themes_loadable() {
        for theme_name in ThemeName::all() {
            let _theme = Theme::from_name(theme_name);
            // If we get here without panic, theme loaded successfully
        }
    }

    #[test]
    fn test_status_colors() {
        let theme = Theme::default();
        assert_eq!(theme.get_status_color("New"), theme.status_new);
        assert_eq!(theme.get_status_color("In Progress"), theme.status_progress);
        assert_eq!(theme.get_status_color("Closed"), theme.status_closed);
    }

    #[test]
    fn test_priority_colors() {
        let theme = Theme::default();
        assert_eq!(theme.get_priority_color("Urgent"), theme.priority_urgent);
        assert_eq!(theme.get_priority_color("High"), theme.priority_high);
        assert_eq!(theme.get_priority_color("Normal"), theme.priority_normal);
        assert_eq!(theme.get_priority_color("Low"), theme.priority_low);
    }
}
