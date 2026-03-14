use crate::{
    RenderOptions,
    markdown::{block, lexer::LineScanner, options::ParserOptions},
};

pub(crate) fn parse_document(
    input: &str,
    options: RenderOptions,
) -> crate::markdown::ast::Document {
    let parser_options = ParserOptions::from(options);
    if input.is_empty() {
        return crate::markdown::ast::Document::Nodes(Vec::new());
    }

    if !input.contains(['\n', '\r']) {
        let mut ctx = block::BlockParseContext::new();
        return ctx.parse_lines(&[input], parser_options.gfm, parser_options.pedantic);
    }

    let scanner = LineScanner::new(input);
    parse_with_scanner(&scanner, parser_options)
}

fn parse_with_scanner(
    scanner: &LineScanner<'_>,
    options: ParserOptions,
) -> crate::markdown::ast::Document {
    let mut parser = BlockParser::new(scanner.as_lines(), options);
    parser.parse()
}

struct BlockParser<'a> {
    lines: &'a [&'a str],
    gfm: bool,
    pedantic: bool,
}

impl<'a> BlockParser<'a> {
    fn new(lines: &'a [&'a str], options: ParserOptions) -> Self {
        Self {
            lines,
            gfm: options.gfm,
            pedantic: options.pedantic,
        }
    }

    fn parse(&mut self) -> crate::markdown::ast::Document {
        let mut ctx = block::BlockParseContext::new();
        ctx.parse_lines(self.lines, self.gfm, self.pedantic)
    }
}
