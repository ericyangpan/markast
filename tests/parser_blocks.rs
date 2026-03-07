use markec::{RenderOptions, render_markdown_to_html};

#[test]
fn parser_blocks_parenthesized_ordered_lists_render() {
    let html = render_markdown_to_html("1) first\n2) second", RenderOptions::default());

    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>first</li>"));
    assert!(html.contains("<li>second</li>"));
}

#[test]
fn parser_blocks_preserve_empty_table_body_cells() {
    let md = "| a | b | c |\n| --- | --- | --- |\n| 1 |   | 3 |";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<table>"));
    assert!(html.contains("<td></td>"));
}

#[test]
fn parser_blocks_support_single_column_tables_without_pipes() {
    let md = "table\n:----\nvalue\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<table>"));
    assert!(html.contains("<th align=\"left\">table</th>"));
    assert!(html.contains("<td align=\"left\">value</td>"));
}

#[test]
fn parser_blocks_prefer_setext_over_pipe_header_and_plain_dash_line() {
    let md = "| setext |\n----------\n| setext |\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<h2>| setext |</h2>"));
    assert!(!html.contains("<table>"));
}

#[test]
fn parser_blocks_disable_tables_in_non_gfm_mode() {
    let md = "| a | b |\n| - | - |\n| 1 | 2 |\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            gfm: false,
            breaks: false,
            pedantic: false,
        },
    );

    assert!(!html.contains("<table>"));
}

#[test]
fn parser_blocks_keep_indented_thematic_breaks_out_of_hr() {
    let html = render_markdown_to_html("    ---", RenderOptions::default());

    assert!(!html.contains("<hr>"));
    assert!(html.contains("<pre>") || html.contains("<p>"));
}

#[test]
fn parser_blocks_render_loose_lists_with_paragraph_wrappers() {
    let md = "- item 1\n-\n  item 2\n\n  still item 2\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<li><p>item 1</p>\n</li>"));
    assert!(html.contains("<li><p>item 2</p>"));
    assert!(html.contains("<p>still item 2</p>"));
}

#[test]
fn parser_blocks_end_list_items_before_following_top_level_paragraphs() {
    let md = "- ***\nparagraph\n- # heading\nparagraph\n-     indented code\nparagraph\n- ```\n  fenced code\n  ```\nparagraph\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert_eq!(html.matches("<ul>").count(), 4);
    assert_eq!(html.matches("<p>paragraph</p>").count(), 4);
    assert!(html.contains("<li><hr>\n</li></ul>\n<p>paragraph</p>"));
    assert!(html.contains("<li><h1>heading</h1>\n</li></ul>\n<p>paragraph</p>"));
    assert!(html.contains("<li><pre><code>indented code"));
    assert!(html.contains("<li><pre><code>fenced code"));
}

#[test]
fn parser_blocks_keep_task_markers_literal_when_gfm_is_disabled() {
    let html = render_markdown_to_html(
        "- [ ] A\n- [x] B\n- [ ] C\n",
        RenderOptions {
            gfm: false,
            breaks: false,
            pedantic: false,
        },
    );

    assert!(!html.contains("checkbox"));
    assert!(html.contains("[ ] A"));
    assert!(html.contains("[x] B"));
}

#[test]
fn parser_blocks_end_nested_list_items_before_parent_blockquotes() {
    let md = "- list item\n  - nested list item\n  > quoteblock\n\n- list item\n  - nested list item\n> quote block\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<ul><li>nested list item</li></ul>\n<blockquote>"));
    assert!(html.contains("</ul>\n</li></ul>\n<blockquote><p>quote block</p>"));
}

#[test]
fn parser_blocks_split_unordered_lists_when_marker_changes() {
    let md = "* alpha\n- beta\n+ gamma\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert_eq!(html.matches("<ul>").count(), 3);
    assert!(html.contains("<ul><li>alpha</li></ul>\n<ul><li>beta</li></ul>\n<ul><li>gamma</li></ul>"));
}

#[test]
fn parser_blocks_keep_inline_html_inside_paragraphs_for_setext() {
    let md = "<b>heading</b>\n-----\n\n<s>not heading</s>\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<h2><b>heading</b></h2>"));
    assert!(html.contains("<p><s>not heading</s></p>"));
}

#[test]
fn parser_blocks_merge_mixed_bullets_in_pedantic_mode() {
    let html = render_markdown_to_html(
        "* alpha\n+ beta\n- gamma\n",
        RenderOptions {
            gfm: true,
            breaks: false,
            pedantic: true,
        },
    );

    assert_eq!(html.matches("<ul>").count(), 1);
    assert!(html.contains("<li>alpha</li>"));
    assert!(html.contains("<li>beta</li>"));
    assert!(html.contains("<li>gamma</li>"));
}

#[test]
fn parser_blocks_keep_multiline_html_open_tags_as_blocks() {
    let md = "<div id=\"foo\"\n  class=\"bar\">\n</div>\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(!html.contains("<p><div"));
    assert!(html.contains("<div id=\"foo\"\n  class=\"bar\">\n</div>"));
}

#[test]
fn parser_blocks_do_not_treat_indented_code_as_setext_heading_text() {
    let md = "# Heading\n    foo\nHeading\n------\n    foo\n----\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<h1>Heading</h1>"));
    assert!(html.contains("<h2>Heading</h2>"));
    assert_eq!(html.matches("<pre><code>foo").count(), 2);
    assert!(html.contains("<hr>"));
}

#[test]
fn parser_blocks_keep_blockquote_indentation_for_list_continuations() {
    let md = "   > > 1.  one\n>>\n>>     two\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<blockquote><blockquote><ol><li><p>one</p>\n<p>two</p>\n</li></ol>\n</blockquote>\n</blockquote>"));
}

#[test]
fn parser_blocks_strip_fence_indent_from_list_code_contents() {
    let md = "1. item\n\n\t```\n\tconst x = 5;\n\tconst y = x + 5;\n\t```\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<pre><code>const x = 5;\nconst y = x + 5;\n</code></pre>"));
    assert!(!html.contains("<pre><code> const x = 5;"));
}

#[test]
fn parser_blocks_treat_tab_after_blockquote_marker_as_marker_padding() {
    let html = render_markdown_to_html(">\ttest\n", RenderOptions::default());

    assert_eq!(html, "<blockquote><p>test</p>\n</blockquote>\n");
}

#[test]
fn parser_blocks_keep_html_blocks_open_until_blank_line() {
    let md = "</div>\n*foo*\n\n<div></div>\n``` c\nint x = 33;\n```\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("</div>\n*foo*"));
    assert!(html.contains("<div></div>\n``` c\nint x = 33;\n```"));
}

#[test]
fn parser_blocks_support_processing_instruction_and_cdata_html_blocks() {
    let md = "<?php\n\necho '>';\n\n?>\nokay\n\n<!DOCTYPE html>\n\n<![CDATA[\ncontent\n]]>\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<?php\n\necho '>';\n\n?>"));
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<![CDATA[\ncontent\n]]>"));
    assert!(html.contains("<p>okay</p>"));
}

#[test]
fn parser_blocks_decode_entities_in_fenced_code_info_string() {
    let html = render_markdown_to_html("``` f&ouml;&ouml;\nbody\n```\n", RenderOptions::default());

    assert!(html.contains("<code class=\"language-föö\">body\n</code>"));
}

#[test]
fn parser_blocks_trim_setext_heading_content_indent_and_trailing_space() {
    let md = "  Foo *bar\nbaz*\t\n====\n\nFoo  \n-----\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            gfm: false,
            ..RenderOptions::default()
        },
    );

    assert!(html.contains("<h1>Foo <em>bar\nbaz</em></h1>"));
    assert!(html.contains("<h2>Foo</h2>"));
}

#[test]
fn parser_blocks_allow_single_equals_setext_and_break_unquoted_hr_after_blockquote() {
    let md = "Foo\n=\n\n> Foo\n---\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            gfm: false,
            ..RenderOptions::default()
        },
    );

    assert!(html.contains("<h1>Foo</h1>"));
    assert!(html.contains("<blockquote><p>Foo</p>\n</blockquote>\n<hr>"));
}

#[test]
fn parser_blocks_extend_tables_with_implicit_tail_rows() {
    let md = "| abc | def |\n| --- | --- |\n| bar | foo |\nhello\n**strong**\n`code`\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<tr><td>hello</td><td></td></tr>"));
    assert!(html.contains("<tr><td><strong>strong</strong></td><td></td></tr>"));
    assert!(html.contains("<tr><td><code>code</code></td><td></td></tr>"));
    assert!(!html.contains("</table><p>hello</p>"));
}

#[test]
fn parser_blocks_stop_table_before_following_blocks() {
    let md = "| abc | def |\n| --- | --- |\n| bar | foo |\n# heading\n> quote\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("</table>\n<h1>heading</h1>"));
    assert!(html.contains("<blockquote><p>quote</p>\n</blockquote>"));
}

#[test]
fn parser_blocks_stop_table_before_following_fences_without_extra_code_newline() {
    let with_pipes = "| abc | def |\n| --- | --- |\n| bar | foo |\n| baz | boo |\n```\nfoobar()\n```\n";
    let no_pipes = " abc | def\n --- | ---\n bar | foo\n baz | boo\n```\nfoobar()\n```\n";

    let html_with_pipes = render_markdown_to_html(with_pipes, RenderOptions::default());
    let html_no_pipes = render_markdown_to_html(no_pipes, RenderOptions::default());

    assert!(html_with_pipes.contains("</table>\n<pre><code>foobar()</code></pre>"));
    assert!(html_no_pipes.contains("</table>\n<pre><code>foobar()</code></pre>"));
}

#[test]
fn parser_blocks_treat_nbsp_tail_after_table_as_empty_row() {
    let md = "| abc | def |\n| --- | --- |\n| bar | foo |\n\u{00A0}\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<tr><td></td><td></td></tr>"));
}

#[test]
fn parser_blocks_accept_space_then_tab_after_list_marker() {
    let md = "1. \tSomeText\n2. \tSomeText\n\n- \tSomeText\n- \tSomeText\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<ol><li>SomeText</li><li>SomeText</li></ol>"));
    assert!(html.contains("<ul><li>SomeText</li><li>SomeText</li></ul>"));
    assert!(!html.contains("<pre><code>SomeText"));
}

#[test]
fn parser_blocks_keep_tab_indented_list_paragraphs_inside_items() {
    let md = "1.\tFirst\n\n\tSecond paragraph\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert!(html.contains("<ol><li><p>First</p>\n<p>Second paragraph</p>\n</li></ol>"));
    assert!(!html.contains("<pre><code>Second paragraph"));
}

#[test]
fn parser_blocks_keep_tab_indented_nested_lists_nested() {
    let md = "*\tTab\n\t*\tTab\n\t\t*\tTab\n";
    let html = render_markdown_to_html(md, RenderOptions::default());

    assert_eq!(html.matches("<ul>").count(), 3);
    assert_eq!(html.matches("<li>Tab").count(), 3);
    assert!(!html.contains("<pre><code>"));
}

#[test]
fn parser_blocks_keep_pedantic_text_after_nested_list_inside_item() {
    let md = "  * item1\n\n    * item2\n\n  text\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            pedantic: true,
            ..RenderOptions::default()
        },
    );

    assert!(html.contains("<ul><li><p>item1</p>"));
    assert!(html.contains("<ul><li>item2</li></ul>"));
    assert!(html.contains("<p>text</p>\n</li></ul>"));
    assert!(!html.ends_with("<p>text</p>\n"));
}

#[test]
fn parser_blocks_parse_pedantic_hash_headings_without_space() {
    let md = "#h1\n\n#h1#\n\n#h1 # #\n\n#h1####\n\n # h1\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            pedantic: true,
            ..RenderOptions::default()
        },
    );

    assert!(html.contains("<h1>h1</h1>"));
    assert!(html.contains("<h1>h1 #</h1>"));
    assert!(html.contains("<p># h1</p>"));
}

#[test]
fn parser_blocks_pedantic_hash_heading_interrupts_paragraph() {
    let md = "paragraph before head with hash\n#how are you\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            pedantic: true,
            ..RenderOptions::default()
        },
    );

    assert!(html.contains("<p>paragraph before head with hash</p>\n<h1>how are you</h1>"));
}

#[test]
fn parser_blocks_pedantic_setext_heading_does_not_absorb_previous_paragraph_line() {
    let md = "paragraph before head with equals\nhow are you again\n===========\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            pedantic: true,
            ..RenderOptions::default()
        },
    );

    assert!(html.contains("<p>paragraph before head with equals</p>\n<h1>how are you again</h1>"));
}

#[test]
fn parser_blocks_parse_pedantic_headings_without_gfm() {
    let md = "#header\n\n# header1\n\n#  header2\n";
    let html = render_markdown_to_html(
        md,
        RenderOptions {
            gfm: false,
            pedantic: true,
            ..RenderOptions::default()
        },
    );

    assert_eq!(html.matches("<h1>").count(), 3);
    assert!(html.contains("<h1>header</h1>"));
    assert!(html.contains("<h1>header1</h1>"));
    assert!(html.contains("<h1>header2</h1>"));
}
