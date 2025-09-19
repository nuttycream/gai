use indoc::indoc;
use ratatui::{text::Text, widgets::Widget};

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
        let str = indoc! {
            "
                      ░██
                         
 ░████████  ░██████   ░██
░██    ░██       ░██  ░██
░██    ░██  ░███████  ░██
░██   ░███ ░██   ░██  ░██
 ░█████░██  ░█████░██ ░██
       ░██               
 ░███████                
                         "
        };

        Text::raw(str).render(area, buf);
    }
}
