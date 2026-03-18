# Releasing `markast`

This repository publishes to npm via GitHub Actions (not from a local machine).

## Normal release flow

1. Bump versions:
   - `package.json`
   - `Cargo.toml`
   - `npm/*/package.json` (run `npm run sync:npm-versions`)
   - `package-lock.json`
   - `Cargo.lock`
2. Run gates:
   - `npm run check:npm-versions`
   - `npm run check:strict`
3. Commit to `main`.
4. Create a tag `vX.Y.Z` that matches `package.json` and push it.
5. Watch the `Release` workflow (`.github/workflows/release.yml`).

## Related workflows

- `Release` (`.github/workflows/release.yml`): builds and publishes platform packages + the main package.
- `Smoke (npm)` (`.github/workflows/smoke-npm.yml`): manually smoke-test any published `markast` version.
- `npm deprecate` (`.github/workflows/npm-deprecate.yml`): deprecate a bad version (requires npm auth).

## Security hardening

### Recommended: npm trusted publishing (OIDC)

npm supports tokenless publishing from GitHub Actions using OIDC ("trusted publishing").
Docs: https://docs.npmjs.com/trusted-publishers/

Configure trusted publishing for each package:

- `markast`
- `markast-darwin-arm64`
- `markast-darwin-x64`
- `markast-linux-arm64-gnu`
- `markast-linux-x64-gnu`
- `markast-win32-x64-msvc`

When adding the GitHub Actions trusted publisher in npm package settings, use:

- Organization/user: `ericyangpan`
- Repository: `markast`
- Workflow filename: `release.yml`
- Environment name: `npm` (matches the workflow job environment)

After you verify an OIDC-based release works, consider locking down token publishing in npm:

- Package settings → Publishing access → require 2FA and disallow tokens
- Revoke any old automation tokens

### Tokens (still needed for `npm deprecate`)

Trusted publishing currently applies to `npm publish`. Other write operations (like `npm deprecate`) still
require traditional authentication.

If you keep a write token:

- Use a granular token with 2FA enabled
- Rotate it regularly (write tokens have a 90-day max lifetime)
- Prefer storing it as an **environment** secret for the GitHub Actions environment `npm` (name: `NPM_TOKEN`)

