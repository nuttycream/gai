use crossterm::terminal;

use super::style::StyleConfig;

pub struct Renderer {
    pub style: StyleConfig,
    pub compact: bool,
    pub width: u16,
}

impl Renderer {
    pub(crate) fn new(
        style: StyleConfig,
        compact: bool,
    ) -> anyhow::Result<Self> {
        let width = terminal::size()?.1;

        //println!("width: {width}");

        Ok(Self {
            style,
            compact,
            width,
        })
    }
}
