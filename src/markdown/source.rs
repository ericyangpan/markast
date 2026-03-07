#[derive(Debug, Clone)]
pub(crate) struct Source {
    normalized: String,
    original_len: usize,
}

impl Source {
    pub(crate) fn new(input: &str) -> Self {
        let normalized = input.replace("\r\n", "\n");
        Self {
            original_len: input.len(),
            normalized,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.normalized
    }

    pub(crate) fn len(&self) -> usize {
        self.normalized.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.normalized.is_empty()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.normalized.split('\n').count()
    }

    pub(crate) fn original_len(&self) -> usize {
        self.original_len
    }
}
