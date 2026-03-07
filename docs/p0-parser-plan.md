# P0 Parser Rebuild Plan

Last updated: 2026-03-06

## Goal

Build a Rust Markdown parser from scratch and remove `pulldown-cmark` from `markec`.

Detailed design:

- `docs/p0-parser-design.md`

## Scope

- In scope:
  - Replace current parse engine used by `render_markdown_to_html`.
  - Keep current HTML output contract and `RenderOptions` behavior (`gfm`, `breaks`).
  - Keep passing own tests and marked compatibility tests.
- Out of scope for P0:
  - Sanitization pipeline (P1).
  - WASM packaging/runtime fallback (P2).

## Compatibility Contract

- Must preserve:
  - `render_markdown_to_html(input, options) -> String`
  - GFM toggle semantics (`options.gfm`)
  - line-break toggle semantics (`options.breaks`)
- Must not regress:
  - `npm run test:own`
  - `npm run test:compat`
- `tests/compat/xfail.yaml` is allowed during migration, but trend must be downward.

## Architecture Split

### 1) Syntax Model

- `src/markdown/ast.rs`
- Define block/inline nodes with source spans.
- Keep spans for better diff/debug output in compat failures.

### 2) Lexer / Scanner

- `src/markdown/lexer.rs`
- Line scanner + inline token scanner utilities.
- Deterministic behavior; no regex-heavy backtracking in hot path.

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

- `src/markdown/html.rs`
- Render AST to HTML.
- Keep current post-process hooks:
  - softbreak-to-`<br>` by option
  - plain URL/email autolink policy compatible with marked tests

### 6) Public API Bridge

- `src/markdown/mod.rs`
- `src/lib.rs`
- `render_markdown_to_html` routes to the new parser module.

## Milestones

### M0: Freeze Baseline

- Lock current compat baseline and snapshot stats.
- Add script output: total cases / xfail / recovered / new failures.

Exit criteria:

- Baseline reproducible locally and in CI.

### M1: Skeleton + Minimal Pipeline

- Add parser module layout and compile-only scaffolding.
- Implement `paragraph`, `heading`, `code fence`, `hr`.

Exit criteria:

- New parser path is the runtime default rendering route.
- Own tests compile and run.

Status: completed (default render route switched to `render_markdown_to_html` in `src/markdown/mod.rs`; old `markec` branch removed).

### M2: Block Completeness

- Implement list/blockquote nesting, indented code, references, table (gfm).
- Add focused unit tests per construct.

Exit criteria:

- No catastrophic parse failures on marked fixtures.

Status: in progress (new block AST + block table/list/blockquote/code paths are mounted behind line-driven block parse; nested edge-cases and interruption rules still pending).

### M3: Inline Completeness

- Implement delimiter stack for emphasis/strong/strike.
- Implement links/images/reference links and edge cases.

Exit criteria:

- Majority of remaining xfails are known HTML formatting diffs or rare edge rules.

Status: in progress (inliner now route is objectized and reference-aware; full delimiter-stack rewrite is pending).

### M4: Marked Edge Behavior

- Address high-value incompatibilities:
  - list tight/loose and interruption rules
  - table vs setext ambiguity
  - autolink quotes and punctuation boundaries
  - pedantic/front-matter-controlled paths used by marked fixtures

Exit criteria:

- `xfail` count significantly reduced from current baseline.

Status: in progress (compat baseline still needs staged reduction by category).

### M5: Cutover

- Remove `pulldown-cmark` dependency.
- Make new parser default.

Exit criteria:

- `Cargo.toml` has no `pulldown-cmark`.
- `npm run test:own` and `npm run test:compat` pass.

Status: parser dependency cutover is structurally complete; next step is evidence pass for `test:own`/`test:compat` after staged fixes.

## Execution Update (2026-03-06)

Current baseline snapshot:

- marked compat suite: `1485` total cases, `614` baseline `xfail`
- current top CommonMark failure sections:
  - `Emphasis and strong emphasis`: `130`
  - `Links`: `68`
  - `List items`: `50`
  - `Lists`: `38`
  - `Images`: `32`
  - `Link reference definitions`: `30`
- current gate gaps before parser rewrites:
  - `cargo test --all-targets` fails in inline unit assertions
  - `npm run test:own` has assertion noise mixed with parser behavior checks
  - planned focused parser test files do not exist yet

Next execution order:

### E0: Gate Repair

- Fix inline unit test compile failures.
- Remove or correct own-test assertions that are checking unstable formatting instead of parser behavior.
- Exit criteria:
  - `cargo test --all-targets` runs
  - `npm run test:own` is green

### E1: Focused Regression Net

- Add `tests/parser_blocks.rs`, `tests/parser_inlines.rs`, `tests/parser_regressions.rs`.
- Seed them with:
  - passing focused coverage for parser contracts already expected to work
  - ignored compat-backed regression cases for known gaps by category
- Exit criteria:
  - new parser changes can be validated without relying only on the full compat harness

### E2: Inline Delimiter / Link-Ref Rewrite

- Highest-yield batch; do this before chasing more fixture-specific xfails.
- Replace greedy delimiter matching with delimiter-stack behavior for `*`, `_`, `~`.
- Unify reference-link normalization with block definition parsing and continuation behavior.
- Target compat categories:
  - emphasis / strong emphasis
  - links / images
  - link reference definitions

### E3: List Model and Paragraph Interruption

- Add explicit list semantics needed by render:
  - `tight` / `loose`
  - marker-aware behavior where necessary
- Remove renderer-side guessing for list paragraph flattening.
- Tighten paragraph interruption rules around list markers.

### E4: Table / Setext / HTML / Blockquote Cleanup

- Fix table row continuation and column normalization.
- Fix table vs setext precedence.
- Narrow HTML block detection so inline HTML is not promoted too early.
- Preserve blockquote indentation needed for nested code/list structure.

Execution notes:

- Do not use `npm run test:compat:update-xfail` as a routine workflow step.
- Every compat reduction batch must include:
  - one focused parser test
  - one compat delta check
  - explicit removal of recovered ids instead of blind baseline refresh

## Rebucket Update (2026-03-06, after E3/E4 pass)

Current baseline snapshot:

- marked compat suite: `1485` total cases, `420` baseline `xfail`
- current focused regression status:
  - `table_vs_setext`: promoted to enforced regression test and passing
  - `nested_blockquote_in_list`: promoted to enforced regression test and passing
  - `incorrectly_formatted_list_and_hr`: still ignored; remaining mismatch is now mostly pretty-HTML / normalization noise, not a high-value parser semantic gap

Remaining `xfail` concentration by bucket:

- CommonMark / GFM sections:
  - `Links`: `62`
  - `List items`: `36`
  - `Images`: `32`
  - `Link reference definitions`: `30`
  - `Autolinks`: `26`
  - `Entity and numeric character references`: `22`
  - `Raw HTML`: `22`
  - `Lists`: `20`
  - `HTML blocks`: `16`
  - `Setext headings`: `14`
  - `Block quotes`: `14`
- non-CommonMark buckets:
  - `new/*`: `41`
  - `original/*`: `10`
  - `gfm.0.29`: `15`

Implication:

- The next highest-yield parser batch is no longer table/list cleanup.
- The next batch should target the link family as one cluster:
  - links
  - images
  - link reference definitions
  - autolinks
- This cluster is large enough that it should be treated as a dedicated execution stage, not mixed with unrelated block cleanup.

Execution change:

### E5: Link / Image / RefDef / Autolink Cluster

- Scope:
  - remaining inline link destination edge cases
  - reference definition lookup / normalization mismatches
  - image title / label edge cases
  - autolink boundaries and entity interaction
- Guardrails:
  - do not refresh `xfail` until the whole link-family batch is complete
  - add 3-5 focused parser/regression tests first, then change parser behavior
  - keep `incorrectly_formatted_list_and_hr` parked unless a real semantic gap is identified

Status update (2026-03-06, E5 batch 1 complete):

- compat baseline moved from `420` to `357`
- completed in this batch:
  - multiline reference-definition titles
  - Unicode case-folded reference labels
  - reference-image resolution with flattened `alt` text
  - generic scheme autolinks
  - angle-autolink boundary fix for spaced `< ... >` inputs
- promoted regressions now enforced and passing:
  - CommonMark `example-196`
  - CommonMark `example-206`
  - CommonMark `example-573`
  - CommonMark `example-598`
- next E5 focus:
  - remaining link destination / entity interaction edge cases
  - remaining image/title variants not covered by recovered CommonMark cases

Status update (2026-03-06, E5 batch 2 complete):

- compat baseline moved from `357` to `306`
- completed in this batch:
  - inline link destination parser rewrite for angle / bare destinations
  - destination normalization for escapes, entity decoding, and percent-encoding
  - title parsing for quoted and parenthesized variants
  - block/container-aware reference-definition prescan rules
  - `pedantic` inline fallback for unclosed angle destinations used by marked fixtures
- promoted regressions now enforced and passing:
  - CommonMark `example-202`
  - CommonMark `example-213`
  - CommonMark `example-218`
  - CommonMark `example-503`
  - CommonMark `example-505`
  - CommonMark `example-609`
  - `new/link_lt`
  - `new/def_blocks`
- remaining deferred gap is still:
  - `incorrectly_formatted_list_and_hr`

Status update (2026-03-07, E5 batch 3 complete):

- compat baseline moved from `306` to `296`
- completed in this batch:
  - reference-label case folding for `ẞ/ß -> ss`
  - first-definition-wins semantics for duplicate reference definitions
  - non-pedantic restriction that full/collapsed reference links do not cross line breaks between labels
  - pedantic-only allowance for reference-style links that span a line break between labels
- promoted regressions now enforced and passing:
  - CommonMark `example-540`
  - CommonMark `example-543`
  - CommonMark `example-544`
  - CommonMark `example-556`
- remaining high-yield E5 work is now concentrated in:
  - nested/illegal link fallback behavior
  - remaining image/link label edge cases

Status update (2026-03-07, E5 batch 4 complete):

- compat baseline moved from `296` to `268`
- completed in this batch:
  - outer link/reference-link fallback when the parsed label contains an inner link
  - raw HTML and autolink skipping during bracket matching, so `]` inside those spans no longer closes a link label
  - non-pedantic restriction that full reference links do not bridge a space between the two bracketed labels
  - percent-encoding of `[` and `]` in normalized destinations for recovered autolink-in-label cases
- promoted regressions now enforced and passing:
  - CommonMark `example-518`
  - CommonMark `example-520`
  - CommonMark `example-524`
  - CommonMark `example-526`
  - CommonMark `example-532`
  - CommonMark `example-542`
- remaining high-yield E5 work is now concentrated in:
  - the rest of the illegal/nested link fallback family
  - residual autolink/entity/image tail cases outside the recovered cluster

Deferred cleanup:

- `incorrectly_formatted_list_and_hr` should only be revisited if:
  - compat normalization is intentionally expanded, or
  - a real block semantic mismatch reappears after future parser changes

Status update (2026-03-07, E6 list/blockquote indentation batch complete):

- compat baseline moved from `268` to `201`
- completed in this batch:
  - list-item `content_indent` now tracks marker padding width instead of using a fixed continuation guess
  - list item collection now distinguishes sibling-vs-nested markers by indentation level instead of exact raw indent equality
  - partial-tab continuation stripping is preserved for nested list structure, so tab-indented continuations no longer collapse incorrectly
  - blockquote marker stripping now removes exactly one marker padding character (`space` or `tab`) instead of trimming all leading whitespace
  - fenced code blocks now remove opening-fence indent from content lines, fixing list-contained tab-indented fence payloads
- promoted regressions now enforced and passing:
  - CommonMark `example-259`
  - `new/tab_after_blockquote`
  - focused parser coverage for quoted list continuation and list-contained fenced-code indent normalization
- notable recovered clusters:
  - multiple CommonMark/GFM list-item and blockquote examples in the `250s`, `270s`, `280s`, and `310s`
  - `original/blockquotes_with_code_blocks`
  - `new/list_wrong_indent`
  - `new/tricky_list`
- next high-yield cluster after this batch:
  - remaining `Links / Images / Entity` tail cases, or
  - a deliberate return to the parked `incorrectly_formatted_list_and_hr` gap if block semantics become the priority again

Status update (2026-03-07, E7 HTML/raw-HTML batch complete):

- compat baseline moved from `201` to `159`
- completed in this batch:
  - HTML block start detection now distinguishes paragraph-interrupting block forms from generic inline-tag forms
  - HTML block parsing now keeps block tags open until the terminating blank line and supports comment / processing-instruction / declaration / CDATA forms
  - raw HTML inline parsing now accepts processing instructions, declarations, and CDATA in addition to tag-like spans
  - HTML tag parsing now handles boolean attributes followed by additional attributes without prematurely rejecting the span
- promoted regressions now enforced and passing:
  - CommonMark `example-151`
  - CommonMark `example-180`
  - CommonMark `example-185`
  - CommonMark `example-627`
  - CommonMark `example-628`
  - CommonMark `example-629`
- notable recovered clusters:
  - multiple CommonMark/GFM HTML block examples in the `140s`, `160s`, and `180s`
  - multiple CommonMark/GFM raw HTML examples in the `620s`
  - hard-line-break tail examples `642` and `643` recovered as part of the same parsing cleanup
- remaining high-yield work after this batch:
  - residual `Links / Images / Entity` tail cases
  - numeric/entity reference normalization edges
  - the parked `incorrectly_formatted_list_and_hr` regression if a real block semantic gap remains after later cleanup

Status update (2026-03-07, E8 entity/entity-reference batch complete):

- compat baseline moved from `159` to `142`
- completed in this batch:
  - added a shared HTML entity parser so inline text, reference metadata, and fenced-code info strings all use the same decode path
  - expanded named entity coverage for the remaining compat corpus, including multi-code-point entities such as `&ngE;`
  - numeric character references now replace invalid scalar values such as `&#0;` with `U+FFFD` instead of leaving the source literal behind
  - inline entity decoding now emits literal text nodes, so decoded `*`, tabs, and newlines no longer accidentally trigger emphasis, list, or block parsing
- promoted regressions now enforced and passing:
  - CommonMark `example-25`
  - CommonMark `example-26`
  - CommonMark `example-34`
  - CommonMark `example-37`
  - CommonMark `example-38`
  - CommonMark `example-39`
- notable recovered clusters:
  - CommonMark/GFM `Entity and numeric character references` examples `25`, `26`, `27`, `34`, `37`, `38`, `39`, `40`, and `41`
  - `original/amps_and_angles_encoding`
- remaining high-yield work after this batch:
  - `Setext headings` and related block-interruption fixtures
  - residual `Lists / Tabs / Hard line breaks` tail cases
  - the parked `incorrectly_formatted_list_and_hr` regression if a real block semantic gap remains after later cleanup

Status update (2026-03-07, E9 setext/blockquote boundary batch complete):

- compat baseline moved from `142` to `122`
- completed in this batch:
  - setext heading content now strips paragraph-continuation indent and trailing horizontal whitespace before inline parsing
  - single-character setext underlines are accepted, matching marked for lone `=` / `-` forms
  - unquoted thematic-break lines no longer remain inside lazy blockquotes, while short `==` / `--` lazy continuations are preserved
  - paragraph continuation lines are normalized only when appended, so block-interruption checks still see the original raw line shape
- promoted regressions now enforced and passing:
  - CommonMark `example-82`
  - CommonMark `example-83`
  - CommonMark `example-89`
  - CommonMark `example-92`
  - CommonMark `example-101`
- notable recovered clusters:
  - CommonMark/GFM `Setext headings` examples `82`, `83`, `84`, `87`, `89`, `92`, and `101`
  - CommonMark/GFM paragraph/blockquote side cases `49`, `70`, `113`, and `234`
  - `new/list_item_empty`
- remaining high-yield work after this batch:
  - `new/blockquote_setext` paragraph softbreak-vs-space normalization
  - `new/pedantic_heading` and `new/pedantic_heading_interrupts_paragraph`
  - table-tail rows that currently escape into setext (`lheading_following_*`, `inlinecode_following_*`, `strong_following_*`, `text_following_*`)

Status update (2026-03-07, E10 table-tail continuation batch complete):

- compat baseline moved from `122` to `112`
- completed in this batch:
  - table parsing now supports marked-style implicit tail rows after a valid table body, so plain text and inline-only lines can continue the table as a first-column cell with remaining cells padded empty
  - implicit tail rows stop cleanly before block-start lines such as ATX headings, blockquotes, fences, lists, and paragraph-interrupting HTML blocks
  - table blank-line detection now uses Markdown block whitespace rules (`space`/`tab` only), so `NBSP` tail rows are preserved and collapse to empty cells instead of ending the table
- promoted regressions now enforced and passing:
  - `new/lheading_following_table`
  - `new/inlinecode_following_tables`
  - `new/text_following_tables`
  - `new/nbsp_following_tables`
- notable recovered clusters:
  - `new/lheading_following_table`
  - `new/lheading_following_nptable`
  - `new/inlinecode_following_tables`
  - `new/inlinecode_following_nptables`
  - `new/strong_following_tables`
  - `new/strong_following_nptables`
  - `new/text_following_tables`
  - `new/text_following_nptables`
  - `new/nbsp_following_tables`
  - GFM spec `example-5`
- remaining high-yield work after this batch:
  - `new/blockquote_setext`
  - `new/pedantic_heading` and `new/pedantic_heading_interrupts_paragraph`
  - `Lists / Tabs / fenced-code tail` cases such as `list_item_tabs`, `list_item_text`, `list_loose_tasks`, and `fences_breaking_paragraphs`

Status update (2026-03-07, E11 list-tabs batch complete):

- compat baseline moved from `112` to `107`
- completed in this batch:
  - list marker padding now measures tabs from the real marker column instead of column zero, so ordered and unordered items using `\t` after the marker no longer over-indent their continuation threshold
  - tab-indented continuation paragraphs stay inside the owning list item instead of being misparsed as indented code blocks
  - tab-indented nested list items stay nested across multiple levels
  - loose task items now render checkboxes inside the first paragraph, and empty task markers such as `[ ]` fall back to literal text
  - indented code detection and stripping now honor tab-expanded leading columns, recovering the CommonMark tab semantics that previously stayed xfailed
- promoted regressions now enforced and passing:
  - `new/list_item_tabs`
  - `new/list_loose_tasks`
- notable recovered cases:
  - `new/list_item_tabs`
  - `new/list_loose_tasks`
  - `original/inline_html_simple`
  - CommonMark/GFM `example-2`
- remaining high-yield work after this batch:
  - `new/list_item_text`
  - `new/list_align_pedantic`
  - `new/paragraph-after-list-item`
  - remaining block/list interaction tails such as `fences_breaking_paragraphs` and `tasklist_blocks`

Status update (2026-03-07, E12 pedantic-list and tight-list-heading batch complete):

- compat baseline moved from `107` to `106`
- completed in this batch:
  - pedantic list-item collection now keeps outdented continuation text inside a non-zero-indented parent item after nested sublists instead of ejecting it to top level
  - tight list items now render an inline-text paragraph followed by a heading with a single separating space, matching marked's `list_code_header` HTML shape
- promoted regressions now enforced and passing:
  - `new/list_code_header`
- notable observations from this batch:
  - `new/list_item_text` is no longer a structural parser mismatch; the remaining diff is a marked pretty-HTML trailing space before `</li>`, so it stays in baseline for now
- remaining high-yield work after this batch:
  - code-block trailing-newline mismatches such as `code_block_no_ending_newline`, `paragraph-after-list-item`, `indented_details`, and `fences_breaking_paragraphs`
  - pedantic list alignment cases such as `list_align_pedantic`

Status update (2026-03-07, E13 pedantic-heading batch complete):

- compat baseline moved from `106` to `103`
- completed in this batch:
  - ATX heading parsing is now pedantic-aware: `#h1` style headings are accepted without a space, leading indentation is rejected in pedantic mode, and trailing closing `#` markers are stripped using marked-compatible pedantic rules
  - pedantic paragraph parsing now yields before a following setext underline candidate, so `pedantic_heading_interrupts_paragraph` no longer absorbs the previous paragraph line
  - the same pedantic heading rules now apply in non-GFM mode, recovering `nogfm_hashtag`
- promoted regressions now enforced and passing:
  - `new/pedantic_heading`
  - `new/pedantic_heading_interrupts_paragraph`
  - `new/nogfm_hashtag`
- remaining high-yield work after this batch:
  - code-block trailing-newline mismatches such as `code_block_no_ending_newline`, `paragraph-after-list-item`, `indented_details`, and `fences_breaking_paragraphs`
  - pedantic list alignment in `list_align_pedantic`

## File Plan

- New:
  - `src/markdown/mod.rs`
  - `src/markdown/ast.rs`
  - `src/markdown/lexer.rs`
  - `src/markdown/block.rs`
  - `src/markdown/inline.rs`
  - `src/markdown/html.rs`
  - `tests/parser_blocks.rs`
  - `tests/parser_inlines.rs`
  - `tests/parser_regressions.rs`
- Update:
  - `src/lib.rs`
  - `Cargo.toml`
  - `README.md`

## Risk Register

- Risk: list and table interruption rules are the largest compat gap.
  - Mitigation: implement explicit precedence matrix + fixture-driven tests.
- Risk: emphasis/link delimiter interactions can explode in complexity.
  - Mitigation: single-pass delimiter stack design, no ad-hoc regex fallback.
- Risk: performance regression.
  - Mitigation: add micro-bench fixture set before cutover.

## Execution Rules

- Every parser behavior change must include:
  - one focused unit test
  - one compat impact check
- No blind baseline refresh:
  - if `xfail` changes, include reason category in commit message.
