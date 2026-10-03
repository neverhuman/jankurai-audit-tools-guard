#!/usr/bin/env bash
# Canonical security lane wrapper for jankurai-tools-guard.
#
# Single shell entrypoint for the high-risk security posture: secret scanning,
# dependency advisories, software bill of materials (SBOM), and provenance.
# The Justfile `security` lane and CI both call this script so local runs and
# the forge-hosted CI execute the exact same checks. Each step emits a
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
mkdir -p target/jankurai/security
cargo cyclonedx --manifest-path crates/jankurai-guard/Cargo.toml --all-features \
  --format json --override-filename cargo-sbom
mv crates/jankurai-guard/cargo-sbom.json target/jankurai/security/cargo-sbom.json
syft dir:. -o cyclonedx-json=target/jankurai/security/sbom-syft.json
grype sbom:target/jankurai/security/sbom-syft.json --fail-on high

# 5. Deterministic provenance inputs for the separately signed release lane.
step "provenance"
sha256sum Cargo.lock target/jankurai/security/cargo-sbom.json \
  target/jankurai/security/sbom-syft.json \
  > target/jankurai/security/provenance-inputs.sha256
