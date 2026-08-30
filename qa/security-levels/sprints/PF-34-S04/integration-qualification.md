# PF-34-S04 transferred G1/G2 integration qualification

Date: 2026-08-30.

## Coordinates

- Immutable lane creation base: `6a35712cd5731b191d875e8c6468f1abe23eb66e`
- Registered integration checkpoint reviewed by Opus:
  `279ce48a9e8d3b28ab518ff184aae770d7462d2f`
- Reconciled current `main`: `3232f5e65bae60bc86122a5495ebb4c280f7c8fb`
- Current-main merge: `158b9b0ebe4b06a81c98be6a58a0d1c7919a0d08`
- Integration guard hardening: `1ddd8e1c972463c9bcaf47db796322718c78187c`
- Cache root:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`

## Passing final-tree checks

All cache, target, temporary, Bazel user-root and Python package data used the
recorded cache root.

| Check | Outcome |
| --- | --- |
| `cd codex-rs && just fix -p codex-content-security` | pass |
| `cd codex-rs && just fmt` | pass |
| `cd codex-rs && cargo clippy -p codex-content-security --all-targets -- -D warnings` | pass |
| `cd codex-rs && just test -p codex-content-security pf_34_s04` | 21 passed, 0 failed, 0 skipped |
| `cd codex-rs && just test -p codex-content-security` | 21 passed, 0 failed, 0 skipped |
| `bazel --output_user_root=… test //codex-rs/content-security:all` | 1 Bazel test target passed; all 21 Rust tests compiled and ran |
| targeted Bazel `--config=argument-comment-lint` over the content-security library and unit-test binary | pass |
| `python3 qa/security-levels/ingress-contract/verify.py` | 7 fixtures verified; schema 1; contract 1 |
| `python3 qa/security-levels/ingress-contract/test_verify.py` | 14 passed, 0 failed |
| Draft 2020-12 `jsonschema==4.25.1` validation | schema valid; manifest valid |
| `git check-attr text eol -- …/benign-v1/raw.txt` | `text: unset`; `eol: lf` |
| `shasum -a 256 -c qa/security-levels/sprints/PF-34-S04/lane-files.sha256` | all 15 files pass |
| `python3 qa/security-levels/sprints/PF-34-S04/verify_evidence.py` | 15 hashes and narrative identities agree |
| `bazel --output_user_root=… mod deps --lockfile_mode=update` | pass; `MODULE.bazel.lock` remained byte-identical at `c8d7e3f8c8bec8f8e71cc3d1d39fcb952eec0f07a41cdac48401a6f64a60d979` |
| `python3 docs/plans/check.py` | pass |
| `python3 docs/sprints/check.py` | pass |
| workflow YAML parse and Python byte compilation | pass |
| `git diff --check` | pass |

## Scoped current-main baseline

- Repository-wide argument-comment lint reports only the existing missing
  `parent_grant_id` and `parent_scope_digest` call-site comments in
  `codex-rs/security-policy/src/grant.rs:201-202`. PF-34-S04 does not modify
  that crate; the targeted content-security invocation passes.
- Cargo shear reports no content-security issue. Its remaining warnings are
  eight empty Core security placeholders, one unlinked network-proxy contract
  file and two empty TUI security placeholders inherited from current `main`.
- Existing Bazel CI runs `//...`, so the new
  `//codex-rs/content-security:all` target is selected by repository CI in
  addition to its local parity run.

No protected-ingress consumer, runtime route, security-level activation,
classifier model, corpus, signing root or distribution claim is created by
this qualification.
