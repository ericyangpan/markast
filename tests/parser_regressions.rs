use std::fs;
use std::path::PathBuf;

use markrs::{RenderOptions, render_markdown_to_html};
use serde_json::Value;

fn compat_fixture_pair(name: &str) -> (String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let md = fs::read_to_string(root.join("third_party/marked/test/specs/new").join(format!("{name}.md")))
        .unwrap_or_else(|e| panic!("failed reading markdown fixture {name}: {e}"));
    let html =
        fs::read_to_string(root.join("third_party/marked/test/specs/new").join(format!("{name}.html")))
            .unwrap_or_else(|e| panic!("failed reading html fixture {name}: {e}"));
    (md, html)
}

fn commonmark_example_pair(example: u64) -> (String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raw = fs::read_to_string(
        root.join("third_party/marked/test/specs/commonmark/commonmark.0.31.2.json"),
    )
    .unwrap_or_else(|e| panic!("failed reading commonmark fixture index: {e}"));
    let examples: Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("invalid commonmark json: {e}"));
    let list = examples
        .as_array()
        .unwrap_or_else(|| panic!("commonmark fixture root is not an array"));
    let entry = list
        .iter()
        .find(|row| row.get("example").and_then(Value::as_u64) == Some(example))
        .unwrap_or_else(|| panic!("missing commonmark example {example}"));
    let markdown = entry
        .get("markdown")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing markdown for example {example}"))
        .to_string();
    let html = entry
        .get("html")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing html for example {example}"))
        .to_string();
    (markdown, html)
}

fn render_compat_fixture(markdown: &str) -> String {
    let mut options = RenderOptions::default();
    let body = strip_marked_front_matter(markdown, &mut options);
    render_markdown_to_html(&body, options)
}

fn strip_marked_front_matter(markdown: &str, options: &mut RenderOptions) -> String {
    let Some(rest) = markdown.strip_prefix("---\n") else {
        return markdown.to_string();
    };
    let Some(end) = rest.find("\n---\n") else {
        return markdown.to_string();
    };

    let header = &rest[..end];
    for line in header.lines() {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim();
        let val = v.trim();
        if key == "gfm" {
            options.gfm = val == "true";
        }
        if key == "breaks" {
            options.breaks = val == "true";
        }
        if key == "pedantic" {
            options.pedantic = val == "true";
        }
    }

    rest[end + "\n---\n".len()..].to_string()
}

fn normalize_html(input: &str) -> String {
    let collapsed = input
        .replace("<br />", "<br>")
        .replace("<br/>", "<br>")
        .replace("<hr />", "<hr>")
        .replace("<hr/>", "<hr>")
        .replace(" />", ">")
        .replace("\n<blockquote>", "<blockquote>")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    collapsed.replace("> <", "><")
}

#[test]
fn compat_em_strong_complex_nesting_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("em_strong_complex_nesting");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_escape_newline_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("escape_newline");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_em_strong_orphaned_nesting_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("em_strong_orphaned_nesting");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_nested_em_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("nested_em");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_emoji_inline_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("emoji_inline");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_strikethrough_in_em_strong_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("strikethrough_in_em_strong");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_list_loose_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("list_loose");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_table_vs_setext_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("table_vs_setext");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
#[ignore = "known compat gap: html-block classification still pending"]
fn compat_incorrectly_formatted_list_and_hr_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("incorrectly_formatted_list_and_hr");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_nested_blockquote_in_list_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("nested_blockquote_in_list");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_list_code_header_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("list_code_header");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_pedantic_heading_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("pedantic_heading");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_pedantic_heading_interrupts_paragraph_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("pedantic_heading_interrupts_paragraph");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_nogfm_hashtag_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("nogfm_hashtag");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_link_lt_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("link_lt");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_def_blocks_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("def_blocks");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_definition_multiline_title_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(196);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_definition_unicode_label_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(206);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_image_alt_text_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(573);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_autolink_scheme_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(598);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_definition_backslashes_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(202);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_definition_does_not_interrupt_paragraph_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(213);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_blockquote_reference_definition_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(218);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_link_destination_entities_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(503);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_link_title_variants_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(505);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_blockquote_list_continuation_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(259);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_tab_after_blockquote_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("tab_after_blockquote");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_inline_processing_instruction_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(627);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_inline_declaration_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(628);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_inline_cdata_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(629);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_html_block_closing_tag_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(151);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_html_block_processing_instruction_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(180);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_html_block_interrupts_paragraph_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(185);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_entity_references_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(25);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_numeric_entity_references_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(26);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_entities_do_not_trigger_emphasis_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(37);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_entities_do_not_trigger_list_markers_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(38);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_entities_preserve_literal_newlines_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(39);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_entities_decode_in_fenced_info_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(34);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_setext_multiline_heading_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(82);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_setext_single_equals_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(83);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_setext_trims_trailing_space_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(89);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_unquoted_thematic_break_does_not_stay_in_blockquote_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(92);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_unquoted_thematic_break_after_blockquote_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(101);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_lheading_following_table_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("lheading_following_table");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_inlinecode_following_tables_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("inlinecode_following_tables");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_text_following_tables_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("text_following_tables");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_nbsp_following_tables_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("nbsp_following_tables");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_fences_following_table_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("fences_following_table");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_fences_following_nptable_matches_marked() {
    let (markdown, expected) = compat_fixture_pair("fences_following_nptable");
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_autolink_scheme_length_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(609);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_label_casefold_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(540);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_link_does_not_cross_line_between_labels_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(543);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_first_reference_definition_wins_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(544);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_collapsed_reference_link_does_not_cross_line_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(556);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_invalid_reference_label_with_open_bracket_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(546);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_invalid_reference_label_with_nested_brackets_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(547);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_invalid_reference_definition_for_nested_shortcut_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(548);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_empty_reference_label_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(552);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_nested_inline_link_rejects_outer_link_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(518);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_nested_inline_image_alt_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(520);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_raw_html_does_not_close_link_label_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(524);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_autolink_does_not_close_link_label_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(526);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_reference_outer_link_rejects_inner_link_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(532);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}

#[test]
fn compat_commonmark_space_between_reference_labels_is_not_allowed_matches_marked() {
    let (markdown, expected) = commonmark_example_pair(542);
    let actual = render_compat_fixture(&markdown);

    assert_eq!(normalize_html(&actual), normalize_html(&expected));
}
