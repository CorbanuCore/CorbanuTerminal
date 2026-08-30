# PF-20-S02 protected authoritative-state evidence

- Date: 2026-08-30
- Integration base: `207644cb61e1dec04e00bb1e310cd1dbe38e381d`
- Reviewed implementation candidate: `1acf9af9dfd00e5cb7a4221c633af6ea163d3830`
- Contract versions: authoritative security state v1, commit v1, and external anchor v1
- Activation posture: fail closed; no measured PF-27-S03 platform currently qualifies protected persistence

## Contract result

The implementation persists controller-owned security level, grant, revocation,
kill-switch, recovery, revision, and owner-generation state separately from
ordinary preferences. Each committed revision is state/intent/commit bound by
SHA-256 and an append-only chain. A versioned external protected anchor provides
atomic compare-and-store and durable-commit semantics, so complete deletion,
suffix truncation, rollback, forged pending records, stale recovery, and owner
rotation fail closed instead of recreating a Permissive first install.

Recovery is forward-only: it cannot lower the security level, authority
generations, owner generation, or kill-switch restrictions. Rejected
uncommitted suffixes are removable only by the currently authorized owner and
never reach anchored committed history. Symlink replacement, weakened Unix
metadata, digest mismatch, missing records, and corrupt anchors are rejected.

`TrustedControllerAuthorization` is derived from the PF-27 platform report, not
from a caller-supplied role. Its provider is additionally required to derive
the expected target and probe identities independently of model-editable state.
The implementation explicitly returns `UnsupportedPlatform` on non-Unix hosts
until qualified ACL, no-follow, and directory-durability mechanisms exist.

## Changed paths

- `codex-rs/config/src/lib.rs`
- `codex-rs/config/src/security_state.rs`
- `codex-rs/core/src/security/authoritative_state.rs`
- `codex-rs/core/src/security/authoritative_state_tests.rs`
- `codex-rs/core/src/security/mod.rs`
- `docs/sprints/current/p0-security-levels/pf-20-s02-protected-authoritative-state.md`
- `qa/security-levels/sprints/PF-20-S02/evidence.md`

The PF-20-S01 archive and evidence are unchanged. No runtime consumer or TUI
activation was added.

## Automated evidence

All caches, targets, temporary files, logs, and captures were placed under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-authoritative-state/`.

| Check | Result |
| --- | --- |
| `cd codex-rs && just fix -p codex-config && just fix -p codex-core && just fmt` | PASS; final tree formatted |
| `cd codex-rs && just test -p codex-config` | PASS; 229 passed |
| `cd codex-rs && just test -p codex-core config::` | PASS; 487 passed, 2964 skipped |
| `cd codex-rs && just test -p codex-core security::` | PASS; 44 passed, 3407 skipped |
| focused authoritative-state run | PASS; 15 passed |
| `cd codex-rs && just write-config-schema` | PASS; generated schema unchanged |
| `python3 docs/plans/check.py` and `python3 docs/sprints/check.py` | PASS |
| `git diff --check` | PASS |

The final authenticated true-TUI smoke used TMUX session
`pf20-authoritative-smoke-auth` and the Corbanu binary built from the candidate.
`/status` rendered Corbanu v0.1.35, the authenticated Claude provider, and the
PF-20 worktree correctly. Command text and Enter were sent separately.

## Cross-platform posture

PF-27-S03's current macOS, Linux, and Windows probe records all report
`protected_store: unsupported` and `protected_mode_eligible: false` under the
same-user adversary. PF-20 consumes that authorization and therefore cannot
activate on any of those hosts. Unix persistence behavior is covered by the
store tests above. A dedicated non-Unix test asserts the explicit
`UnsupportedPlatform` blocker; no Windows persistence qualification is claimed.

## Independent review

The exact candidate was reviewed read-only in real Corbanu Terminal session
`pf27-opus5-g1-review` using Claude Opus 5 Plan Max. The final verdict was
`NO FINDINGS` after verifying the non-Unix blocker, rejected anchored-pending
recovery, records-ahead deletion bounds, anchor serialization/CAS contract, and
all previously remediated rollback, availability, and TOCTOU cases.

Transcript:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-authoritative-state/tmux-artifacts/opus5-max-final-review.txt`
(SHA-256 `3a6f9379fe8ccb1d9be69d6f308792278788069e36f48bd4f6819f762ba5f4d2`).

## Consumer and integration handoff

PF-27 must supply a genuinely protected external-anchor provider before this
contract can activate. PF-22 owns runtime policy adoption and PF-24/PF-26 own
human transition/recovery proof. The integration owner must merge this lane
after PF-19, rerun the combined config/security suites and governance checkers,
archive PF-20-S02, and retain the fail-closed activation posture.

## Integration result

The integration owner merged the lane at `628c63b3c`. The explicit non-Unix
blocker also passed on the remote Windows target at `fb54216dc`; protected
persistence remains unavailable on every currently measured platform. On the
final three-lane tree, `codex-config` passed 229/229 and the focused
`codex-core security` run passed 53/53. Governance and whitespace checks passed.
