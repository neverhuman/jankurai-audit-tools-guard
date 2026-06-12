# jankurai-tools-guard Architecture

jankurai-tools-guard is the guard/save-gate runtime of the jankurai family. It
ships a single Rust crate, `jankurai-guard`, that audits every agent file write
as a single-file jankurai candidate and forces a failing agent to see and fix
the failure before moving on. The full product design is documented in
[`guard.md`](guard.md).

The family product standard is:

```text
Rust core + TypeScript/React/Vite product surface + PostgreSQL truth
+ generated contracts + exception-only Python AI/data service
```

New implementation should be Rust-first. Agents must not write Python for repo
tools, proof lanes, product services, general backend glue, authorization, or
production database writes. This repo is pure Rust: there is no web surface, no
PostgreSQL database, and no Python AI/data service, so those stack arms are not
applicable here.

## Crate layout

The guard crate lives under `crates/jankurai-guard/src`:

| Module | Role |
| --- | --- |
| `watch/` | filesystem watcher backend and debounce |
| `fuse/` | FUSE save-gate filesystem (Linux, behind the `fuse` feature) |
| `pty/` | PTY launcher and denial-banner injector |
| `transaction.rs` / `state.rs` / `commit.rs` | save-gate transaction state machine |
| `poison/` | poison-payload generation for blocked writes |
| `audit_client/` | client that runs the single-file jankurai audit |
| `policy.rs` / `layout.rs` / `status.rs` | policy parsing, repo layout, status |
| `cli/` | command-line surface (`args`, `handlers`, `prompt`) |
| `feedback/` | denial reports and banners |
| `platform/` | Linux and macOS platform shims |
| `doctor.rs` | environment readiness checks |

## Ownership and proof

Agents should prefer `agent/owner-map.json` and `agent/test-map.json` for
changes, then route to the smallest proof lane in `agent/proof-lanes.toml`.
Boundary rules are declared in `agent/boundaries.toml` with prose in
[`boundaries.md`](boundaries.md).
