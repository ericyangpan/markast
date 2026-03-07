#!/usr/bin/env bash
set -euo pipefail
MARKEC_WRITE_XFAIL=1 cargo test --test compat_marked -- --nocapture
