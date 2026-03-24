use crossterm::style::Color;

/// configurable styling
/// available styles
#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub primary: Color,
    pub secondary: Color,
    pub highlight: Color,

    pub allow_bold: bool,
    pub allow_italic: bool,
    pub allow_underline: bool,
    pub allow_strikethrough: bool,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            primary: Color::White,
            secondary: Color::DarkGrey,
            highlight: Color::Blue,

            allow_bold: true,
            allow_italic: true,
            allow_underline: true,
            allow_strikethrough: true,
        }
    }
}
