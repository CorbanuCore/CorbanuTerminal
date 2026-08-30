# PF-34-S04 G1 registration handback

This lane deliberately did not edit shared registration surfaces. Jim Ricketts
owns the serialized G1 application and combined-tree reruns. Apply these changes
only after auditing collision order with PF-27-S03.

## Lane identity

- Lane base: `6a35712cd5731b191d875e8c6468f1abe23eb66e`
- Allocation commit: `8534a61d6a5b00f601972bba8afd35879d646657`
- Initial implementation commit: `bc0864e7dae2082cce276be8199ad4ac07781621`
- Post-review implementation commit: `002c6382152910d984db22a41a24f80281f6d19b`
- Contract version: `1`
- Fixture schema version: `1`
- Fixture manifest SHA-256: `7e8a4850f67052b2b5b2e0d17f5227116f226c65ee25a6945d04ff7a2a1a1fc3`

## Exact shared files to create

`codex-rs/content-security/Cargo.toml`:

```toml
[package]
name = "codex-content-security"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
doctest = false

[lints]
workspace = true

[dependencies]
sha2 = { workspace = true }

[dev-dependencies]
pretty_assertions = { workspace = true }
```

`codex-rs/content-security/src/lib.rs`:

```rust
//! Fail-closed external-content screening contracts for Corbanu Terminal.

mod contract;

pub use contract::ClassifierVerdict;
pub use contract::ContentAuthority;
pub use contract::ContentBinding;
pub use contract::ContentDigest;
pub use contract::ContentTaint;
pub use contract::ContractError;
pub use contract::ContractId;
pub use contract::DiagnosticCode;
pub use contract::ModelIdentity;
pub use contract::MAX_SCREENED_CONTENT_BYTES;
pub use contract::MAX_SCREENING_SEGMENTS;
pub use contract::SCREENING_CONTRACT_VERSION;
pub use contract::SCREENING_FIXTURE_SCHEMA_VERSION;
pub use contract::ScreenedContent;
pub use contract::ScreeningBudget;
pub use contract::ScreeningDecision;
pub use contract::ScreeningProgress;
pub use contract::ScreeningSession;
pub use contract::ScreeningTarget;
pub use contract::SegmentEnvelope;
pub use contract::SourceBinding;
pub use contract::ThresholdIdentity;
pub use contract::TransformationBinding;
pub use contract::UnavailableReason;
pub use contract::VerdictIdentity;
pub use contract::VerdictKind;
pub use contract::WithheldContent;
```

`codex-rs/content-security/BUILD.bazel`:

```starlark
load("//:defs.bzl", "codex_rust_crate")

codex_rust_crate(
    name = "content-security",
    compile_data = [
        "//:qa/security-levels/ingress-contract/fixtures/benign-v1/raw.txt",
        "//:qa/security-levels/ingress-contract/fixtures/benign-v1/rendered.txt",
        "//:qa/security-levels/ingress-contract/fixtures/benign-v1/sanitized.txt",
        "//:qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/raw.txt",
        "//:qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/rendered.txt",
        "//:qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/sanitized.txt",
    ],
    crate_name = "codex_content_security",
)
```

## Exact shared edits

1. Add `"content-security",` to the root workspace `members` array in
   `codex-rs/Cargo.toml`.
2. Add `codex-content-security = { path = "content-security" }` to
   `[workspace.dependencies]` in `codex-rs/Cargo.toml`.
3. Add a second root `BUILD.bazel` `exports_files` declaration containing the
   six fixture paths from `compile_data`, with visibility limited to
   `//codex-rs/content-security:__pkg__`. Do not change the existing exports or
   their visibility.
4. Regenerate `codex-rs/Cargo.lock` and `MODULE.bazel.lock`; do not hand-edit
   either lock.

The additional root export declaration is exactly:

```starlark
exports_files(
    [
        "qa/security-levels/ingress-contract/fixtures/benign-v1/raw.txt",
        "qa/security-levels/ingress-contract/fixtures/benign-v1/rendered.txt",
        "qa/security-levels/ingress-contract/fixtures/benign-v1/sanitized.txt",
        "qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/raw.txt",
        "qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/rendered.txt",
        "qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/sanitized.txt",
    ],
    visibility = ["//codex-rs/content-security:__pkg__"],
)
```

## Required combined-tree commands

Set all cache variables to the recorded CorbanuDrive cache root, then run in
this order:

```text
cd codex-rs && just fix -p codex-content-security
cd codex-rs && just fmt
cd codex-rs && just test -p codex-content-security pf_34_s04
cd codex-rs && just test -p codex-content-security
just bazel-lock-update
bazel test //codex-rs/content-security:all
python3 qa/security-levels/ingress-contract/verify.py
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Confirm that 20 named Rust tests and seven fixture files run, not merely that
the commands exit successfully. After the combined tree passes, record the
integration commit, update the sprint evidence/ledgers, archive PF-34-S04, and
only then allocate PF-35-S01.
