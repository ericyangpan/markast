# P0 Parser Detailed Design

Last updated: 2026-03-15

## 1. Objective

Keep evolving the in-house Rust Markdown parser for `markast` without changing the public rendering contract.

Public contract to preserve:

- `render_markdown_to_html(input: &str, options: RenderOptions) -> String`
- `render_markdown_to_html_buf(input: &str, options: RenderOptions, buf: &mut String)`
- `RenderOptions { gfm, breaks, pedantic }` semantics
- Existing own tests and marked compatibility tests remain the release gate

## 2. Non-goals (P0)

- HTML sanitize pipeline integration (P1)
- WASM runtime packaging and fallback loader (P2)
- Full AST public API stability (internal-only AST is fine in P0)

## 3. Current Baseline

- Engine now: in-house `markdown` parser modules (`src/markdown/*`)
- Marked compatibility test harness already in place
- `xfail` baselines exist and are intentionally mutable only when behavior changes are verified

Baseline freeze rule:

- Before every milestone cut, run:
  - `npm run test:compat`
  - `npm run test:own`
  - record `xfail` count delta with reason category

## 4. Architecture

Code layout target:

- `src/markdown/mod.rs`
- `src/markdown/options.rs`
- `src/markdown/span.rs`
- `src/markdown/ast.rs`
- `src/markdown/lexer.rs`
- `src/markdown/block.rs`
- `src/markdown/inline.rs`
- `src/markdown/render.rs`
- `src/markdown/render_html.rs`
- `src/markdown/tests/*` (unit-level parser tests)

Integration boundary:

- `src/lib.rs` calls `markdown::render_markdown_to_html(input, options)`
- CLI stays unchanged

## 5. Data Model

### 5.1 Source Span

Purpose:

- Accurate debug output
- Easier compat diff localization
- Shared parser bookkeeping without exposing spans publicly

Suggested structure:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

Spans now live in `src/markdown/span.rs`.

### 5.2 Block AST

Suggested block node set:

- `Document { children }`
- `Paragraph { inlines }`
- `Heading { level, inlines }`
- `BlockQuote { children }`
- `List { ordered, start, tight, items }`
- `ListItem { children }`
- `CodeBlock { fenced, info, text }`
- `ThematicBreak`
- `Table { aligns, head, rows }` (GFM)
- `HtmlBlock { raw }`
- `LinkReferenceDef { label, destination, title }` (kept for resolution phase)

### 5.3 Inline AST

Suggested inline node set:

- `Text`
- `SoftBreak`
- `HardBreak`
- `CodeSpan`
- `Emphasis`
- `Strong`
- `Strikethrough` (GFM)
- `Link { href, title, children }`
- `Image { src, title, alt }`
- `HtmlInline { raw }`

## 6. Parsing Pipeline

Pipeline stages:

1. Normalize source newlines (`\r\n` -> `\n`)
2. Block parse to block AST
3. Collect and resolve link reference definitions
4. Inline parse paragraph/heading/list-item inline text
5. Render HTML

## 7. Block Parser Design

### 7.1 Core strategy

- Use a line cursor with byte offsets
- Maintain container stack (`blockquote`, `list`, `list-item`)
- Implement block start precedence explicitly

### 7.2 Precedence (high-level)

Evaluation order per candidate line:

1. Container continuation checks
2. ATX heading
3. Fenced code start/continuation/end
4. Thematic break
5. List marker start/continuation
6. Blockquote marker
7. Table start (GFM only, with strict header+delimiter validation)
8. Setext heading upgrade
9. HTML block rules
10. Paragraph continuation/new paragraph

### 7.3 Critical behaviors to encode

- Lazy continuation lines in blockquotes/lists
- Tight vs loose list detection
- Indented code interaction inside lists
- Table interruption vs paragraph/setext ambiguity
- Thematic break vs list item ambiguity

## 8. Inline Parser Design

### 8.1 Scanner

- Single forward scan producing token stream
- Track:
  - delimiter runs (`*`, `_`, `~`)
  - brackets (`[`, `![`)
  - backticks
  - escapes
  - raw HTML spans

### 8.2 Delimiter algorithm

- Use delimiter stack (open/close flags + length + position)
- Resolve in reverse with CommonMark-like flanking logic
- GFM `~~` enabled only when `options.gfm == true`

### 8.3 Link and image resolution

- Inline link: `[text](dest "title")`
- Reference link:
  - full `[text][label]`
  - collapsed `[label][]`
  - shortcut `[label]`
- Unmatched brackets degrade to text

### 8.4 Autolink handling

- Keep current markast policy:
  - plain `www.` auto-link in GFM mode
  - plain emails auto-link in GFM mode
  - skip inside tags that should not be rewritten
- Keep this behavior isolated from the block parser so parser core remains deterministic

## 9. HTML Renderer Design

Renderer invariants:

- Deterministic tag ordering
- Stable escaping behavior for text and attributes
- No sanitizer concerns in P0

Functions:

- `escape_text`
- `escape_attr`
- `render_block`
- `render_inline`
- `render_document`

Output normalization policy:

- Keep the same general style used today to avoid noise in compat tests

## 10. Marked Compatibility Strategy

Use current harness unchanged as external oracle.

Additional guidance:

- Keep front-matter option extraction in tests (`gfm`, `breaks`, `pedantic`)
- Keep `xfail`, but classify each mismatch category:
  - block structure mismatch
  - inline delimiter mismatch
  - link/ref resolution mismatch
  - formatting-only mismatch

Reduction strategy:

1. Remove high-yield runtime mismatches first
2. Recover snapshot gaps that fall out of the same fixes
3. Leave purely historical snapshot deltas for dedicated cleanup passes

## 11. Cutover Status

The in-house parser is already the default and only active rendering path.

Rendering goes through:

- `markdown::render_markdown_to_html`
- `markdown::render_markdown_to_html_buf`

## 12. Performance Plan

Measure before and after large parser batches.

Benchmark sets:

- Small docs: README-scale
- Medium docs: 10-50 KB markdown
- Large docs: 100-500 KB markdown
- Shared corpora from `third_party/marked/test/specs`

Metrics:

- Throughput (MiB/s)
- trimmed-mean and median render time
- compatibility-preserving performance changes only

Guardrail:

- No drop below the benchmark thresholds documented in `docs/performance.md` without explicit sign-off

## 13. Test Plan

### 13.1 Existing gates

- `tests/own_rendering.rs`
- `tests/compat_snapshot.rs`
- `tests/compat_runtime.rs`

### 13.2 Parser-focused tests

- `tests/parser_blocks.rs`
- `tests/parser_inlines.rs`
- `tests/parser_regressions.rs`

Coverage map (minimum):

- Headings: ATX + setext
- Lists: ordered/unordered, nested, tight/loose
- Blockquote nesting and lazy continuation
- Fenced and indented code interactions
- Tables and interruption rules
- Emphasis/strong/strike combinations
- Links/images/reference links
- Escapes/entities/backticks edge cases
