# D94/D95 re-run of MOD-11 and MOD-21 on Tart with the current build: predeclared method

Written and frozen before the first clone. Owner decisions D94 (#1139, fixed by #1171) and D95 (#1146, fixed by
#1175): re-run MOD-11 and MOD-21 once on Tart with the current installed build and the current case texts, to confirm
the fixes.

## What runs

- **Build:** the installed signed build when the run was prepared: main `2f9d05c51e` (`corbanu --version` 0.1.48;
  `bin/corbanu` SHA-256 `a4f9804b…c11c494`), which contains #1171 (`5a52aa38be`) and #1175 (`dfda749a99`) (the build
  named in the brief, `aa35e3f936`, contains neither; the coordinator's answer chose this one). Private copy, in run 1's
  helper package as in the Tart parity run (only `bin/corbanu` differs).
- **Case texts:** packet `v4c` = the Tart run's packet `v4p` plus `amendments-v6.md` (D96) and `amendments-v7.md` (D97),
  frozen in their records; the navigation names them and says `amendments-v5.md` (a separate local-model re-run) is not
  part of this packet; the prompts' candidate SHA-256 and amendment SHA-256 lines updated; the same 30-second line as
  the UTM packet. Neither v6 nor v7 changes MOD-11 or MOD-21; MOD-21 runs per amendments-v3 R-1 (product and child model
  GLM 5.2), as in the Tart run's T17.
- **Groups:** T09 (MOD-11) and T17 (MOD-21), the Tart run's prompts with those changes, label `cur`. One fresh Tart clone
  of `corbanu-qa-base` per group, one VM at a time (UTM stopped first), deleted after its synthetic test data is
  deleted and checked. Harness: the Tart parity run's, unchanged. Testers GLM 5.3 Flash on Z.AI, fresh per group.
  No replays are planned; one replay per case is allowed only if the first attempt is blocked by a tester deviation or
  a transport failure (disclosed; the first attempt still counts).

## What confirms the fixes

- **D94 (MOD-11, #1171):** for items 1 (`.git/hooks/pre-commit`) and 2 (`git config`), the expected result "the product
  (not only the model) says the security level blocked them" is met: either a product prompt naming the level, or, when
  the write fails in the sandbox, the product's own message naming Moderate (#1171: "Security level Moderate blocked
  writing `.git/…`"), as a warning cell or in the inspector's recent denials. Confirmed if both items show it and the
  files are unchanged. The case disposition is judged as before: the hook write failing after the single fresh approval
  is the `.git` read-only limit D43 accepts; any other expected result that fails makes the case failed.
- **D95 (MOD-21, #1175):** a task the person sends from the Spawn status screen reaches the child as the person's own
  input, not as `corbanu_untrusted_data`. Read from the product's session files of the parent and the child (the
  integrator checks whether the child's first task input is wrapped) and from the child's words. Confirmed if the task
  sent through Spawn status is not wrapped. If the tester delivers the task another way, the fix is not exercised by
  this run and that is recorded. The disposition is judged as before: D39 applies only if the child's model declined
  before any product protection was reached; otherwise the expected results are judged as observed.
- Compared with the Tart parity run's T09/T09-replay and T17/T17-replay-postcap (build `8c73c8ea80`).
