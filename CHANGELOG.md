# Changelog

All notable changes to jankurai-tools-guard are documented in this file. The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The authoritative version string lives in [`VERSION`](VERSION).

## [Unreleased]

### Removed

- GitHub Actions workflows and the GitHub-only `ops/ci/aggregate.sh` and
  workflow-lint (actionlint, zizmor) steps. GitHub is a publishing mirror only;
  CI runs on the forge and our hosts.

### Added

- Root `Justfile` command surface with `setup`, `fast`, `check`, `security`, and
  `audit` lanes for one-command setup and validation.
- GitHub Actions CI (`.github/workflows/ci.yml`) with build, security, and
  jankurai audit jobs, all third-party actions pinned to commit SHAs.
- Agent-readable documentation: `README.md`, `docs/architecture.md`,
  `docs/testing.md`, `docs/boundaries.md`, `docs/release.md`, and
  `docs/exceptions.md`.

### Changed

- Re-scoped `agent/owner-map.json`, `agent/test-map.json`, and
  `agent/generated-zones.toml` to the paths that exist in this single-purpose
  guard repo, and added `agent/audit-policy.toml`, `agent/boundaries.toml`, and
  `agent/proof-lanes.toml`.

## [1.7.0] - 2026-06-12

### Added

- Initial split-family extraction of the jankurai-guard crate: filesystem
  watcher, FUSE save-gate, PTY injection, transaction state machine, poison
  generation, and audit client.
