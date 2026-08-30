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

## Remediated deterministic verification

- Clean implementation commit:
  `7adff552130ec922e9ae85d927001869090ef634`. The generated report records that
  exact source commit, an empty `source_dirty_paths` array and
  `candidate_runtime_tree: clean`.
- Exact command (run from the allocated worktree):

  ```sh
  python3 scripts/security-level-compat \
    --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb \
    --upstream af5a4e39b590e7517120fd935ccfac8cbf7cf131 \
    --candidate /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/candidate-target/debug/corbanu \
    --cache-root /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/cache \
    --temp-root /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/tmp \
    --output /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/full-run-5
  ```

- Result: PASS, 36/36 exact tests: nine inherited cases on each of baseline,
  upstream and candidate (27), four candidate-only protected cases, and five
  immutable S01 probes. Every recorded command executed exactly one test.
- Report:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/full-run-5/compatibility-report.json`,
  SHA-256
  `2a94da7019ddff70f9c9aa2810155205f5f964b650f65352ca80bd288ab3e087`.
- Identities: baseline binary `c8fb4a879a64604c17fe9500864377f86985d180b1627d3af77a3e29e869da88`
  (`corbanu 0.1.35`); upstream binary
  `1b20f5ef0dfa59734bfddd5311690ee1c068515c4c2085e3bfe96bffe3c7fdba`
  (`corbanu 0.1.34`); candidate binary
  `9675ddfa7f3e558a46c7bf75611553939d51055a986ab316e1699e0e6b2c90a0`
  (`corbanu 0.1.35`). Baseline/upstream `codex-rs` tree identities are the
  unequal values recorded above.
- Expanded-control SHA-256:
  `070bb1bd0171d4a5255e58cbacbbf97b95a73fb6020f9ebbfaeace1de688fe19`;
  zero-entry drift-ledger SHA-256:
  `f01e34d04675175a32a70c9571130dd934cb2fe0d4cfe18d9c378cfbfa327fe8`;
  configuration digest
  `fbf6d9f7eee3b7d067fbdecbe52e9ac465330116eb134482f8ef73386cf364fe`;
  environment digest
  `a1ebaf35d76a84b468a23702ebae7087a8f25e4a7389d71988e62086fd984cf4`.
- The harness removed its detached per-run source root (`run_root: null`) and
  retained only the two bounded control targets. The report and all build,
  cache and temporary artifacts live on `CorbanuDrive` outside the worktree.
- `ruff format --check scripts/security_level_compat.py scripts/test_security_level_compat.py`,
  `ruff check scripts/security_level_compat.py scripts/test_security_level_compat.py`,
  and `python3 -m unittest scripts.test_security_level_compat -v` pass; the
  self-test result is 37/37. Governance, immutable-oracle, diff and scope
  checks are rerun after the final review evidence is recorded.

## TMUX evidence and final independent review

- Supporting real-candidate TMUX `/status` and clean-exit smoke artifacts:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/tmux-smoke/status-pane.txt`
  (SHA-256
  `17a09ef7fb46d24dd1f296dae32a10d5985f94868e688148cb2cc4fb656ae9ef`)
  and `tmux-smoke/logs/codex-tui.log` (SHA-256
  `3194fb4857b6083f09896a3ef97e7a67b2c0b4389d1ea28b79fbbfc1eb0a6162`).
- Final read-only Corbanu Terminal + Claude Opus 5 Plan/max review: pending.

The earlier 32-test report is superseded by the clean-source 36/36 report
above and is not final evidence.

The integration owner still must merge after PF-19/PF-20, rerun the combined
tree, update shared plan/navigation, and archive PF-21-S02. This lane does not
perform those shared actions.
