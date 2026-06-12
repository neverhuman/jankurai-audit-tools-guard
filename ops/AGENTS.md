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

- Do not inline CI commands into `.github/workflows/*.yml`; every workflow step
  must delegate to `bash ops/ci/<lane>.sh` so local runs and CI never drift
  (HLT-042 CI-local parity).
- Do not unpin a third-party GitHub Action; every `uses:` is pinned to a full
  40-character commit SHA.
- Do not hand-edit anything under `target/` (a generated zone).

## Proof lane

Changes here route to the security lane and a workflow lint:

```bash
bash scripts/ci-local.sh security
bash ops/ci/quality-gates.sh
```

See [`agent/test-map.json`](../agent/test-map.json) for the per-path routing and
[`agent/proof-lanes.toml`](../agent/proof-lanes.toml) for the runnable lanes.
