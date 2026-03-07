use crate::markdown::{source::Source, token::SpannedToken};

#[derive(Debug, Clone)]
pub(crate) struct Line<'a> {
    pub(crate) number: usize,
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
            .map(|(index, text)| Line {
                number: index,
                text,
            })
            .collect();
        Self { lines }
    }

    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|line| line.text)
    }

    pub(crate) fn as_lines(&self) -> &[Line<'a>] {
        &self.lines
    }

    pub(crate) fn _to_tokens(&self) -> Vec<SpannedToken> {
        self.lines
            .iter()
            .map(|line| SpannedToken {
                token: crate::markdown::token::Token::Text(line.text.to_string()),
                span: crate::markdown::token::Span::new(0, line.text.len()),
            })
            .collect()
    }
}
