# PF-34-S04 lane evidence

Date: 2026-08-30. Status: **lane candidate at G1; shared registration and
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
- Latest `main` rebase: `1a5562738cb3d53bd4d0b6668761cfe76bd4b93e`
- Allocation: `6a614e8afa825267b329b3274fd3cba648e99151`
- Initial implementation: `9c838e16bbda9726e6c352d57d9223e098dc817f`
- Pre-Opus-review implementation: `475ed05147fb8801672556048014a3ae28bcba86`
- First Opus-remediated implementation: `74e97148701ef541ff9ef2d0a9194ba472b2801c`
- Final Opus-remediated implementation: `a75efecc0a37d5544e123ad19d57867cac360a68`
- Final lane evidence remediation: `c0ede26f2`
- Cache root: `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`
- Product TUI applicability: none; this is a pure, unregistered preparation
  boundary. The mandated independent review itself ran through TMUX and the
  rebased Corbanu Terminal true TUI.

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
- Byte-bearing `Debug` output exposes only byte length and SHA-256; it cannot
  bypass the explicit `into_raw_untrusted` audit token or log hostile payloads.
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
| repository `just argument-comment-lint` against the isolated harness | pass; [log](lane-checks/argument-comment-lint.log) |
| `python3 qa/security-levels/ingress-contract/verify.py` | seven fixtures verified; schema 1; contract 1; [log](lane-checks/fixture-verifier.log) |
| `python3 qa/security-levels/ingress-contract/test_verify.py` | 14 passed, 0 failed; [log](lane-checks/fixture-verifier-tests.log) |
| Draft 2020-12 `jsonschema==4.25.1` validation | schema valid and bundled manifest valid; [log](lane-checks/jsonschema.log) |
| fixture Git attributes | `text: unset`; `eol: lf`; [log](lane-checks/fixture-attributes.log) |
| `python3 docs/plans/check.py` | pass; active 1/2; [log](lane-checks/plan-governance.log) |
| `python3 docs/sprints/check.py` | pass; current 67; archived 88; [log](lane-checks/sprint-governance.log) |
| lane hash ledger | all 15 files verified; [log](lane-checks/lane-hashes.log) |
| narrative/hash consistency guard | 15 hashes and current identities agree; [log](lane-checks/evidence-consistency.log) |
| `git diff --check` | pass; [log](lane-checks/diff-check.log) |

Platform: macOS 26.0 / Darwin 25.0.0, arm64, Apple M2 Ultra, 192 GiB RAM. This
machine is not claimed as the weakest supported CPU.

## Immutable identities

- Contract/schema: `1` / `1`
- Manifest SHA-256: `7e8a4850f67052b2b5b2e0d17f5227116f226c65ee25a6945d04ff7a2a1a1fc3`
- Contract SHA-256: `afde67e16a9117c3bf6052749e450ee805a8266862a87112e84b7918806c12ca`
- Contract tests SHA-256: `895662f8139c0b5e6e9520cb3c9c52ded6812142b14aa2e1774336043728af1f`
- Third full review packet SHA-256: `3813e9783ddbf09fb9e2bdbb16fa9600adeb62b58fcd09385bf6328089bc3389`
- Final evidence-confirmation packet SHA-256: `9753a4b8046359e0c3e6e385fa86770fde692312f3a8c87e9ffd3a979c34ecca`

## Supplemental structured review

Command: repository Autoreview, branch mode from the original pre-rebase
allocation lineage, Codex `gpt-5.5`, high reasoning, web search disabled. Its
raw artifacts preserve pre-rebase commit IDs; the equivalent current allocation
is `6a614e8afa825267b329b3274fd3cba648e99151`. The reviewer found four P2 defects over three
cycles: unfrozen expected verdicts, unfrozen quarantine semantics, traversal and
undeclared-object defects in the JSON Schema, and eager/reassembly allocation
exhaustion. All were verified as in-scope, fixed, and covered by regression
tests or schema validation. The final full-branch rerun reports no findings and
`patch is correct` at confidence 0.84: [text](autoreview.txt),
[structured result](autoreview.json).

## Mandated independent review

The review ran in TMUX through the rebased Corbanu Terminal with provider
`claude-plan`, route `claude-opus-5-plan` (provider-reported Claude Opus 5.0),
and reasoning effort `max`. Four checksum-verified immutable packets produced:

1. 3 P1 / 7 P2 / 16 P3, all lane findings remediated or correctly accepted at G1.
2. 0 P1 / 4 P2 / 12 P3, all four P2 findings remediated.
3. 0 P1 / 1 P2 / 4 P3; the contract, fixtures, verifier, handback, and scope
   exception were accepted, with stale evidence as the sole P2.
4. **clean**, with N-5 resolved and 0 new P0/P1/P2.

The complete runtime attestation, packet hashes, dispositions, and transient
artifact hashes are recorded in
[`claude-opus-5-max-review.md`](claude-opus-5-max-review.md).

## Pending G1/G2 evidence

- Integration-owner Cargo/Bazel/module/lock registration.
- Repository `just fix`, `just fmt`, named/full crate tests and Bazel parity on
  the combined registered tree.
- Integration commit, combined-tree source hashes and acceptance.
- Sprint completion/archive and PF-35-S01 reallocation.

No human or release acceptance is claimed; downstream PF-34/PF-35 consumers and
PF-26 retain those gates.
