#!/usr/bin/env bash
# Required lane: the gate that must pass on every push.
# Verifies the workspace manifest resolves against the locked dependency
# graph, then formats, lints and tests it. Every cargo invocation is
# --locked --offline so the lane proves the committed lockfile is complete
# and cannot silently resolve a different dependency graph.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

log "required lane: cargo metadata"
cargo metadata --locked --offline --no-deps --format-version 1 >/dev/null

log "required lane: cargo fmt --check"
cargo fmt --all --check

log "required lane: cargo clippy"
cargo clippy --workspace --all-targets --locked --offline -- -D warnings

log "required lane: cargo nextest run"
cargo nextest run --workspace --locked --offline
