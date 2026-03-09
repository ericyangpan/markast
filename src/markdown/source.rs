use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(crate) struct Source<'a> {
    normalized: Cow<'a, str>,
}

impl<'a> Source<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        let normalized = if input.contains("\r\n") {
            Cow::Owned(input.replace("\r\n", "\n"))
        } else {
            Cow::Borrowed(input)
        };
        Self { normalized }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.normalized
    }
}
