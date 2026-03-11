# markrs

`markrs` is a Rust Markdown renderer distributed through npm.

By default it outputs HTML fragments like `marked`.
It can also output a full HTML document with built-in or custom styles.

## Install

```bash
npm i -g markrs
```

## Usage

Render Markdown to HTML fragment (default):

```bash
markrs README.md > out.html
cat README.md | markrs
```

Render full HTML document with built-in theme:

```bash
markrs --document --theme github README.md > page.html
markrs --document --theme dracula README.md > page.html
markrs --document --theme paper README.md > page.html
```

Apply custom style definition (JSON):

```bash
markrs --document --theme-file theme.json README.md > page.html
```

`theme.json` format:

```json
{
  "variables": {
    "--markrs-bg": "#0f1115",
    "--markrs-fg": "#f2f5f9",
    "--markrs-link": "#65c1ff"
  },
  "css": ".markrs h1 { letter-spacing: 0.02em; }"
}
```

Append extra CSS file:

```bash
markrs --document --css ./extra.css README.md > page.html
```

## Development

```bash
npm run check
npm run check:strict
npm run test:own
npm run test:compat:snapshot
npm run test:compat:runtime
npm run test:compat
npm run test:compat:report
npm run build
```

Parser engine:
Current default and only parser is the in-house `markdown` module (new parser pipeline), with no external markdown engine dependency.

Requirements and roadmap: `docs/requirements.md`

Compatibility fixtures are synced under `third_party/marked/test/specs`.

Compat now has two layers:

- `npm run check:strict`: runs Rust compile/test gates with warnings denied.
- `npm run test:compat:snapshot`: gated comparison against vendored marked fixture/spec snapshots.
- `npm run test:compat:runtime`: gated comparison against the current vendored `marked` npm runtime.
- `npm run test:compat:runtime-drift`: auxiliary audit that checks whether snapshot-xfailed vendored fixtures still match the current runtime.
- `npm run test:compat`: runs both in sequence.

Known snapshot gaps are tracked in `tests/compat/xfail.yaml`.
Known runtime gaps are tracked in `tests/compat/runtime_xfail.yaml`.

Refresh the snapshot xfail baseline after intentional parser behavior changes:

```bash
npm run test:compat:snapshot:update-xfail
```

Refresh the runtime xfail baseline after intentional parser behavior changes:

```bash
npm run test:compat:runtime:update-xfail
```

## Compatibility Report

Current report date: 2026-03-08

This table compares the same parser-output cases from the official marked corpus under `third_party/marked/test/specs`.

Included in the same-case comparison:
- `new` + `original` fixture pairs: 153
- CommonMark JSON examples: 652
- GFM CommonMark mirror examples: 652
- GFM spec examples: 28
- Total comparable cases: 1485

Excluded from this table:
- `third_party/marked/test/unit/*.test.js`: 158 JS unit cases. These exercise Marked's JS API surface such as hooks, lexer/parser classes, CLI integration, and instance behavior, so there is no 1:1 Rust-side case mapping in `markrs` yet.
- `third_party/marked/test/specs/redos`: 7 ReDoS fixtures. These are security/performance-oriented fixtures and are not currently part of the `markrs` compat gates.

| Target | Case source | Passed | Gaps | Pass rate |
| --- | --- | ---: | ---: | ---: |
| `marked` self-spec result | vendored `marked` fixture/spec corpus | 1485 | 0 | 100.0% |
| `markrs` snapshot compat | vendored fixture/spec snapshots | 1449 | 36 | 97.6% |
| `markrs` runtime compat | current `marked@17.0.4` runtime | 1353 | 132 | 91.1% |

How to refresh:
- `npm run test:compat`
- `npm run test:compat:report`

## Benchmark

Reproduce locally:

```bash
npm install
npm run bench
```

The harness benchmarks shared Markdown corpora against five engines:
- `markrs` through an in-process Rust benchmark binary
- `pulldown-cmark` through an in-process Rust benchmark binary
- `marked` through `marked.parse(...)`
- `markdown-it` through `markdown-it.render(...)`
- `remark` through `remark + remark-gfm + remark-html`

`CommonMark Core` is the fairest suite for `pulldown-cmark`, because it runs the official CommonMark examples with `gfm=false`.

Raw data is written to `bench/results/latest.json`.

Performance strategy and optimization batches live in `docs/performance.md`.

`pulldown-cmark` is included as a throughput ceiling reference. `markrs` is not expected to match its architecture or semantics in Phase 1.

<!-- benchmark-report:start -->
Benchmark date: 2026-03-11

Method: in-process render throughput on the same default-GFM corpus for all engines. Outputs are not normalized for semantic equality; this report only measures rendering speed on shared inputs. `Trimmed mean ms` drops one run from each side for 6-9 samples, or 10% from each side for 10+ samples.

Environment: Apple M4 | darwin 24.6.0 (arm64) | Node 22.17.0 | Rust rustc 1.93.0 (254b59607 2026-01-19)

| Suite | Docs | Input size | Warmup | Measured | Source |
| --- | ---: | ---: | ---: | ---: | --- |
| README.md | 1 | 7.2 KiB | 10 | 30 | Project README rendered as a single document |
| CommonMark Core | 652 | 14.6 KiB | 4 | 10 | Official CommonMark 0.31.2 JSON examples rendered in non-GFM mode |
| Marked Fixtures | 153 | 58.3 KiB | 4 | 12 | `new` + `original` fixture pairs from vendored marked specs |
| Comparable Corpus | 1485 | 88.9 KiB | 4 | 12 | All 1485 comparable parser-output cases from vendored marked specs |

| Suite | Engine | Trimmed mean ms | Median ms | Docs/s | MiB/s | vs marked |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| README.md | markrs (Rust) | 0.29 | 0.29 | 3403.9 | 24.00 | 1.39x |
| README.md | pulldown-cmark (Rust) | 0.05 | 0.05 | 20505.5 | 144.55 | 8.39x |
| README.md | marked (JS) | 0.41 | 0.40 | 2442.7 | 17.22 | 1.00x |
| README.md | markdown-it (JS) | 0.48 | 0.46 | 2101.9 | 14.82 | 0.86x |
| README.md | remark + gfm + html | 4.94 | 4.92 | 202.3 | 1.43 | 0.08x |
| CommonMark Core | markrs (Rust) | 1.27 | 1.22 | 511652.6 | 11.17 | 1.57x |
| CommonMark Core | pulldown-cmark (Rust) | 0.48 | 0.49 | 1372406.2 | 29.95 | 4.21x |
| CommonMark Core | marked (JS) | 2.00 | 1.98 | 326087.5 | 7.12 | 1.00x |
| CommonMark Core | markdown-it (JS) | 2.15 | 2.14 | 303678.8 | 6.63 | 0.93x |
| CommonMark Core | remark + gfm + html | 30.46 | 30.20 | 21404.6 | 0.47 | 0.07x |
| Marked Fixtures | markrs (Rust) | 3.27 | 3.22 | 46833.0 | 17.42 | 1.46x |
| Marked Fixtures | pulldown-cmark (Rust) | 0.49 | 0.48 | 313390.8 | 116.58 | 9.78x |
| Marked Fixtures | marked (JS) | 4.78 | 4.76 | 32032.0 | 11.92 | 1.00x |
| Marked Fixtures | markdown-it (JS) | 3.40 | 3.34 | 45061.6 | 16.76 | 1.41x |
| Marked Fixtures | remark + gfm + html | 47.21 | 47.06 | 3241.0 | 1.21 | 0.10x |
| Comparable Corpus | markrs (Rust) | 5.62 | 5.65 | 264303.7 | 15.45 | 1.57x |
| Comparable Corpus | pulldown-cmark (Rust) | 1.23 | 1.24 | 1207681.1 | 70.60 | 7.16x |
| Comparable Corpus | marked (JS) | 8.81 | 8.85 | 168596.8 | 9.86 | 1.00x |
| Comparable Corpus | markdown-it (JS) | 6.71 | 6.74 | 221405.5 | 12.94 | 1.31x |
| Comparable Corpus | remark + gfm + html | 163.97 | 167.53 | 9056.8 | 0.53 | 0.05x |

Raw benchmark data: `bench/results/latest.json`
<!-- benchmark-report:end -->

## Release

Push a semver tag like `v0.1.0`.

GitHub Actions workflow `.github/workflows/release.yml` will:

1. Build each platform binary.
2. Pack and publish platform npm packages.
3. Publish the main package `markrs`.
