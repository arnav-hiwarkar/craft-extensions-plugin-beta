#!/usr/bin/env bash
# No catalogue/signing key or desktop needed. Artifacts/logs retained in a temporary folder.
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS=4
python3 scripts/cross-app-smoke.py "$@"
