# PF-21-S02 candidate evidence

## Candidate and scope

- Dispatch base: `77f56da1ecddf6093184280b541339e1869ca7b3`.
- Branch/worktree: `feat/p0-security-compatibility-drift` at
  `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift`.
- Contract: expanded Permissive compatibility schema version 2; the accepted
  PF-21-S01 oracle remains schema version 1 and byte-identical at SHA-256
  `45d1f2bd96733381638bb62961ee59fb1c026bc05a6a78d03b560cb794406b8d`.
- No Rust/runtime, manifest, lock, active plan, sprint index, MkDocs, archive,
  or other sprint path changed.

## Controls and executable inventory

- Accepted pre-feature baseline:
  `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`.
- Genuinely different, buildable upstream-aligned 0.1.34 source control:
  `af5a4e39b590e7517120fd935ccfac8cbf7cf131`. Its `codex-rs` tree
  `9d36bc6b6694d1043b0b75bc13d0e0c0eb5473c9` differs from baseline tree
  `730cdac4e3a14a5ac494b6a80c13b1e1204ac650`; it descends from convergence
  commit `45a60f03d2f6c041d284b41cc3f33c416d9eeed1` and upstream Codex parent
  `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`.
- Nine inherited compatibility cases execute on upstream, baseline, and
  candidate. Four additional candidate-only protected-boundary cases are pinned
  to the pre-candidate dispatch base and directly execute
  `codex-secret-broker`, `codex-network-proxy`, `codex-browser-isolation`, and
  `codex-content-security`. The five immutable S01 probes remain unchanged.
- The four protected cases verify platform eligibility rejection, explicit
  credential-injection opt-in, Permissive browser-inactivity, and current-verdict
  screening. They do not substitute for PF-26 live-repository qualification.

## First independent review and remediation

The first read-only review used real TMUX session `pf27-opus5-g1-review` with
Corbanu Terminal and `claude-opus-5-plan` at `max`. It found coverage and harness
hardening issues; it was not a clean verdict. The complete recovered response is
stored outside the repository at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/tmux-review/pf21-opus-review-final-full.txt`.

Remediation adds direct security-crate cases, replaces the Rust-identical
upstream release with the convergence source, exercises every drift-ledger
branch, caps review freshness in code, distinguishes command/contract crashes,
hashes all tracked execution recipe inputs, rejects in-worktree artifacts,
prunes stale control targets, preserves primary errors through cleanup, removes
per-run detached sources, and uses a test-anchored lexical Rust extractor.

## Pending final evidence

The earlier 32-test report is superseded by this remediation and is not final
evidence. Before handoff, record here:

- the clean remediation implementation commit;
- a clean-source 36/36 report, exact command, identities and artifact hashes;
- formatted/linted Python and the 37/37 self-test result;
- governance, immutable-oracle, diff and scope results;
- the existing TMUX smoke artifacts;
- a fresh read-only Corbanu Terminal + Claude Opus 5 Plan/max `NO FINDINGS`
  transcript and hash.

The integration owner still must merge after PF-19/PF-20, rerun the combined
tree, update shared plan/navigation, and archive PF-21-S02. This lane does not
perform those shared actions.
