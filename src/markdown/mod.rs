use crate::RenderOptions;

mod ast;
mod block;
mod inline;
mod parser;
mod render_html;
mod autolink;
mod lexer;
mod options;
mod source;
mod token;
mod render;

pub(crate) fn render_markdown_to_html(input: &str, options: RenderOptions) -> String {
    render_html::render_markdown_to_html(input, options)
}
