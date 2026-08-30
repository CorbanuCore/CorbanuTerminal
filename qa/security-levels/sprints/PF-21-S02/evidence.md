# PF-21-S02 candidate evidence

## Candidate and scope

- Implementation commit: `c469cbb862a1533c77ed4a552d91f97ca65fa6ed`.
- Dispatch base: `77f56da1ecddf6093184280b541339e1869ca7b3`.
- Branch/worktree: `feat/p0-security-compatibility-drift` at
  `/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift`.
- Contract: expanded Permissive compatibility schema version 2; the accepted
  PF-21-S01 oracle remains schema version 1.
- Changed implementation paths:
  - `scripts/security_level_compat.py`
  - `scripts/test_security_level_compat.py`
  - `qa/security-levels/compatibility/README.md`
  - `qa/security-levels/compatibility/upstream-control-v2.json`
  - `qa/security-levels/compatibility/drift-ledger-v2.json`
  - the branch-local PF-21-S02 sprint record
- No Rust/runtime, manifest, lock, active plan, sprint index, MkDocs, archive,
  or other sprint path changed. The immutable
  `qa/security-levels/permissive-baseline-v1.json` SHA-256 remains
  `45d1f2bd96733381638bb62961ee59fb1c026bc05a6a78d03b560cb794406b8d`.

## Independent controls

| Identity | Commit | Executable version | Executable SHA-256 |
| --- | --- | --- | --- |
| Accepted pre-feature baseline | `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb` | `corbanu 0.1.35` | `80473883a379a04f1c37caeaacc985e64ded9818d69f707563daadd82a4b0613` |
| Last qualified pre-baseline upstream-aligned release | `b0dc0624326c706fec5329fd48ed44f243937469` | `corbanu 0.1.35` | `95048d9904180d33b92939bab410763f204ad4152beed837d94a60a48079f0fb` |
| Final implementation candidate | `c469cbb862a1533c77ed4a552d91f97ca65fa6ed` | `corbanu 0.1.35` | `9675ddfa7f3e558a46c7bf75611553939d51055a986ab316e1699e0e6b2c90a0` |

The upstream-aligned control descends from Corbanu convergence commit
`45a60f03d2f6c041d284b41cc3f33c416d9eeed1`, whose upstream Codex parent is
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`. The nine reviewed source blocks
were byte-identical at upstream, baseline, and candidate, so the owner-reviewed
drift ledger has zero accepted entries. Its SHA-256 is
`f01e34d04675175a32a70c9571130dd934cb2fe0d4cfe18d9c378cfbfa327fe8`;
the upstream-control manifest SHA-256 is
`e020dc7ca24ce580da0040195b50ca83fb710a6fcf3629e2d75005a40d46629f`.

## Deterministic qualification

All build, cache, target, report, and temporary paths were beneath
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/`.
The final clean-source command was:

```bash
python3 scripts/security-level-compat \
  --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb \
  --upstream b0dc0624326c706fec5329fd48ed44f243937469 \
  --candidate /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/candidate-target/debug/corbanu \
  --cache-root /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/cache \
  --temp-root /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/tmp \
  --output /Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-compatibility-drift/full-run-4
```

Result: PASS, 32/32 exact tests executed: nine expanded cases on the candidate,
nine on the accepted baseline, nine on the upstream-aligned control, and the
original five immutable PF-21-S01 probes on the candidate. Report:
`full-run-4/compatibility-report.json`, SHA-256
`541d1f6daecc77b17176e6108cbd2c3de5fc46a5ac702b4c61a63f0dd3a2d0a7`.
It records source commit `c469cbb862a1533c77ed4a552d91f97ca65fa6ed`,
`source_dirty_paths: []`, and `candidate_runtime_tree: clean`.

- Configuration digest:
  `8310856bcdb54e986153b1b5837ab5f2811ed1803ebba9281d0c2a1309077aee`.
- Allowlisted environment-facts digest:
  `a1ebaf35d76a84b468a23702ebae7087a8f25e4a7389d71988e62086fd984cf4`.
- `ruff format` and `ruff check` on both affected Python files: PASS.
- `python3 -m unittest scripts.test_security_level_compat -v`: PASS, 22/22.
- `python3 docs/plans/check.py`: PASS, `active 1/2; available slots 1`.
- `python3 docs/sprints/check.py`: PASS, `current 64; archived 91`.
- `git diff --check`: PASS.

Self-tests cover missing inventory surfaces, mismatched identities,
candidate-derived expectations, stale and future-dated ledgers, unobserved
ledger entries, non-exact filters, more than one executed test, every expanded
case failure, dirty runtime masking, and ambient secret exclusion.

## TMUX evidence

Supporting applicability smoke (not PF-26 release qualification) used the real
candidate at 120x40 with isolated synthetic auth, `RUST_LOG=trace`, an isolated
`CODEX_HOME`, and all mutable artifacts on CorbanuDrive. Literal `/status` was
settled visibly before Enter was sent separately; the `Permissions:` checkpoint
rendered, then literal `/exit` plus a separate Enter closed the pane cleanly.

- Session: `pf21-compat-smoke` on private volume-backed tmux socket `pf21`.
- Pane: `tmux-smoke/status-pane.txt`, SHA-256
  `17a09ef7fb46d24dd1f296dae32a10d5985f94868e688148cb2cc4fb656ae9ef`.
- Trace: `tmux-smoke/logs/codex-tui.log`, SHA-256
  `3194fb4857b6083f09896a3ef97e7a67b2c0b4389d1ea28b79fbbfc1eb0a6162`.

Independent review evidence will be appended after the mandatory Corbanu
Terminal + Claude Opus 5 Plan/max read-only verdict completes.

## Integration handoff and limits

The integration owner must merge this scripts-and-evidence-only candidate after
PF-19/PF-20, re-run the compatibility harness and both governance checkers on
the combined tree, verify the immutable oracle hash and no Rust/runtime delta,
then update shared plan/navigation and archive PF-21-S02. This lane does not
claim PF-26-S02 live-repository/actual-key true-TUI or release qualification.
