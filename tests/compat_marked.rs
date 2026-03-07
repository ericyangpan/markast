use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use globset::{Glob, GlobSet, GlobSetBuilder};
use markec::{RenderOptions, render_markdown_to_html};
use regex::Regex;
use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Debug)]
struct CompatCase {
    id: String,
    markdown: String,
    expected_html: String,
    options: RenderOptions,
}

#[derive(Debug, Deserialize)]
struct JsonSpecCase {
    markdown: String,
    html: String,
    #[allow(dead_code)]
    example: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct XfailConfig {
    #[serde(default)]
    cases: Vec<XfailCase>,
    #[serde(default)]
    patterns: Vec<XfailPattern>,
}

#[derive(Debug, Deserialize)]
struct XfailCase {
    id: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct XfailPattern {
    pattern: String,
    #[allow(dead_code)]
    reason: String,
}

static WS_BETWEEN_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r">\s+<").expect("invalid regex"));
static XHTML_VOID_SLASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(br|hr|img|input)([^>]*?)\s*/?\s*>").expect("invalid regex"));
static BR_FOLLOWED_BY_NEWLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<br>\s*\n+").expect("invalid regex"));
static TEXT_BEFORE_BLOCKQUOTE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([^\s>])\s+<blockquote>").expect("invalid regex"));
static INPUT_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<input([^>]*)>").expect("invalid regex"));

fn normalize_html(input: &str) -> String {
    let compact = WS_BETWEEN_TAGS.replace_all(input, "><");
    let normalized_void = XHTML_VOID_SLASH.replace_all(&compact, "<$1$2>");
    let normalized_break = BR_FOLLOWED_BY_NEWLINE.replace_all(&normalized_void, "<br>");
    let normalized_blockquote = TEXT_BEFORE_BLOCKQUOTE.replace_all(&normalized_break, "$1<blockquote>");
    let normalized_input = normalize_input_attrs(&normalized_blockquote);
    normalized_input
        .replace("\r\n", "\n")
        // Treat equivalent entity spellings as equal for compat checks.
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&#x22;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&gt;", ">")
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn normalize_input_attrs(input: &str) -> String {
    INPUT_TAG
        .replace_all(input, |caps: &regex::Captures<'_>| {
            let attrs = parse_attrs(caps.get(1).map(|m| m.as_str()).unwrap_or_default());
            if attrs.is_empty() {
                return "<input>".to_string();
            }
            let mut sorted = attrs;
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = String::from("<input");
            for (name, value) in sorted {
                out.push(' ');
                out.push_str(&name);
                if let Some(v) = value {
                    out.push_str("=\"");
                    out.push_str(&v);
                    out.push('"');
                }
            }
            out.push('>');
            out
        })
        .to_string()
}

fn parse_attrs(raw: &str) -> Vec<(String, Option<String>)> {
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        let name_start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '=' {
            i += 1;
        }
        if name_start == i {
            i += 1;
            continue;
        }
        let name: String = chars[name_start..i].iter().collect();

        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        let mut value = None;
        if i < chars.len() && chars[i] == '=' {
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                let val_start = i;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                value = Some(chars[val_start..i].iter().collect());
                if i < chars.len() && chars[i] == quote {
                    i += 1;
                }
            } else {
                let val_start = i;
                while i < chars.len() && !chars[i].is_whitespace() {
                    i += 1;
                }
                value = Some(chars[val_start..i].iter().collect());
            }
        }

        out.push((name, value));
    }

    out
}

fn should_use_gfm(case_id: &str) -> bool {
    if case_id.contains("/commonmark/") {
        return false;
    }
    if case_id.contains("_nogfm") {
        return false;
    }
    true
}

fn strip_marked_front_matter(
    markdown: &str,
    mut options: RenderOptions,
) -> (String, RenderOptions) {
    let Some(rest) = markdown.strip_prefix("---\n") else {
        return (markdown.to_string(), options);
    };
    let Some(end) = rest.find("\n---\n") else {
        return (markdown.to_string(), options);
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

    let body = &rest[end + "\n---\n".len()..];
    (body.to_string(), options)
}

fn collect_md_html_cases(specs_root: &Path) -> Vec<CompatCase> {
    let mut cases = Vec::new();

    for entry in WalkDir::new(specs_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let html_path = path.with_extension("html");
        if !html_path.exists() {
            continue;
        }

        let rel = path
            .strip_prefix(specs_root.parent().unwrap_or(specs_root))
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let markdown_raw = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed reading {}: {e}", path.display()));
        let expected_html = fs::read_to_string(&html_path)
            .unwrap_or_else(|e| panic!("failed reading {}: {e}", html_path.display()));
        let default_options = RenderOptions {
            gfm: should_use_gfm(&rel),
            breaks: false,
            pedantic: false,
        };
        let (markdown, options) = strip_marked_front_matter(&markdown_raw, default_options);

        cases.push(CompatCase {
            id: rel.clone(),
            markdown,
            expected_html,
            options,
        });
    }

    cases
}

fn collect_json_cases(json_path: &Path, root_for_id: &Path) -> Vec<CompatCase> {
    let rel_base = json_path
        .strip_prefix(root_for_id)
        .unwrap_or(json_path)
        .to_string_lossy()
        .replace('\\', "/");

    let content = fs::read_to_string(json_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {e}", json_path.display()));
    let list: Vec<JsonSpecCase> = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("invalid JSON {}: {e}", json_path.display()));

    let gfm = should_use_gfm(&rel_base);

    list.into_iter()
        .enumerate()
        .map(|(idx, item)| CompatCase {
            id: format!("{rel_base}#example-{}", idx + 1),
            markdown: item.markdown,
            expected_html: item.html,
            options: RenderOptions {
                gfm,
                breaks: false,
                pedantic: false,
            },
        })
        .collect()
}

fn collect_all_compat_cases(repo_root: &Path) -> Vec<CompatCase> {
    let specs_root = repo_root.join("third_party/marked/test/specs");
    assert!(
        specs_root.exists(),
        "missing third_party marked specs: {}",
        specs_root.display()
    );

    let mut cases = collect_md_html_cases(&specs_root.join("new"));
    cases.extend(collect_md_html_cases(&specs_root.join("original")));

    for json_file in [
        specs_root.join("commonmark/commonmark.0.31.2.json"),
        specs_root.join("gfm/commonmark.0.31.2.json"),
        specs_root.join("gfm/gfm.0.29.json"),
    ] {
        if json_file.exists() {
            cases.extend(collect_json_cases(
                &json_file,
                &repo_root.join("third_party/marked"),
            ));
        }
    }

    cases.sort_by(|a, b| a.id.cmp(&b.id));
    cases
}

fn load_xfail_config(path: &Path) -> XfailConfig {
    if !path.exists() {
        return XfailConfig::default();
    }

    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed reading {}: {e}", path.display()));
    serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("invalid yaml {}: {e}", path.display()))
}

fn build_pattern_matcher(patterns: &[XfailPattern]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(&pattern.pattern)
            .unwrap_or_else(|e| panic!("invalid xfail pattern '{}': {e}", pattern.pattern));
        builder.add(glob);
    }
    builder.build().expect("failed building xfail glob set")
}

fn write_xfail(path: &Path, failing_ids: &[String]) {
    let mut out = String::new();
    out.push_str("# Auto-generated baseline for marked compatibility mismatches.\n");
    out.push_str(
        "# Update with: MARKEC_WRITE_XFAIL=1 cargo test --test compat_marked -- --nocapture\n",
    );
    out.push_str("cases:\n");

    for id in failing_ids {
        out.push_str("  - id: \"");
        out.push_str(id);
        out.push_str("\"\n");
        out.push_str("    reason: \"baseline mismatch vs marked\"\n");
    }

    out.push_str("patterns: []\n");
    fs::write(path, out).unwrap_or_else(|e| panic!("failed writing {}: {e}", path.display()));
}

#[test]
fn marked_compatibility_suite() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let xfail_path = repo_root.join("tests/compat/xfail.yaml");
    let ignore_xfail = std::env::var("MARKEC_IGNORE_XFAIL").ok().as_deref() == Some("1");
    let print_diffs = std::env::var("MARKEC_PRINT_DIFFS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let cases = collect_all_compat_cases(&repo_root);
    assert!(!cases.is_empty(), "no marked compatibility cases found");

    let xfail = load_xfail_config(&xfail_path);
    let pattern_matcher = build_pattern_matcher(&xfail.patterns);
    let exact: HashMap<&str, &str> = xfail
        .cases
        .iter()
        .map(|c| (c.id.as_str(), c.reason.as_str()))
        .collect();

    let mut failures = Vec::new();
    let mut xfailed = Vec::new();
    let mut recovered = Vec::new();
    let mut mismatch_samples: Vec<(String, String, String, bool)> = Vec::new();

    for case in &cases {
        let actual = render_markdown_to_html(&case.markdown, case.options);
        let normalized_actual = normalize_html(&actual);
        let normalized_expected = normalize_html(&case.expected_html);
        let ok = normalized_actual == normalized_expected;

        let exact_xfail = exact.get(case.id.as_str()).copied();
        let pattern_xfail = pattern_matcher.is_match(case.id.as_str());
        let is_xfail = !ignore_xfail && (exact_xfail.is_some() || pattern_xfail);

        if ok {
            if exact_xfail.is_some() || pattern_xfail {
                recovered.push(case.id.clone());
            }
            continue;
        }

        if print_diffs > 0 && mismatch_samples.len() < print_diffs {
            mismatch_samples.push((
                case.id.clone(),
                normalized_expected,
                normalized_actual,
                is_xfail,
            ));
        }

        if is_xfail {
            xfailed.push(case.id.clone());
        } else {
            failures.push(case.id.clone());
        }
    }

    if std::env::var("MARKEC_WRITE_XFAIL").ok().as_deref() == Some("1") {
        let mut baseline = failures.clone();
        baseline.extend(xfailed.clone());
        baseline.sort();
        baseline.dedup();
        write_xfail(&xfail_path, &baseline);
        eprintln!(
            "wrote {} baseline xfail entries to {}",
            baseline.len(),
            xfail_path.display()
        );
        return;
    }

    if !mismatch_samples.is_empty() {
        eprintln!("compat mismatch samples:");
        for (id, expected, actual, is_xfail) in &mismatch_samples {
            let state = if *is_xfail { "xfail" } else { "fail" };
            eprintln!("--- [{state}] {id}");
            eprintln!("expected: {expected}");
            eprintln!("actual  : {actual}");
        }
    }

    // Surface stale xfail entries (exact ids only).
    let case_ids: BTreeSet<&str> = cases.iter().map(|c| c.id.as_str()).collect();
    let stale_xfail: Vec<String> = xfail
        .cases
        .iter()
        .filter(|x| !case_ids.contains(x.id.as_str()))
        .map(|x| x.id.clone())
        .collect();

    let mut report = String::new();

    if !failures.is_empty() {
        report.push_str("\nnew compat failures (not in xfail):\n");
        for id in failures.iter().take(40) {
            report.push_str("  - ");
            report.push_str(id);
            report.push('\n');
        }
        if failures.len() > 40 {
            report.push_str("  ...\n");
        }
    }

    if !recovered.is_empty() {
        report.push_str("\nxfail recovered (should be removed):\n");
        for id in recovered.iter().take(40) {
            report.push_str("  - ");
            report.push_str(id);
            report.push('\n');
        }
        if recovered.len() > 40 {
            report.push_str("  ...\n");
        }
    }

    if !stale_xfail.is_empty() {
        report.push_str("\nstale xfail ids (fixture missing):\n");
        for id in stale_xfail.iter().take(40) {
            report.push_str("  - ");
            report.push_str(id);
            report.push('\n');
        }
        if stale_xfail.len() > 40 {
            report.push_str("  ...\n");
        }
    }

    if !report.is_empty() {
        panic!(
            "marked compatibility check failed.\n{}
summary: total_cases={}, xfailed={}, new_failures={}, recovered={}, stale_xfail={}\n\nIf baseline changed intentionally, refresh with:\nMARKEC_WRITE_XFAIL=1 cargo test --test compat_marked -- --nocapture",
            report,
            cases.len(),
            xfailed.len(),
            failures.len(),
            recovered.len(),
            stale_xfail.len(),
        );
    }
}
