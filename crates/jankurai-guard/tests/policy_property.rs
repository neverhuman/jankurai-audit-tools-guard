//! Property tests for `jankurai_guard::policy`.
//!
//! These exercise invariants that must hold for *every* well-formed policy, not
//! just the hand-picked fixtures in `policy_parsing.rs`. To keep the crate's
//! dependency tree minimal and its `Cargo.lock` hermetic, this file deliberately
//! does not pull in the `proptest`, `quickcheck`, or `rstest` crates; it uses a
//! small, fully deterministic generator (a splitmix64 PRNG seeded from a fixed
//! constant) so the sweep is reproducible bit-for-bit on every run. Each test
//! generates hundreds of distinct policies and asserts a universal property:
//!
//! 1. **TOML round-trip**: serializing a valid policy and parsing it back yields
//!    an equal policy. Encoding never loses or mutates configuration.
//! 2. **Validation total**: `validate()` accepts every policy built from valid
//!    components and never panics or rejects a valid one.
//! 3. **Exclusion monotonicity**: once a prefix is in `extra_excluded_paths`,
//!    every path under that prefix is excluded.

use jankurai_guard::policy::{AuditScope, GuardPolicy, HardeningPolicy, OnFail, PathPolicy};
use jankurai_guard::GuardMode;

/// Deterministic splitmix64 PRNG. Seeded from a fixed constant so the property
/// sweep is reproducible; no external `rand`/`proptest` dependency is required.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Returns a value in `0..n`.
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    fn bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

const SEVERITIES: [&str; 4] = ["critical", "high", "medium", "low"];

/// Generates an arbitrary but always-valid policy from the PRNG.
fn arb_policy(rng: &mut Rng) -> GuardPolicy {
    let pick_sevs = |rng: &mut Rng| {
        let count = rng.below(4) as usize;
        (0..count)
            .map(|_| SEVERITIES[rng.below(4) as usize].to_string())
            .collect::<Vec<_>>()
    };
    let excl = (0..rng.below(5))
        .map(|i| format!("dir{i}/"))
        .collect::<Vec<_>>();
    GuardPolicy {
        schema_version: "1.0.0".to_string(),
        mode: if rng.bool() {
            GuardMode::Enforce
        } else {
            GuardMode::Observe
        },
        block_on: pick_sevs(rng),
        warn_on: pick_sevs(rng),
        debounce_ms: 1 + rng.below(10_000),
        stable_ms: rng.below(10_000),
        on_fail: match rng.below(4) {
            0 => OnFail::Warn,
            1 => OnFail::Interrupt,
            2 => OnFail::Revert,
            _ => OnFail::Poison,
        },
        quarantine_new_files: rng.bool(),
        respect_gitignore: rng.bool(),
        audit_scope: if rng.bool() {
            AuditScope::FileOnly
        } else {
            AuditScope::FilePlusControl
        },
        fail_closed: rng.bool(),
        max_audit_ms: 1 + rng.below(60_000),
        paths: PathPolicy {
            extra_excluded_paths: excl,
        },
        hardening: HardeningPolicy::default(),
    }
}

/// Runs `check` over `cases` generated policies, panicking on the first failure
/// with the seed so a counterexample is reproducible.
fn for_all(cases: u64, mut check: impl FnMut(&GuardPolicy, u64)) {
    for seed in 0..cases {
        let mut rng = Rng::new(0xC0FFEE ^ seed.wrapping_mul(0x100000001B3));
        let policy = arb_policy(&mut rng);
        check(&policy, seed);
    }
}

#[test]
fn toml_round_trip_is_identity() {
    for_all(500, |policy, seed| {
        let encoded = toml::to_string(policy).expect("policy serializes to TOML");
        let decoded: GuardPolicy =
            toml::from_str(&encoded).expect("encoded policy parses back to a policy");
        assert_eq!(
            &decoded, policy,
            "round-trip mismatch for seed {seed}:\n{encoded}"
        );
    });
}

#[test]
fn valid_components_always_validate() {
    for_all(500, |policy, seed| {
        assert!(
            policy.validate().is_ok(),
            "policy built from valid components failed validation (seed {seed}): {policy:?}"
        );
    });
}

#[test]
fn excluded_prefixes_cover_children() {
    for_all(500, |policy, seed| {
        for prefix in &policy.paths.extra_excluded_paths {
            let trimmed = prefix.trim_end_matches('/');
            if trimmed.is_empty() {
                continue;
            }
            for tail in ["", "a", "nested/deep/file.rs", "x.txt"] {
                let child = format!("{trimmed}/{tail}");
                assert!(
                    policy.is_excluded(&child),
                    "child `{child}` of excluded prefix `{prefix}` not excluded (seed {seed})"
                );
            }
        }
    });
}
