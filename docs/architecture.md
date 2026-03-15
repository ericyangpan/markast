# Architecture

This document explains where the main logic lives and how Markdown becomes HTML in `markast`.

## System Overview

The repository has three main surfaces:

- Rust library: parses Markdown and renders HTML
- Rust CLI: reads files or stdin and prints fragment or document HTML
- npm packaging: distributes the CLI as a root package plus platform-specific binary packages

## Main Entry Points

`src/main.rs`

- CLI argument parsing with `clap`
- Reads Markdown input from a file or stdin
- Calls `render_markdown_to_html(...)`
- Optionally wraps the fragment with `build_html_document(...)`

`src/lib.rs`

- Public library surface
- Defines `RenderOptions`
- Defines theme file format and HTML document builder
- Routes rendering requests into `src/markdown`

## Rendering Pipeline

Current high-level flow:

1. `src/main.rs` or library callers pass Markdown plus `RenderOptions`
2. `src/markdown/render_html.rs` takes a trivial-paragraph fast path when possible
3. otherwise `src/markdown/parser.rs` parses the document into the internal AST
4. `src/markdown/block.rs` builds block structure
5. `src/markdown/inline.rs` resolves inline syntax inside block content
6. `src/markdown/render.rs` converts the AST into HTML

The public API contract is preserved through:

- `render_markdown_to_html(input, options) -> String`
- `render_markdown_to_html_buf(input, options, buf)`

## Module Map

`src/markdown/mod.rs`

- Internal module boundary used by `src/lib.rs`
- Re-exports the current parser/render entrypoints inside the crate

`src/markdown/ast.rs`

- Internal document model used by the parser and renderer

`src/markdown/span.rs`

- Source span helpers used for parser bookkeeping and renderer decisions

`src/markdown/lexer.rs`

- Low-level scanning helpers used by parsing

`src/markdown/options.rs`

- Internal parser option mapping from public `RenderOptions`

`src/markdown/parser.rs`

- Parser entrypoint that coordinates block parsing and inline resolution

`src/markdown/block.rs`

- Block parsing logic
- Lists, blockquotes, code blocks, headings, tables, HTML blocks, references

`src/markdown/inline.rs`

- Inline parsing logic
- Links, emphasis, images, code spans, raw HTML, task markers

`src/markdown/render.rs`

- HTML renderer for the internal AST

`src/markdown/render_html.rs`

- Top-level render coordinator
- Contains the trivial single-paragraph fast path

## Tests by Responsibility

`tests/own_rendering.rs`

- Product-level assertions for `markast` behavior

`tests/parser_blocks.rs`

- Focused block parser regressions

`tests/parser_inlines.rs`

- Focused inline parser regressions

`tests/parser_regressions.rs`

- Fixture-backed targeted regressions

`tests/compat_snapshot.rs`

- Compares `markast` output to vendored `marked` fixture snapshots

`tests/compat_runtime.rs`

- Compares `markast` output to the current vendored `marked` npm runtime

## npm and Release Layout

Root `package.json`:

- exposes the `markast` binary through `bin/markast.js`
- defines developer commands
- depends on platform packages through `optionalDependencies`

`npm/*` packages:

- each package contains one platform-specific binary
- versions must stay aligned with the root package

`.github/workflows/ci.yml`:

- checks npm version sync
- runs the strict Rust gate

`.github/workflows/release.yml`:

- rebuilds platform binaries
- stages platform packages
- publishes platform packages and then the root npm package

## Where To Change Things

If the task is about CLI flags or input handling, start in `src/main.rs`.

If the task is about public rendering options or document themes, start in `src/lib.rs`.

If the task changes Markdown semantics, start in `src/markdown/*` and the focused parser tests.

If the task changes compatibility expectations, inspect `tests/compat_*`, `tests/compat/*.yaml`, and `scripts/render-marked-runtime.mjs`.
