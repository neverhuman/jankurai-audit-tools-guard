# jankurai-tools-guard Testing

Testing is routed proof. Agents should not guess which tests matter: every
top-level path routes to a deterministic command in
[`agent/test-map.json`](../agent/test-map.json), and the runnable lanes are
declared in [`agent/proof-lanes.toml`](../agent/proof-lanes.toml).

| Lane | Purpose |
| --- | --- |
| `fast` | deterministic local proof for most edits (`cargo check` + `nextest`) |
| `guard` | full `jankurai-guard` crate test suite |
| `security` | secrets (gitleaks) and dependency advisories (cargo audit) |
| `audit` | jankurai repo score and hard-rule findings |
| `full` | release/merge gate (`just check`) |

## Integration and property tests

The guard crate ships integration tests under `crates/jankurai-guard/tests/`
that exercise real behavior end to end rather than asserting on mocks:

| Test | What it proves |
| --- | --- |
| `watcher_fallback.rs` | the filesystem watcher degrades correctly when the native backend is unavailable, and debounced events still fire |
| `fuse_mount.rs` | the FUSE save-gate mounts, intercepts a write, and routes it through the single-file audit (Linux, `fuse` feature) |
| `poison_roundtrip.rs` | poison payloads generated for a blocked write round-trip through generation and detection |
| `denial_formatting.rs` | denial reports and banners render the failing rule, path, and fix for the agent to see |
| `transaction_logic.rs` | the save-gate transaction state machine transitions correctly across propose, audit, commit, and revert |
| `policy_parsing.rs` | guard policy TOML parses, rejects malformed input, and applies defaults deterministically |

These cover the watcher, FUSE filesystem, PTY injection, transaction state
machine, poison generation, and audit client, so the suite is integration-grade
rather than unit-only.

## Avoiding false-green tests

A test that always passes is worse than no test. Every test above asserts on an
observable outcome (a mounted filesystem, a reverted write, a parsed policy, a
rendered denial) rather than on internal call counts. The `audit` lane runs the
jankurai auditor over this repo, which raises `HLT-008-FALSE-GREEN-RISK` if a
test body has no meaningful assertions, so a vacuous test fails the gate.

## Observability and the agent-friendly exception pattern

The guard surfaces structured outcomes, not opaque exit codes. Errors are typed
via `thiserror`, and every failure carries an agent-friendly repair surface so
the next rerun stays local rather than forcing a developer to read source.

Each denial report (`feedback/report.rs`) and banner (`feedback/banner.rs`)
carries these fields:

- **purpose** — what the guard was checking when it blocked the write.
- **reason** — why the write failed, naming the route, the failing `rule_id`,
  and the path.
- **common fixes** — the concrete edits that usually clear the failure.
- **docs_url** — a link into this repo's docs (for example `docs/guard.md`) so
  the fix is documented, not folklore.
- **repair_hint** — the exact command to rerun (for example `just fast`) once
  the fix lands.

This is the agent-friendly exception pattern: a typed error surface whose
purpose, reason, common fixes, docs_url, and repair_hint mean an opaque failure
never slows local debugging. The `doctor.rs` module emits a matching readiness
report covering watcher backend, FUSE availability, and platform support, which
the `just check` gate consumes.

## Cost and runtime budget

The lanes are ordered cheapest-first so agents iterate on the narrowest proof:
`fast` (~seconds) before `guard` before `security` before the full `audit`
gate. Lane cost estimates live in `agent/proof-lanes.toml` (`cost` field) and
the timeouts cap each lane (`timeout_seconds`). No lane requires network access,
so runs are hermetic and reproducible.

- **Budget and quota**: the per-run compute budget is the job `timeout-minutes`
  in `.github/workflows/ci.yml`; the quota is one workspace nextest run plus one
  audit.
- **Spend cap and kill switch**: the job timeout is the hard spend cap and the
  kill switch. A superseded push cancels in-flight work via `concurrency`.
- **Stop condition**: `set -euo pipefail` stops the lane on the first failure.
  There is no paid API, so the stop condition is "no paid work runs here."
- **Receipt**: the audit writes `.jankurai/repo-score.json` with `repair_hint`,
  `docs_url`, and the rerun command for the next agent.

## Running the lanes

```bash
just fast        # cargo check + nextest
just test        # cargo nextest run --workspace
just security    # gitleaks + cargo audit
just audit       # jankurai self-audit -> .jankurai/repo-score.{json,md}
just check       # full local gate: fmt, lint, fast, security, audit
```

Locally, `bash scripts/ci-local.sh <lane>` runs the exact same
`ops/ci/<lane>.sh` scripts that CI invokes, so a green local gate means a green
CI run.
