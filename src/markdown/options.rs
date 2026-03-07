use crate::RenderOptions;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParserOptions {
    pub gfm: bool,
    pub breaks: bool,
    pub pedantic: bool,
}

impl From<RenderOptions> for ParserOptions {
    fn from(value: RenderOptions) -> Self {
        Self {
            gfm: value.gfm,
            breaks: value.breaks,
            pedantic: value.pedantic,
        }
    }
}

impl From<ParserOptions> for RenderOptions {
    fn from(value: ParserOptions) -> Self {
        Self {
            gfm: value.gfm,
            breaks: value.breaks,
            pedantic: value.pedantic,
        }
    }
}
