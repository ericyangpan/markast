use crate::{
    RenderOptions,
    markdown::{autolink, parser, render},
};

pub(crate) fn render_markdown_to_html(input: &str, options: RenderOptions) -> String {
    let document = parser::parse_document(input, options);
    let html = render::render_document(&document, options);
    autolink::post_process_document_html(html, options)
}
