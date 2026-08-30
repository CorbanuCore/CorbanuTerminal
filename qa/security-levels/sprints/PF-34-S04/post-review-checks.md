# PF-34-S04 final lane-local checks

Date: 2026-08-30. Candidate:
`a75efecc0a37d5544e123ad19d57867cac360a68`, rebased onto `main` at
`1a5562738cb3d53bd4d0b6668761cfe76bd4b93e`.

All Cargo, target, temporary, Python-package, and package-download caches were
kept beneath
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`.
The Cargo checks used the isolated unregistered-crate harness; G1 must repeat
them on the registered combined tree.

| Check | Actual result |
| --- | --- |
| isolated `cargo fmt -- --check` | pass; standalone stable Cargo emitted only the known repository nightly-option warning |
| isolated `cargo clippy --all-targets -- -D warnings` | pass |
| isolated focused `cargo test … pf_34_s04 -- --nocapture` | 20 passed; 0 failed; 0 ignored |
| isolated full `cargo test` | 20 passed; 0 failed; 0 ignored |
| repository `just argument-comment-lint --manifest-path <isolated-harness>` | pass with nightly `2025-09-18` |
| `python3 qa/security-levels/ingress-contract/verify.py` | verified seven fixtures; schema 1; contract 1 |
| `python3 qa/security-levels/ingress-contract/test_verify.py` | 14 passed; 0 failed |
| Draft 2020-12 `jsonschema==4.25.1` validation | schema valid; manifest valid |
| `git check-attr text eol -- …/benign-v1/raw.txt` | `text: unset`; `eol: lf` |
| `python3 docs/plans/check.py` | active 1/2; available slots 1 |
| `python3 docs/sprints/check.py` | current 67; archived 88 |
| `shasum -a 256 -c lane-files.sha256` | all 15 lane implementation/fixture files pass |
| `python3 qa/security-levels/sprints/PF-34-S04/verify_evidence.py` | 15 hashes and narrative identities agree |
| `git diff --check` | pass |

The optional JSON Schema validation used the bundled Python 3.12 runtime with
`jsonschema==4.25.1` installed under the lane cache. The system Homebrew Python
3.14 had a host Expat dynamic-link failure; no candidate test relied on that
broken interpreter.

No repository `just fix`, registered crate test, Cargo lock update, or Bazel
test is claimed. Those require the shared files and exact combined-tree reruns
owned by Jim Ricketts at G1.
