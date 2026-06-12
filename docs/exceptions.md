# Agent exceptions and overrides

This document defines the agent-friendly exception pattern for
jankurai-tools-guard: how an agent or maintainer requests, records, and bounds an
override of a standard rule. Exceptions are the only sanctioned way to deviate
from the audit baseline.

## Principle

The default answer is "follow the standard." An exception is a dated, owned,
expiring waiver for a specific rule on a specific path. Exceptions are data, not
prose: they live next to the code they govern and are reviewed on every audit.

## How to request an exception

1. Identify the exact `rule_id` and `path` the exception applies to (from the
   audit JSON `findings[]`).
2. Add an entry to the relevant `agent/*.toml` manifest. For boundary
   reclassifications use `agent/boundaries.toml`; for scan scope use the
   `[scan]` block of `agent/audit-policy.toml`.
3. Every exception entry MUST carry:
   - `owner` — the team or person accountable.
   - `classification` — e.g. `brownfield`, `temporary`, `vendor`.
   - `expires` — an ISO date after which the exception is invalid and the audit
     fails again.
   - `migration_path` — the concrete plan to remove the exception.

## Example

A bounded boundary reclassification is recorded in
[`agent/boundaries.toml`](../agent/boundaries.toml) as a dated, owned block:

```toml
[[boundary_exception]]
path = "crates/jankurai-guard/src/platform"
classification = "temporary"
owner = "guard"
expires = "2026-12-31"
migration_path = "Keep platform shims behind the cfg-gated module until the
fanotify and landlock tiers land, then re-scope."
```

## Override review

- Every exception is re-evaluated on each `just audit` run.
- An expired exception is treated as a hard finding, not a pass.
- Removing an exception requires deleting its entry and proving the underlying
  rule now passes on its own.

## What is never excepted

Secret leakage, destructive migrations without rollback, and hand-edits to
generated zones are never granted exceptions. Fix the underlying cause instead.
