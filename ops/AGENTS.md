# ops Agent Instructions

This directory owns the operational control plane for jankurai-tools-guard: the
pinned CI lane scripts and the git-hook entrypoints.

## Owns

- `ops/ci/*.sh` — the lane scripts (`fast`, `security`, `audit`, `required`,
  `quality-gates`, `tool-adoption`) that both CI and `scripts/ci-local.sh` call,
  plus the shared `ops/ci/lib.sh` helper module.
- `ops/git-hooks/pre-push` — the mandatory pre-push gate that runs
  `bash ops/ci/quality-gates.sh`.

## Forbidden

- Do not add GitHub Actions workflows; GitHub is a publishing mirror only. CI
  runs on the forge and our hosts and must delegate to `bash ops/ci/<lane>.sh`
  so local runs and CI never drift (HLT-042 CI-local parity).
- Do not hand-edit anything under `target/` (a generated zone).

## Proof lane

Changes here route to the security lane and the quality gates:

```bash
bash scripts/ci-local.sh security
bash ops/ci/quality-gates.sh
```

See [`agent/test-map.json`](../agent/test-map.json) for the per-path routing and
[`agent/proof-lanes.toml`](../agent/proof-lanes.toml) for the runnable lanes.
