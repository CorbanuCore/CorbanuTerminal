# PF-34-S04 lane evidence

Date: 2026-08-29. Status: **lane candidate at G1; shared registration and
combined-tree acceptance pending**.

Product citations:

- **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected
  paths fail visibly rather than falling back to raw secrets or unscreened
  execution.”
- **Non-negotiable controls** — “Classify instruction intent and provenance
  before external content can influence tools or financial actions.”

## Candidate and scope

- Lane owner: Codex ingress/classifier lane
- Integration owner: Jim Ricketts
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-ingress-classifier`
- Branch: `feat/p0-security-ingress-classifier`
- Base: `6a35712cd5731b191d875e8c6468f1abe23eb66e`
- Allocation: `8534a61d6a5b00f601972bba8afd35879d646657`
- Initial implementation: `bc0864e7dae2082cce276be8199ad4ac07781621`
- Post-review implementation: `002c6382152910d984db22a41a24f80281f6d19b`
- Cache root: `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`
- TUI applicability: none; this is a pure, unregistered preparation boundary.

The lane changed only its allocated contract/test files, versioned fixture
directory, sprint evidence, and sprint record. It did not edit `lib.rs`, Cargo,
Bazel, locks, shared registries, Core/TUI, plan indexes, MkDocs navigation, or
archive state. Exact shared changes are in
[`registration-handback.md`](registration-handback.md).

## Delivered invariants

- Exact contract/source/raw/rendered/sanitized/reassembly and segment bindings.
- Exact classifier artifact and threshold configuration identity.
- One-shot full-content release; progress exposes counters but never bytes.
- Typed allow/suspicious/hostile/unavailable decisions with safe diagnostics.
- Sticky malformed, duplicate, partial, digest/version/source/transformation,
  model/threshold, cancellation, timeout, stale/future verdict and resource
  failures; later forced allow cannot recover a failed session.
- Released bytes remain `untrusted` and carry `authority: none`.
- Seven synthetic fixture files cover benign transformation, a cross-segment
  hostile instruction, and quarantine transitions. The verifier binds the
  complete inventory, hashes, source/transformation/reassembly identities,
  boundaries, fixture-only verdict identity, taint and forbidden transitions.

This does not activate protected ingestion, qualify a detector, grant fixture
authority, clear taint, or make corpus/model/CPU/offline/signing/distribution
claims owned by PF-35-S01/S02.

## Lane-local verification

The exact files were compiled through an isolated Cargo harness because shared
crate registration is G1-owned. The final candidate used `CARGO_HOME`,
`CARGO_TARGET_DIR`, `TMPDIR`, `UV_CACHE_DIR`, and `PIP_CACHE_DIR` beneath the
recorded cache root.

| Check | Result |
| --- | --- |
| isolated `cargo fmt --check` | pass; [log](lane-checks/fmt-check.log) |
| isolated `cargo clippy --all-targets -- -D warnings` | pass; [log](lane-checks/clippy.log) |
| isolated focused `cargo test … pf_34_s04 -- --nocapture` | 20 passed, 0 failed, 0 ignored; [log](lane-checks/focused-tests.log) |
| isolated full harness `cargo test` | 20 passed, 0 failed, 0 ignored; [log](lane-checks/full-harness-tests.log) |
| `python3 qa/security-levels/ingress-contract/verify.py` | seven fixtures verified; schema 1; contract 1; [log](lane-checks/fixture-verifier.log) |
| `python3 qa/security-levels/ingress-contract/test_verify.py` | 3 passed, 0 failed; [log](lane-checks/fixture-verifier-tests.log) |
| Draft 2020-12 `jsonschema==4.25.1` validation | schema valid and bundled manifest valid; [log](lane-checks/jsonschema.log) |
| `python3 docs/plans/check.py` | pass; active 1/2; [log](lane-checks/plan-governance.log) |
| `python3 docs/sprints/check.py` | pass; current 69; archived 86; [log](lane-checks/sprint-governance.log) |
| `git diff --check` | pass; [log](lane-checks/diff-check.log) |

Platform: macOS 26.0 / Darwin 25.0.0, arm64, Apple M2 Ultra, 192 GiB RAM. This
machine is not claimed as the weakest supported CPU.

## Immutable identities

- Contract/schema: `1` / `1`
- Manifest SHA-256: `7e8a4850f67052b2b5b2e0d17f5227116f226c65ee25a6945d04ff7a2a1a1fc3`
- Contract SHA-256: `d9186b41835f5042722909de651d6a79eec228a6fdbfe59e053569ad4765a2bb`
- Contract tests SHA-256: `774f0a52cd2952bd98b77f773bbfa6051f16bbafe2d52b716f2758c96643f43b`
- Review packet SHA-256: `10894e0f3f338fd4090f0a53696a8f3fca4bea59b1615d832717788b3e6eba5c`

## Supplemental structured review

Command: repository Autoreview, branch mode from allocation commit
`8534a61d6a5b00f601972bba8afd35879d646657`, Codex `gpt-5.5`, high
reasoning, web search disabled. The reviewer found four P2 defects over three
cycles: unfrozen expected verdicts, unfrozen quarantine semantics, traversal and
undeclared-object defects in the JSON Schema, and eager/reassembly allocation
exhaustion. All were verified as in-scope, fixed, and covered by regression
tests or schema validation. The final full-branch rerun reports no findings and
`patch is correct` at confidence 0.84: [text](autoreview.txt),
[structured result](autoreview.json).

## Pending G1/G2 evidence

- Required Claude Opus 5.0 Max review and verified finding disposition (Mac currently
  locked when Computer Use first attempted the review).
- Integration-owner Cargo/Bazel/module/lock registration.
- Repository `just fix`, `just fmt`, named/full crate tests and Bazel parity on
  the combined registered tree.
- Integration commit, combined-tree source hashes and acceptance.
- Sprint completion/archive and PF-35-S01 reallocation.

No human or release acceptance is claimed; downstream PF-34/PF-35 consumers and
PF-26 retain those gates.
