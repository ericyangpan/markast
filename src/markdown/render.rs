use std::fmt::Write as _;

use crate::RenderOptions;
use crate::markdown::ast::{self, inline::Inline};

pub(crate) fn render_document(doc: &ast::Document, options: RenderOptions) -> String {
    let mut out = String::new();

    match doc {
        ast::Document::Nodes(blocks) => {
            for block in blocks {
                render_block(block, &mut out, options);
            }
        }
    }
    out
}

fn render_block(block: &ast::Block, out: &mut String, options: RenderOptions) {
    match block {
        ast::Block::Paragraph { inlines } => {
            out.push_str("<p>");
            render_inlines(inlines, out, options);
            out.push_str("</p>\n");
        }
        ast::Block::Heading { level, inlines } => {
            write!(out, "<h{level}>").expect("write heading");
            render_inlines(inlines, out, options);
            write!(out, "</h{level}>\n").expect("write heading close");
        }
        ast::Block::List {
            ordered,
            start,
            tight,
            items,
        } => {
            if *ordered {
                if *start == 1 {
                    out.push_str("<ol>");
                } else {
                    write!(out, "<ol start=\"{start}\">").expect("write ordered list start");
                }
            } else {
                out.push_str("<ul>");
            }
            for item in items {
                out.push_str("<li>");
                let children = &item.children;
                if *tight {
                    if let Some(ast::Block::Paragraph { inlines }) = children.first() {
                        if let Some(done) = item.task {
                            render_task_checkbox(done, out);
                        }
                        render_inlines(inlines, out, options);
                        if children.len() == 1 {
                            out.push_str("</li>");
                            continue;
                        }
                        render_tight_list_separator(&children[1], out);
                        for child in &children[1..] {
                            render_block(child, out, options);
                        }
                        out.push_str("</li>");
                        continue;
                    }
                }

                let mut rendered_task_paragraph = false;
                if let Some(done) = item.task {
                    if let Some(ast::Block::Paragraph { inlines }) = children.first() {
                        out.push_str("<p>");
                        render_task_checkbox(done, out);
                        if !inlines.is_empty() {
                            out.push(' ');
                        }
                        render_inlines(inlines, out, options);
                        out.push_str("</p>\n");
                        rendered_task_paragraph = true;
                    } else if let Some(ast::Block::HtmlBlock(raw)) = children.first() {
                        out.push_str("<p>");
                        render_task_checkbox(done, out);
                        out.push(' ');
                        out.push_str(raw);
                        out.push_str("</p>\n");
                        rendered_task_paragraph = true;
                    } else {
                        render_task_checkbox(done, out);
                    }
                }

                for (idx, child) in children.iter().enumerate() {
                    if rendered_task_paragraph && idx == 0 {
                        continue;
                    }
                    render_block(child, out, options);
                }
                out.push_str("</li>");
            }
            if *ordered {
                out.push_str("</ol>\n");
            } else {
                out.push_str("</ul>\n");
            }
        }
        ast::Block::ListItem { .. } => {}
        ast::Block::BlockQuote { children } => {
            out.push_str("<blockquote>");
            for child in children {
                render_block(child, out, options);
            }
            out.push_str("</blockquote>\n");
        }
        ast::Block::CodeBlock {
            info,
            content,
            fenced: _,
        } => {
            if let Some(language) = info.as_deref() {
                if !language.is_empty() {
                    out.push_str("<pre><code class=\"language-");
                    out.push_str(&escape_html(language));
                    out.push_str("\">");
                    out.push_str(&escape_html(content));
                    out.push_str("</code></pre>\n");
                    return;
                }
            }
            out.push_str("<pre><code>");
            out.push_str(&escape_html(content));
            out.push_str("</code></pre>\n");
        }
        ast::Block::ThematicBreak => {
            out.push_str("<hr>\n");
        }
        ast::Block::Table {
            aligns,
            header,
            rows,
        } => {
            out.push_str("<table><thead><tr>");
            for (idx, cell) in header.iter().enumerate() {
                out.push_str("<th");
                render_table_align_attr(aligns.get(idx).copied().flatten(), out);
                out.push('>');
                render_inlines(cell, out, options);
                out.push_str("</th>");
            }
            out.push_str("</tr></thead><tbody>");
            for row in rows {
                out.push_str("<tr>");
                for (idx, cell) in row.iter().enumerate() {
                    out.push_str("<td");
                    render_table_align_attr(aligns.get(idx).copied().flatten(), out);
                    out.push('>');
                    render_inlines(cell, out, options);
                    out.push_str("</td>");
                }
                out.push_str("</tr>");
            }
            out.push_str("</tbody></table>\n");
        }
        ast::Block::HtmlBlock(raw) => {
            out.push_str(raw);
            out.push('\n');
        }
    }
}

fn render_task_checkbox(done: bool, out: &mut String) {
    out.push_str("<input type=\"checkbox\"");
    if done {
        out.push_str(" checked=\"\"");
    }
    out.push_str(" disabled=\"\">");
}

fn render_tight_list_separator(next_child: &ast::Block, out: &mut String) {
    match next_child {
        ast::Block::Heading { .. } => out.push(' '),
        _ => out.push('\n'),
    }
}

fn render_inlines(inlines: &[Inline], out: &mut String, options: RenderOptions) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => out.push_str(&escape_html(text)),
            Inline::RawHtml(html) => out.push_str(html),
            Inline::SoftBreak => {
                if options.breaks {
                    out.push_str("<br>\n");
                } else {
                    out.push('\n');
                }
            }
            Inline::HardBreak => out.push_str("<br>\n"),
            Inline::Code(text) => {
                out.push_str("<code>");
                out.push_str(&escape_html(text));
                out.push_str("</code>");
            }
            Inline::Em(children) => {
                out.push_str("<em>");
                render_inlines(children, out, options);
                out.push_str("</em>");
            }
            Inline::Strong(children) => {
                out.push_str("<strong>");
                render_inlines(children, out, options);
                out.push_str("</strong>");
            }
            Inline::Del(children) => {
                out.push_str("<del>");
                render_inlines(children, out, options);
                out.push_str("</del>");
            }
            Inline::Link { label, href, title } => {
                out.push_str("<a href=\"");
                out.push_str(&escape_href_attr(href));
                out.push('"');
                if let Some(title) = title {
                    out.push_str(" title=\"");
                    out.push_str(&escape_html_attr(title));
                    out.push('"');
                }
                out.push('>');
                render_inlines(label, out, options);
                out.push_str("</a>");
            }
            Inline::Image { alt, src, title } => {
                out.push_str("<img src=\"");
                out.push_str(&escape_html_attr(src));
                out.push_str("\" alt=\"");
                out.push_str(&escape_html_attr(&inline_text_content(alt)));
                out.push('"');
                if let Some(title) = title {
                    out.push_str(" title=\"");
                    out.push_str(&escape_html_attr(title));
                    out.push('"');
                }
                out.push('>');
            }
        }
    }
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_html_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_href_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("%22"),
            _ => out.push(ch),
        }
    }
    out
}

fn render_table_align_attr(align: Option<ast::TableAlignment>, out: &mut String) {
    match align {
        Some(ast::TableAlignment::Left) => out.push_str(" align=\"left\""),
        Some(ast::TableAlignment::Center) => out.push_str(" align=\"center\""),
        Some(ast::TableAlignment::Right) => out.push_str(" align=\"right\""),
        None => {}
    }
}

fn inline_text_content(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(text) | Inline::Code(text) | Inline::RawHtml(text) => out.push_str(text),
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            Inline::Em(children) | Inline::Strong(children) | Inline::Del(children) => {
                out.push_str(&inline_text_content(children));
            }
            Inline::Link { label, .. } => out.push_str(&inline_text_content(label)),
            Inline::Image { alt, .. } => out.push_str(&inline_text_content(alt)),
        }
    }
    out
}

pub(crate) fn post_autolink(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    let bytes = input.as_bytes();
    let mut stack: Vec<String> = Vec::new();

    while i < input.len() {
        if bytes[i] == b'<' {
            let tag_end = input[i..]
                .find('>')
                .map(|offset| i + offset + 1)
                .unwrap_or(input.len());
            let tag = &input[i..tag_end];
            update_html_stack(tag, &mut stack);
            out.push_str(tag);
            i = tag_end;
            continue;
        }

        let next_tag = input[i..]
            .find('<')
            .map(|offset| i + offset)
            .unwrap_or(input.len());
        let text = &input[i..next_tag];
        if should_skip_autolink(&stack) {
            out.push_str(text);
        } else {
            out.push_str(&autolink_text_segment(text));
        }
        i = next_tag;
    }

    out
}

fn should_skip_autolink(stack: &[String]) -> bool {
    stack.iter().any(|tag| {
        matches!(
            tag.as_str(),
            "a" | "code" | "pre" | "script" | "style" | "textarea"
        )
    })
}

fn autolink_text_segment(text: &str) -> String {
    use linkify::{LinkFinder, LinkKind};

    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url, LinkKind::Email]);
    finder.url_must_have_scheme(false);

    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;

    for link in finder.links(text) {
        let start = link.start();
        let end = link.end();
        out.push_str(&text[last..start]);

        let mut linked_text = text[start..end].to_string();
        if linked_text.ends_with('\"') || linked_text.ends_with('\'') {
            linked_text.pop();
        }

        match link.kind() {
            LinkKind::Url => {
                if is_non_marked_bare_url(&linked_text) {
                    out.push_str(&linked_text);
                } else {
                    let href = if has_scheme(&linked_text) {
                        linked_text.to_string()
                    } else {
                        format!("http://{linked_text}")
                    };
                    out.push_str("<a href=\"");
                    out.push_str(&escape_href_attr(&href));
                    out.push_str("\">");
                    out.push_str(&linked_text);
                    out.push_str("</a>");
                }
            }
            LinkKind::Email => {
                out.push_str("<a href=\"mailto:");
                out.push_str(&escape_href_attr(&linked_text));
                out.push_str("\">");
                out.push_str(&linked_text);
                out.push_str("</a>");
            }
            _ => out.push_str(&linked_text),
        }

        last = end;
    }

    out.push_str(&text[last..]);
    out
}

fn has_scheme(text: &str) -> bool {
    text.split_once("://").is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

fn is_non_marked_bare_url(text: &str) -> bool {
    if has_scheme(text) {
        return false;
    }
    !text.starts_with("www.")
}

fn is_void_tag(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn parse_tag_name(tag_body: &str) -> Option<String> {
    let mut chars = tag_body.chars().peekable();
    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }

    if matches!(chars.peek(), Some('/')) {
        chars.next();
    }

    let mut name = String::new();
    while let Some(c) = chars.peek().copied() {
        if c.is_whitespace() || c == '/' || c == '>' {
            break;
        }
        name.push(c.to_ascii_lowercase());
        chars.next();
    }

    if name.is_empty() { None } else { Some(name) }
}

fn update_html_stack(tag: &str, stack: &mut Vec<String>) {
    if tag.starts_with("<!--") || tag.starts_with("<!") || tag.starts_with("<?") {
        return;
    }

    let Some(name) = parse_tag_name(tag.trim_start_matches('<').trim_end_matches('>')) else {
        return;
    };

    let trimmed = tag.trim();
    let is_end_tag = trimmed.starts_with("</");
    let self_closing = trimmed.ends_with("/>");

    if is_end_tag {
        if let Some(pos) = stack.iter().rposition(|n| n == &name) {
            stack.drain(pos..);
        }
        return;
    }

    if !self_closing && !is_void_tag(&name) {
        stack.push(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_image_title_without_duplication() {
        let node = ast::Block::Paragraph {
            inlines: vec![ast::inline::Inline::Image {
                alt: vec![ast::inline::Inline::Text("logo".to_string())],
                src: "logo.png".to_string(),
                title: Some("Markec logo".to_string()),
            }],
        };

        let mut out = String::new();
        render_block(&node, &mut out, RenderOptions::default());
        assert_eq!(
            out,
            "<p><img src=\"logo.png\" alt=\"logo\" title=\"Markec logo\"></p>\n"
        );
    }

    #[test]
    fn renders_ordered_list_start_when_non_one() {
        let node = ast::Block::List {
            ordered: true,
            start: 2,
            tight: true,
            items: vec![ast::ListItem {
                children: vec![ast::Block::Paragraph {
                    inlines: vec![ast::inline::Inline::Text("two".to_string())],
                }],
                task: None,
            }],
        };

        let mut out = String::new();
        render_block(&node, &mut out, RenderOptions::default());
        assert_eq!(out, "<ol start=\"2\"><li>two</li></ol>\n");
    }
}
