use owo_colors::Style;

/// configurable styling
/// available styles
#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub primary: Style,
    pub secondary: Style,
    pub highlight: Style,

    pub warning: Style,
    pub error: Style,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            primary: Style::new().white(),
            secondary: Style::new().magenta(),
            highlight: Style::new().blue(),

            warning: Style::new().yellow(),
            error: Style::new().red(),
        }
    }
}
