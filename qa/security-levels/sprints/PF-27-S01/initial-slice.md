# PF-27-S01 — initial-slice historical evidence

Historical record only. See [current evidence](evidence.md) for sprint status.

Date: 2026-08-27. Status: **in progress**, not sprint completion or release acceptance.
Product initiative under **P0 `/security` levels**: “Permissive preserves the
shipping behavior and does not silently change existing policies.”
Plan: `docs/plans/active/p0-security-levels.md`; accountable owner: Jim Ricketts.

## Allocation and baseline

- Implementation worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01`.
- Branch: `codex/pf-27-shared-security-contracts`.
- Fork base: `ea7d4bec720098f6e0994fcfcc59e272108f7e70`.
- Candidate: uncommitted first slice on that base; code fingerprints below.
- Upstream: `https://github.com/openai/codex.git` at
  `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.
- Incorporating merge: `45a60f03d2f6c041d284b41cc3f33c416d9eeed1`, parents
  `4b57758644f761d3205039f6e8f870b6de5ece94` and the upstream SHA above.
- PF-13 worktree remained clean at its existing candidate. Its macOS/Windows
  qualification and independent-review requirements are unchanged.

Read-only ancestry verification after fetching additional history:

```text
git fetch --no-tags --deepen=200 origin feat/pf-13-s02-scoped-vault-resolver
git fetch --no-tags https://github.com/openai/codex.git 413492cd6c3a4d4f8dff6f406247ccda5a9d88aa
git merge-base --is-ancestor 413492cd6c3a4d4f8dff6f406247ccda5a9d88aa ea7d4bec720098f6e0994fcfcc59e272108f7e70
git show -s --format='%H %P %s' 45a60f03d2f6c041d284b41cc3f33c416d9eeed1
```

All succeeded. This verifies an inherited baseline, not a new upstream upgrade.
Retain the product-owned policy crate; no native runtime adapter is changed in
this slice. Provider wire, permissions, history, persistence, cancellation, and
resume are untouched; their later adapter conformance is still pending.

| Completed dependency | Evidence-linked implementation |
| --- | --- |
| PF-17-S01 | `5a03e1e0ec125e87cb1a71983ce4696e22849eb0` |
| PF-19-S01 | `46e9121ebe319e891787bfeee68df7635dcc119e` |
| PF-20-S01 | `655a66c17322d046801825f1b230331bc09dc0b4` |
| PF-22-S01 | `18c4a0a3fae23cf74bb01acb3e58f63c7c0c60c6`; prior tested tree `2a0f3abfd0974841cc881d82b09796a6a9bf436e` |

## First reviewable slice

Added `security-policy/src/integration.rs`, its sibling tests, and explicit
exports/test registration in `security-policy/src/lib.rs`. No dependencies,
installers, manifests, lockfiles, native adapters, or runtime activation changed.

- Versioned inspector snapshots validate both construction and deserialization.
- Committed requested policy and effective floor are distinct; narrowing is
  allowed, weakening is rejected. Pending UI selections are not requested facts.
- Browser, content-firewall, confidentiality, and protected-action health are
  independent; defaults are unavailable even at a stronger effective floor.
- Fixed degraded-reason enums carry no free-form backend errors. Successful
  snapshots contain no arbitrary strings, credential labels, content, or grants.
- Unknown versions, levels, status/reason values, missing facts, and extra fields
  fail parsing. Raw deserialization errors are not safe audit messages and must
  still pass the PF-28 safe-output boundary when adapters are implemented.
- These are observations, not authority. A valid payload does not authenticate
  its sender. Trusted host producers and action-epoch binding are later work;
  never accept model/tool snapshots or use snapshots to authorize actions.

Existing-type inventory: reuse `SecurityLevel` directly here. Subsequent slices
must reuse `AuthorizationRequest`/`ActorChain`, `BoundedGrant`, and
`RevocationState::generation`; `SecuritySettings` and Core's
`PersistedHumanSecurityState` remain persistence owners. Core's existing
`EffectivePolicyView`/`TrustedSecurityController` remain read/write capabilities.
No replacement state machine or scheduler has been added.

## Verification

Environment: macOS 15.6.1 (24G90), arm64; from `codex-rs`, Rust/Cargo 1.95.0;
cargo-nextest 0.9.143, just 1.58.0. Separate local build directory: 181 MiB after
the run; approximately 55 GiB free. No PF-13 build products were reused or changed.

| Command | Result |
| --- | --- |
| `python3 docs/sprints/check.py` at ready, then in_progress | PASS, 25 current / 84 archived; 2 of 3 active slots, disjoint scopes |
| `cd codex-rs && just fix -p codex-security-policy` | PASS |
| `cd codex-rs && just fmt` | PASS, diff inspected before final tests |
| `cd codex-rs && just fmt-check` | PASS |
| `cd codex-rs && just test -p codex-security-policy` | PASS, 29 passed / 0 failed / 0 skipped; 8 new integration tests plus 21 existing policy/credential tests |
| `python3 -m unittest discover -s docs/sprints/tests -p 'test_*.py'` | PASS, 19 tests |
| `git diff --check` | PASS |
| Structured Autoreview | PASS, Codex `gpt-5.5`, no accepted/actionable findings; not sprint-wide independent security acceptance |

Final test run ID: `947d099c-06d8-4a39-933d-a1feebb50af3` (nextest profile `local`).
Earlier run `d8390c89-2222-4329-a42b-fe56adcf0261` had 28 passes / 1 failure:
Serde internally tagged unit variants accepted extra fields. Empty struct
variants fixed the issue; the all-variant payload-extension regression now passes.
Existing frozen-surface Permissive composition, grant narrowing, revocation,
mandate integrity, and credential capability regressions all passed.

SHA-256 fingerprints (paths below `codex-rs/security-policy/src/`):

```text
76c93ffead62e5fa0e244e6de5037ae6d6863a109d287427d1ce48c450635a58  integration.rs
d974567defcd69f6b2299a8413571658471e4530733a0c8ad4246c955c5af8b8  integration_tests.rs
c12392659d8cef8f211aedc318171289d3a72bc12dad25c725b09ee03be5b181  lib.rs
```

## Remaining and non-claims

Provenance/taint, immutable action context, authority epochs across resume,
trusted human requests, protocol/Core/TUI seams, consumer conformance fixtures,
and integrated acceptance remain unchecked in the current sprint. Dependent
sprints, including browser PF-30, remain draft until their prerequisites finish.

No interactive behavior changed: actual-key TUI and live TensorCash/Isometric
workflows are not applicable to this internal slice. No Core/workspace, Linux,
Windows, browser containment, or new upstream qualification is claimed. Human
acceptance, independent security review, final integrated tests, finished docs,
and any due benchmarks remain release gates. No release candidate is published.

Review scope baseline: user asked to start PF-27; only the inspector contract
slice and activation/evidence metadata are changed. Owner boundary is
`codex-security-policy`, with 8 source/record files before review artifacts.
Non-test Rust change: 163 lines including registration. Remaining PF-27 work is
intentional staging under the Rust change-size policy, not a completion claim.

Review command: `python3 /Users/travisgood/.codex/skills/autoreview/scripts/autoreview --mode local`
with the explicit staged-scope prompt and `--output` / `--json-output` pointing
to `autoreview.txt` / `autoreview.json` in this directory. Python was used because
the installed helper lacks executable permission; no permission change or
review-engine substitution was needed. The helper exited 0; no findings were
accepted or rejected and no review-triggered code changes were required.
Artifacts: [review text](autoreview.txt), [structured review](autoreview.json).
After review only evidence/activation wording was reconciled; code fingerprints
remain those of the tested and reviewed tree. Work remains local and uncommitted.
