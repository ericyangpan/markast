#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Text(String),
    BlankLine,
    Eof,
}

#[derive(Debug, Clone)]
pub(crate) struct SpannedToken {
    pub(crate) token: Token,
    pub(crate) span: Span,
}
