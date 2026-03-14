#[derive(Debug)]
pub(crate) struct LineScanner<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineScanner<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        let bytes = input.as_bytes();
        let mut lines = Vec::with_capacity((bytes.len() / 32).max(4));

        let mut start = 0usize;
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    lines.push(&input[start..i]);
                    start = i + 1;
                }
                b'\r' => {
                    lines.push(&input[start..i]);
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        i += 1;
                    }
                    start = i + 1;
                }
                _ => {}
            }
            i += 1;
        }

        lines.push(&input[start..]);
        Self { lines }
    }

    pub(crate) fn as_lines(&self) -> &[&'a str] {
        &self.lines
    }
}
