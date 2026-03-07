#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REF="${1:-22f0c555375becb1eda9406a2975e71a266637cb}"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

curl -L -o "$TMPDIR/marked.tgz" "https://codeload.github.com/markedjs/marked/tar.gz/$REF"
mkdir -p "$TMPDIR/extracted"
tar -xzf "$TMPDIR/marked.tgz" -C "$TMPDIR/extracted"
TOP="$(find "$TMPDIR/extracted" -maxdepth 1 -type d -name 'marked-*' | head -n 1)"

rm -rf "$REPO_ROOT/third_party/marked"
mkdir -p "$REPO_ROOT/third_party/marked"
cp -R "$TOP/test" "$REPO_ROOT/third_party/marked/"
cp "$TOP/LICENSE.md" "$REPO_ROOT/third_party/marked/"
cp "$TOP/package.json" "$REPO_ROOT/third_party/marked/"

cat > "$REPO_ROOT/third_party/marked/VERSION" <<META
ref=$REF
source=https://github.com/markedjs/marked
synced_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
META

echo "synced marked specs at ref=$REF"
