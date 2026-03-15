# markast Requirements

Last updated: 2026-03-15

## Product Direction

`markast` is an HTML-output Markdown renderer that targets compatibility with `marked` while adding project-specific styling features.

## Current Status

The parser cutover portion of P0 is complete:

- `render_markdown_to_html` uses the in-house Rust parser
- `Cargo.toml` no longer depends on `pulldown-cmark`
- the remaining P0 work is compatibility reduction, benchmark discipline, and release hardening

Current checked-in baselines:

- snapshot compatibility: `1404 / 1485` passed, `81` tracked gaps
- runtime compatibility: `1402 / 1485` passed, `83` tracked gaps

## Priority Roadmap

### P0 (Primary Mission)

- Keep hardening the in-house Rust parser until compatibility gaps are small and well-understood.
- Prioritize current-runtime parity first, while keeping vendored snapshot gaps on a downward trend.
- Preserve benchmark guardrails while reducing compatibility gaps.

Acceptance criteria:

- `npm run check:strict`, `npm run test:own`, and `npm run test:compat` pass in CI.
- `tests/compat/runtime_xfail.yaml` and `tests/compat/xfail.yaml` continue to shrink or are intentionally justified.
- Isolated benchmark runs keep `Comparable Corpus` at `1.25x` or better vs `marked`, with `Marked Fixtures` at or above parity.
- Remaining gaps, if any, are explicitly categorized rather than left as unexplained baseline noise.

Near-term execution order:

1. Reduce runtime mismatches in `tests/compat/runtime_xfail.yaml`.
2. Fold in the same fixes to snapshot coverage where they also recover vendored fixtures.
3. Revisit snapshot-only deltas once runtime parity work stops producing efficient wins.
4. Resolve the remaining Windows npm package publish block when release work resumes.

Primary working docs:

- `docs/p0-parser-plan.md`
- `docs/p0-parser-design.md`
- `docs/testing-and-compat.md`
- `docs/performance.md`

Packaging follow-up:

- `docs/rename-to-markast.md`

What counts as "next work" right now:

- CommonMark / GFM runtime mismatches
- Remaining fixture tails around HTML, autolinks, entity handling, and list structure
- The npm registry block on `markast-win32-x64-msvc`

Default aggressive execution target:

- drive `tests/compat/runtime_xfail.yaml` from `83` toward `0`
- keep `tests/compat/xfail.yaml` trending downward from `81` without regressing runtime behavior
- preserve benchmark guardrails:
  - `Comparable Corpus >= 1.25x` vs `marked`
  - `Marked Fixtures >= 1.00x` vs `marked`

### P1 (Security Mode)

- Add optional HTML sanitization support for output.
- Preserve marked-compat mode by keeping sanitize disabled by default.

Acceptance criteria:

- Provide explicit sanitize toggle in API/CLI.
- Add dedicated sanitize tests separate from compat tests.

### P2 (WASM Runtime Support)

- Add WASM build output for browser/edge/runtime fallback.
- Keep native prebuilt binaries as the primary Node path.

Acceptance criteria:

- Shared Rust core for native and WASM outputs.
- NPM package supports native-first loading with WASM fallback.
