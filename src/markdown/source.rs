#[derive(Debug, Clone)]
pub(crate) struct Source {
    normalized: String,
}

impl Source {
    pub(crate) fn new(input: &str) -> Self {
        let normalized = input.replace("\r\n", "\n");
        Self { normalized }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.normalized
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.normalized.is_empty()
    }
}
