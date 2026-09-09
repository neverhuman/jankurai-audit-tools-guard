# jankurai-tools-guard

<!-- jankurai-badge:start -->
[![Jankurai score: 91/100](agent/jankurai-badge.svg)](agent/jankurai-badge.json)
<!-- jankurai-badge:end -->

[![ci](https://github.com/neverhuman/jankurai-tools-guard/actions/workflows/ci.yml/badge.svg)](https://github.com/neverhuman/jankurai-tools-guard/actions/workflows/ci.yml)
[![jankurai score](https://img.shields.io/badge/jankurai%20score-passing-brightgreen)](.jankurai/repo-score.md)

Guarded filesystem and save-gate runtime for the **jankurai** auditor. This
repository is one member of the Jankurai split family; read [`SPLIT.md`](SPLIT.md)
for the family contract and [`AGENTS.md`](AGENTS.md) for agent routing rules.

The `jankurai-guard` crate makes a failed audit impossible for an AI agent to
miss: the moment an agent writes a file, that single file is audited and, if it
fails, the agent is forced to see the failure before moving on. See
[`docs/guard.md`](docs/guard.md) for the full design.

## Stack

Rust core + TypeScript/React/Vite product surface + PostgreSQL truth + generated
contracts + exception-only Python AI/data service. New implementation is
Rust-first; see [`docs/architecture.md`](docs/architecture.md).

## Quick start

```bash
# One-command setup (toolchain + locked dependencies).
just setup

# Deterministic fast lane (check + tests).
just fast

# Full local check: format, lint, fast, security, and self-audit.
just check
```

The full command surface lives in the root [`Justfile`](Justfile). Continuous
integration runs the same lanes under
[`.github/workflows/ci.yml`](.github/workflows/ci.yml).

## Layout

| Path | Role |
| --- | --- |
| `crates/jankurai-guard` | Rust guard crate: watcher, FUSE, PTY, transaction state |
| `agent/` | machine-readable owner, test, boundary, and proof maps |
| `docs/` | guard design, architecture, testing, boundaries, release, and exception docs |
| `ops/` | pinned CI script entrypoints |
| `scripts/` | local CI helpers |

## Documentation

- [Guard design](docs/guard.md)
- [Architecture](docs/architecture.md)
- [Testing](docs/testing.md)
- [Boundaries](docs/boundaries.md)
- [Release process](docs/release.md)
- [Agent exceptions and overrides](docs/exceptions.md)

## Versioning

The current version is recorded in [`VERSION`](VERSION) and the change history in
[`CHANGELOG.md`](CHANGELOG.md). Release mechanics are documented in
[`docs/release.md`](docs/release.md).

## License

See [`LICENSE`](LICENSE).
