use super::style::StyleConfig;

pub struct Renderer {
    pub style: StyleConfig,
    pub compact: bool,
}

impl Renderer {
    pub(crate) fn new(
        style: StyleConfig,
        compact: bool,
    ) -> anyhow::Result<Self> {
        Ok(Self { style, compact })
    }
}
