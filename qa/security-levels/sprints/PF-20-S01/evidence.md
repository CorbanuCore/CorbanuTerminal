# PF-20-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Original implementation reviewed: `0e3f2dfd92`
- Tested code tree: `655a66c17322d046801825f1b230331bc09dc0b4`
- Corrective implementation commit: none required.
- TUI applicability: none.

## Review result

Missing legacy `[security]` state resolves to `SecurityLevel::Permissive`. Explicit Permissive, Moderate, and Aggressive values round-trip through the versioned table. Unknown levels and malformed explicit state fail during typed TOML parsing; unsupported versions fail with `InvalidData` while effective config is built. `ConfigEditsBuilder::set_security_level` batches version and level mutations through the existing `write_atomically` path, preserving unrelated configuration.

`just write-config-schema` reproduced `codex-rs/core/config.schema.json` without a tracked diff. Cargo/Bazel dependency metadata did not change during reconciliation, so no lock update was required.

The first broad Core config run encountered two project-layer failures because this host has an external `/tmp/.codex` directory, which those tests discover while using the default temp root. The same two cases passed independently and the entire 477-test Core config filter passed with a clean fixed `TMPDIR`; this records the environment rather than suppressing the tests.

## Final-tree commands

```text
cd codex-rs && just write-config-schema
PASS — generated schema matched the checked-in file.

cd codex-rs && just fix -p codex-config && just fix -p codex-core
PASS — clippy completed; only pre-existing Core dead-code warnings were emitted.

cd codex-rs && just fmt
PASS — formatting completed without tracked changes.

cd codex-rs && just test -p codex-config
PASS — 229 tests passed.

cd codex-rs && TMPDIR=/var/tmp/corbanu-pf20-tests just test -p codex-core config::
PASS — 477 tests passed, 2,894 filtered out.

cd codex-rs && TMPDIR=/var/tmp/corbanu-pf20-tests just test -p codex-core security_state
PASS — 4 security-state tests passed.

cd codex-rs && TMPDIR=/var/tmp/corbanu-pf20-tests just test -p codex-core set_security_level
PASS — 1 atomic persistence test passed.
```

The fixed temporary directory was removed after the run.
