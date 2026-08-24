# PF-15-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Product citation: **P0 `/security` levels** — “Permissive preserves the shipping behavior and does not silently change existing policies.”
- Implementation commit reviewed: `a4f178fe15d2fd2f6cce697c69beeee02c1f0772`
- Tested code tree: `220af8dae884968f81d47e58c18c1aab8aec37f2`
- Reviewed paths: `codex-rs/security-policy/src/{level,bounded,lib}.rs`, crate/workspace manifests, Cargo lock, and Bazel target.
- Corrective implementation commit: none required.
- TUI applicability: none; this sprint has no interactive surface.

## Review result

The existing change is confined to the typed level, versioned settings, bounded policy text, and build registration authorized by PF-15. `SecurityLevel` accepts only `permissive`, `moderate`, and `aggressive`; unknown serialized values fail. `SecuritySettings::validate` rejects unsupported versions. Legacy absence is resolved by the existing configuration compatibility path and remains Permissive; explicit corrupt values are not converted to Permissive.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy
PASS — clippy completed without changing tracked files.

cd codex-rs && just fmt
PASS — formatting completed without changing tracked files.

cd codex-rs && just test -p codex-security-policy level
PASS — 1 test passed, 9 filtered out.

bazel test //codex-rs/security-policy:all
PASS — 1 Bazel test passed; crate library and test targets built successfully.
```

## Build parity

Cargo and Bazel expose the same `codex_security_policy` library surface from `codex-rs/security-policy/src/lib.rs`. No dependency metadata changed during reconciliation, so `just bazel-lock-update` was not required.
