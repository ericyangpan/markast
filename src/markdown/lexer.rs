use crate::markdown::source::Source;

#[derive(Debug, Clone)]
pub(crate) struct Line<'a> {
    pub(crate) text: &'a str,
}

#[derive(Debug)]
pub(crate) struct LineScanner<'a> {
    lines: Vec<Line<'a>>,
}

impl<'a> LineScanner<'a> {
    pub(crate) fn new(source: &'a Source) -> Self {
        let lines = source
            .as_str()
            .split('\n')
            .enumerate()
            .map(|(_, text)| Line { text })
            .collect();
        Self { lines }
    }

    pub(crate) fn as_lines(&self) -> &[Line<'a>] {
        &self.lines
    }
}
