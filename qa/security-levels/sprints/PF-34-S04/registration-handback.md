# PF-34-S04 G1 registration handback

This lane deliberately did not edit shared registration surfaces. Jim Ricketts
owns the serialized G1 application and combined-tree reruns. Apply these changes
only after auditing collision order with PF-27-S03.

## Lane identity

- Lane base: `6a35712cd5731b191d875e8c6468f1abe23eb66e`
- Latest `main` rebase used for lane verification:
  `1a5562738cb3d53bd4d0b6668761cfe76bd4b93e`
- Allocation commit: `6a614e8afa825267b329b3274fd3cba648e99151`
- Initial implementation commit: `9c838e16bbda9726e6c352d57d9223e098dc817f`
- Pre-Opus-review implementation commit:
  `475ed05147fb8801672556048014a3ae28bcba86`
- Opus-remediated implementation commit:
  `74e97148701ef541ff9ef2d0a9194ba472b2801c`
- Final redacted-debug remediation commit:
  `a75efecc0a37d5544e123ad19d57867cac360a68`
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
pub use contract::MAX_SCREENED_CONTENT_BYTES;
pub use contract::MAX_SCREENING_ELAPSED_MS;
pub use contract::MAX_SCREENING_SEGMENTS;
pub use contract::MAX_VERDICT_AGE_MS;
pub use contract::ModelIdentity;
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
pub use contract::UntrustedBytes;
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
   The lane verified at rebase `1a5562738cb3d53bd4d0b6668761cfe76bd4b93e`
   that `sha2` and `pretty_assertions` already exist in that table. Confirm
   both are still present before applying the crate manifest; if either was
   removed, restore it at the workspace-pinned version rather than adding a
   crate-local version.
3. Add a second root `BUILD.bazel` `exports_files` declaration containing the
   six fixture paths from `compile_data`, with visibility limited to
   `//codex-rs/content-security:__pkg__`. Do not change the existing exports or
   their visibility.
4. Regenerate `codex-rs/Cargo.lock` and `MODULE.bazel.lock`; do not hand-edit
   either lock.

`contract_tests.rs` contains the six `include_bytes!` calls under `#[cfg(test)]`.
Before accepting the Bazel recipe, confirm that `codex_rust_crate` makes its
library `compile_data` available while compiling the generated unit-test
target. If it does not, give the unit-test target its own compile-data input (or
use the macro's equivalent test compile-data parameter). Do not accept a
library-only compile-data attachment. The durable alternative is a later
contract-preserving conversion to `codex_utils_cargo_bin::find_resource!` with
runtime test data, but that is not part of this lane's shared G1 patch.

The root-package labels below rely on there being no nested `BUILD` or
`BUILD.bazel` beneath `qa/security-levels/`. If a nested Bazel package is added,
move the exports to that owning package and update the labels in the same
change; otherwise the test resource becomes unreachable.

The accepted current-sprint scope exception keeps the frozen v1 surface in the
two allocated Rust files. The first post-G1 change that touches `contract.rs`
must receive a new disjoint allocation and split identity, segment, and session
implementation into separate modules while preserving these public re-exports.
The Opus remediation brought the module slightly above the approximate
800-line target; do not grow the monolith further.

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
cd codex-rs && just argument-comment-lint
just bazel-lock-update
bazel test //codex-rs/content-security:all
git check-attr text eol -- qa/security-levels/ingress-contract/fixtures/benign-v1/raw.txt
python3 qa/security-levels/ingress-contract/verify.py
python3 qa/security-levels/ingress-contract/test_verify.py
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Confirm that 20 named Rust tests, 14 verifier regression tests, and seven
fixture files run, not merely that the commands exit successfully. Confirm the
fixture attribute reports `text: unset`; its pinned bytes must not be converted.
After the combined tree passes, record the integration commit, update the
sprint evidence/ledgers, archive PF-34-S04, and only then allocate PF-35-S01.

G1 must also wire both `qa/security-levels/ingress-contract/verify.py` and
`test_verify.py` into a recurring cross-platform CI job. A one-time integration
run is not durable drift protection; the Windows leg must exercise the byte
attributes and verifier behavior as well as the Rust contract tests.
