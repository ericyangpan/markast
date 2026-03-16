# P0 Parser Rebuild Plan

Last updated: 2026-03-16

## Goal

Keep hardening the in-house Rust Markdown parser until runtime and snapshot compatibility gaps are small, explicit, and stable.

Detailed design:

- `docs/p0-parser-design.md`

## Scope

- In scope:
  - Keep improving the parser used by `render_markdown_to_html`.
  - Keep current HTML output contract and `RenderOptions` behavior (`gfm`, `breaks`, `pedantic`).
  - Keep passing own tests and marked compatibility tests.
- Out of scope for P0:
  - Sanitization pipeline (P1).
  - WASM packaging/runtime fallback (P2).

## Compatibility Contract

- Must preserve:
  - `render_markdown_to_html(input, options) -> String`
  - `render_markdown_to_html_buf(input, options, buf)`
  - GFM toggle semantics (`options.gfm`)
  - line-break toggle semantics (`options.breaks`)
  - pedantic toggle semantics (`options.pedantic`)
- Must not regress:
  - `npm run test:own`
  - `npm run test:compat`

## Autonomous Execution Loop

This is the standing work agreement for parser/runtime compatibility work.

Current checked-in baseline (2026-03-16):

- current-runtime compatibility: `1479` passed, `6` gaps
- vendored snapshot compatibility: `1327` passed, `158` gaps
- comparable corpus size: `1485` cases
- benchmark guardrails:
  - `Comparable Corpus >= 1.25x` vs `marked`
  - `Marked Fixtures >= 1.00x` vs `marked`

Default aggressive target:

- reduce `tests/compat/runtime_xfail.yaml` from `6` to `0`
- keep `tests/compat/xfail.yaml` intentionally classified while runtime-first work continues
- preserve the benchmark guardrails above

Current work ordering:

1. Runtime-first mismatch reduction from `tests/compat/runtime_xfail.yaml`.
2. Shared recovery of snapshot gaps in `tests/compat/xfail.yaml`.
3. Snapshot-only cleanup after runtime-first batches stop paying off.

Current runtime fixture tail to inspect first:

- `new/em_and_reflinks.md`
- `new/html_comments.md`
- `original/markdown_documentation_syntax.md`
- `original/ordered_and_unordered_lists.md`
- `commonmark` / `gfm-commonmark` example `93`

Default batch loop:

1. Pick the highest-yield cluster from `tests/compat/runtime_xfail.yaml`.
2. Make the narrowest parser or renderer change that removes a real mismatch category.
3. Add or promote focused regression coverage near the affected layer.
4. Run focused tests, then `npm run check:strict`, then `npm run test:compat`.
5. If hot paths changed, run `npm run bench` in isolation.
6. Update `tests/compat/runtime_xfail.yaml` and `tests/compat/xfail.yaml` only when the behavior change is intentional and verified.
7. Continue to the next cluster without waiting for a new direction unless an escalation condition fires.

Escalate only when:

- a fix requires changing the public API, CLI behavior, package contract, or documented default semantics
- a fix requires editing `third_party/marked/*` or changing the vendored/current `marked` version target
- runtime and snapshot targets require conflicting behavior that the current harness cannot represent cleanly
- three consecutive well-scoped batches produce no net reduction in runtime gaps
- a correctness fix would push isolated benchmark results below the guardrail floor

## Architecture Split

### 1) Syntax Model

- `src/markdown/ast.rs`
- Define block/inline nodes with source spans.
- Keep spans for better diff/debug output in compat failures.

### 2) Lexer / Scanner

- `src/markdown/lexer.rs`
- Low-level scanning helpers for blocks and inlines.
- Deterministic behavior; no regex-heavy backtracking in the hot path.

### 3) Block Parser

- `src/markdown/block.rs`
- Parse:
  - paragraph
  - ATX/setext headings
  - blockquote
  - ordered/unordered list (+ nesting/tight-loose)
  - fenced/indented code
  - thematic break
  - table (gfm)
  - html block (pass-through rules)
  - reference definitions

### 4) Inline Parser

- `src/markdown/inline.rs`
- Parse:
  - emphasis/strong/strikethrough
  - codespan
  - links/images/reflinks/autolinks
  - escapes/entities
  - raw html inline
  - tasklist checkbox marker handling (gfm)

### 5) HTML Renderer

- `src/markdown/render.rs`
- Render AST to HTML.
- Keep current rendering contract stable while parser internals evolve.

### 6) Public API Bridge

- `src/markdown/mod.rs`
- `src/lib.rs`
- `render_markdown_to_html` and `render_markdown_to_html_buf` route to the parser module.

## Milestones

### M1: Skeleton + Minimal Pipeline

- New parser path is the runtime default rendering route.
- Own tests compile and run.

Status: completed.

### M2: Block Completeness

- Lists, blockquotes, indented code, references, and tables are implemented.
- Remaining work is edge-rule refinement instead of missing major constructs.

Status: functionally complete, still receiving compatibility cleanup.

### M3: Inline Completeness

- Delimiter, links/images, reflinks, autolinks, and entity handling exist.
- Remaining work is edge-behavior reduction against `marked`.

Status: functionally complete, still receiving compatibility cleanup.

### M4: Marked Edge Behavior

- Address high-value incompatibilities in list interruption, HTML handling, autolink boundaries, references, and fixture-specific edges.

Status: active.

### M5: Cutover

- Remove `pulldown-cmark` dependency.
- Make new parser default.

Status: completed.

Historical batch-by-batch execution notes were intentionally removed from this document.
Use Git history and focused tests as the archive for completed work.
