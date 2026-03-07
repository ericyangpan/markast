# markec

`markec` is a Rust Markdown renderer distributed through npm.

By default it outputs HTML fragments like `marked`.
It can also output a full HTML document with built-in or custom styles.

## Install

```bash
npm i -g markec
```

## Usage

Render Markdown to HTML fragment (default):

```bash
markec README.md > out.html
cat README.md | markec
```

Render full HTML document with built-in theme:

```bash
markec --document --theme github README.md > page.html
markec --document --theme dracula README.md > page.html
markec --document --theme paper README.md > page.html
```

Apply custom style definition (JSON):

```bash
markec --document --theme-file theme.json README.md > page.html
```

`theme.json` format:

```json
{
  "variables": {
    "--markec-bg": "#0f1115",
    "--markec-fg": "#f2f5f9",
    "--markec-link": "#65c1ff"
  },
  "css": ".markec h1 { letter-spacing: 0.02em; }"
}
```

Append extra CSS file:

```bash
markec --document --css ./extra.css README.md > page.html
```

## Development

```bash
npm run check
npm run test:own
npm run test:compat
npm run build
```

Parser engine:
Current default and only parser is the in-house `markdown` module (new parser pipeline), with no external markdown engine dependency.

Requirements and roadmap: `docs/requirements.md`

Compatibility fixtures are synced under `third_party/marked/test/specs`.
Known compatibility gaps are tracked in `tests/compat/xfail.yaml`.

Refresh xfail baseline after intentional parser behavior changes:

```bash
npm run test:compat:update-xfail
```

## Release

Push a semver tag like `v0.1.0`.

GitHub Actions workflow `.github/workflows/release.yml` will:

1. Build each platform binary.
2. Pack and publish platform npm packages.
3. Publish the main package `markec`.
