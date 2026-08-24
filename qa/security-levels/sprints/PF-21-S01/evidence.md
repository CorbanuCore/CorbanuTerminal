# PF-21-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Baseline commit: `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`
- Original implementation: `220af8dae8`
- Corrective implementation: `fcaa84dfb8`
- Tested code tree: `fcaa84dfb86758ac8872fe1cc9a9d2a7533e49e6`
- Baseline manifest SHA-256: `2effbae28c1479009179996f6748daa712c757e4b9404a0ae36bfc7b2325115a`
- Candidate: `corbanu 0.1.35`
- Candidate SHA-256: `364fa6b33314f4f6b6aada25dbc79b679a9a6d1e292668994fff8c541596052f`
- TUI applicability: deferred to PF-26-S02; this sprint freezes automated compatibility evidence.

## Review result

The first baseline covered representative decisions but did not bind them to an independently frozen executable contract. It also underrepresented approval modes, sandbox profiles, network states, spawn-depth boundaries, the full credential-type matrix, tool decisions, and missing legacy security configuration.

Corrective commit `fcaa84dfb8` adds a fail-closed compatibility harness and expands the immutable baseline to ten policy surfaces. Each candidate probe is tied to a package, test selector, exact test function, and frozen source SHA-256. The harness validates the manifest and source hashes before it invokes any candidate test, identifies the actual candidate binary by version and SHA-256, runs every probe under an isolated temporary directory, and writes its JSON report atomically. The candidate therefore cannot rewrite its own expected behavior without making the compatibility run fail before testing.

Permissive now has explicit adjacent-case coverage for the configured approval/sandbox/network matrix, absent security state, every vault credential type, agent spawn depths on both sides of each limit, and existing tool-policy denial. Every frozen surface retains the composition rule `final_allow = existing_allow && security_layer_allow` with an allow-neutral Permissive security decision.

The harness report is [compatibility-report.json](harness/compatibility-report.json).

## Final-tree commands

```text
cd codex-rs && just fix -p codex-core && just fix -p codex-vault
PASS — clippy completed; only pre-existing Core dead-code warnings were emitted.

cd codex-rs && just fmt
PASS — formatting completed; final diff inspected.

cd codex-rs && just test -p codex-core permissive
PASS — 3 tests passed, 3,368 filtered out.

cd codex-rs && just test -p codex-vault permissive
PASS — 1 test passed, 22 filtered out.

cd codex-rs && just test -p codex-security-policy permissive_composition_preserves_every_frozen_surface_decision
PASS — 1 test passed, 13 filtered out.

python3 -m unittest scripts/test_security_level_compat.py -v
PASS — 4 harness unit tests passed.

cargo build -p codex-cli --bin corbanu
PASS — built the candidate binary used below.

python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --candidate codex-rs/target/debug/corbanu --output qa/security-levels/sprints/PF-21-S01/harness
PASS — all 5 immutable probes passed across 10 frozen policy surfaces.
```

## Changed paths

- `codex-rs/core/src/agent/registry_tests.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
- `codex-rs/vault/src/tests.rs`
- `qa/security-levels/permissive-baseline-v1.json`
- `scripts/security-level-compat`
- `scripts/security_level_compat.py`
- `scripts/test_security_level_compat.py`
