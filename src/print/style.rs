use crossterm::style::Color;

/// configurable styling
/// available styles
#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub allow_colors: bool,

    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub highlight: Color,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            allow_colors: true,

            primary: Color::White,
            secondary: Color::DarkGrey,
            tertiary: Color::Yellow,
            highlight: Color::Blue,
        }
    }
}
