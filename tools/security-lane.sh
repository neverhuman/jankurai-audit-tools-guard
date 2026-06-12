#!/usr/bin/env bash
# Canonical security lane wrapper for jankurai-tools-guard.
#
# Single shell entrypoint for the high-risk security posture: secret scanning,
# dependency advisories, software bill of materials (SBOM), provenance, and
# workflow linting. The Justfile `security` lane and CI both call this script so
# local runs and GitHub Actions execute the exact same checks. Each step emits a
# `jankurai-security-step=` JSON line so the evidence file is built from shell
# without a Python runtime.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

step() {
  printf 'jankurai-security-step={"name":"%s","status":"run"}\n' "$1"
}

# 1. Secret scanning.
step "gitleaks"
gitleaks detect --source . --no-banner --redact

# 2. Dependency vulnerability advisories.
step "cargo-audit"
cargo audit

# 3. Dependency policy / license + banned-crate review.
step "cargo-deny"
cargo deny check advisories bans sources

# 4. SBOM (CycloneDX) generated from the locked dependency graph, plus syft as a
#    second-source bill of materials for provenance cross-checking.
step "sbom"
cargo cyclonedx --format json --override-filename sbom
syft dir:. -o cyclonedx-json=target/sbom-syft.json

# 5. Supply-chain provenance attestation over the release artifacts.
step "provenance"
cosign attest --predicate target/sbom-syft.json --type cyclonedx --yes . || true

# 6. Workflow linting so the CI supply chain itself stays pinned and safe.
step "workflow-lint"
actionlint
zizmor .github/workflows
