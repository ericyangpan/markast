# markast Development Docs

This directory is the main entrypoint for people and agents working on `markast`.

## Start Here

Read in this order when you are new to the repo or returning after a while:

1. `docs/README.md`
2. `docs/architecture.md`
3. `docs/testing-and-compat.md`
4. `docs/performance.md`

Then read these depending on the workstream:

- `docs/requirements.md`
- `docs/p0-parser-plan.md`
- `docs/p0-parser-design.md`
- `docs/rename-to-markast.md`

## Current Status

As of 2026-03-18, the repository state is:

- the in-house Rust parser is already the default and only production parser
- snapshot compatibility baseline: `1327 / 1485` passing, `158` tracked gaps
- current-runtime compatibility baseline: `1479 / 1485` passing, `6` tracked gaps
- release/publish follow-up: npm platform packages (including Windows) are published; `markast@0.1.1` is deprecated and the current release line is `0.1.2`

These numbers come from the checked-in baselines:

- `tests/compat/xfail.yaml`
- `tests/compat/runtime_xfail.yaml`

## Current Priorities

If you are picking up the next meaningful work, use this order:

1. Reduce `tests/compat/runtime_xfail.yaml`.
2. Keep `tests/compat/xfail.yaml` moving down when the same fix also helps vendored snapshots.
3. Keep benchmark guardrails intact while fixing compatibility.
4. Keep npm release automation healthy (trusted publishing / token rotation).

The current runtime gap is now a small fixture tail:

- `new/em_and_reflinks.md`
- `new/html_comments.md`
- `original/markdown_documentation_syntax.md`
- `original/ordered_and_unordered_lists.md`
- `commonmark` / `gfm-commonmark` example `93`

Use `docs/p0-parser-plan.md` for the execution loop around those clusters.

## Project Snapshot

`markast` is a Rust Markdown renderer shipped through npm.

Core characteristics:

- CLI entrypoint in `src/main.rs`
- public library API in `src/lib.rs`
- in-house Markdown parser under `src/markdown/*`
- npm wrapper package in the repo root and platform packages under `npm/*`
- compatibility fixtures vendored from `marked` under `third_party/marked`

## Fast Path

For humans:

- Install Rust stable and Node.js 18+.
- Run `npm install`.
- Run `npm run check:strict`.
- Run `npm run test:compat` before merging parser-affecting work.

For agents:

- Check `git status --short` before editing.
- Assume the source of truth is code plus tests, not older roadmap notes.
- Prefer the narrowest validation that proves the change, then run broader gates if parser behavior changed.
- For parser/runtime compatibility work, follow the autonomous execution loop in `docs/p0-parser-plan.md`.
- Do not update `tests/compat/xfail.yaml` or `tests/compat/runtime_xfail.yaml` unless the behavior change is intentional and verified.
- Do not edit `third_party/marked/*` unless the task is explicitly about fixture sync.

## Doc Map

Stable reference docs:

- `docs/architecture.md`
- `docs/releasing.md`
- `docs/testing-and-compat.md`
- `docs/performance.md`

Working docs:

- `docs/requirements.md`
- `docs/p0-parser-plan.md`
- `docs/p0-parser-design.md`

Release-status doc:

- `docs/rename-to-markast.md`

## Documentation Scope

These docs should prefer durable working agreements over execution logs.

Good doc topics:

- code layout
- API and behavior contracts
- test and release gates
- roadmap and design intent

Usually not worth documenting here:

- temporary implementation steps
- one-off execution notes
- change logs that already exist in Git history
- detailed per-batch status logs once the batch is complete

## Operating Principles

These rules make the repo easier for both humans and agents to change safely:

- Keep behavior changes covered by focused tests near the affected layer.
- Treat compatibility failures as product signals, not just test noise.
- Prefer small, source-located fixes over large speculative rewrites.
- Keep docs updated when the working agreement changes.
