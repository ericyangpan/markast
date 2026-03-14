#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    #[inline]
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    pub(crate) fn as_str<'a>(self, input: &'a str) -> &'a str {
        debug_assert!(input.is_char_boundary(self.start));
        debug_assert!(input.is_char_boundary(self.end));
        &input[self.start..self.end]
    }
}
