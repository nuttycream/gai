use ratatui::{text::Text, widgets::Widget};

use crate::consts::COMMIT_CONVENTION;

pub fn build_prompt(
    use_convention: bool,
    sys_prompt: &str,
    rules: &str,
) -> String {
    let convention = if use_convention {
        format!("Convention:\n{}", COMMIT_CONVENTION)
    } else {
        "".to_owned()
    };

    format!("{}\nRules:\n{}\n{}", sys_prompt, rules, convention)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GaiLogo {}

impl GaiLogo {
    pub fn new() -> Self {
        GaiLogo {}
    }
}

impl Widget for GaiLogo {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        let str = "";

        Text::raw(str).render(area, buf);
    }
}
