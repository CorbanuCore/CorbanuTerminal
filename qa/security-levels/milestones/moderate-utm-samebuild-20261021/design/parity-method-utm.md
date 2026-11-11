# UTM same-build check (utm-par2, owner decision D93): predeclared comparison method

Written and frozen before "macOS 3" is started. It follows the template of the Tart parity run's frozen method
(`moderate-tart-parity-20261020/design/parity-method.md`) and the same-build rule of `vm-parity-method.md` (D97).
Owner's rule (D93): switch milestone runs to Tart if these cases behave on UTM as they did on Tart.

## What runs

- **Cases and groups:** MOD-11 (T09), MOD-21 (T17), MOD-22 (T18), MOD-26 and MOD-29 (T21), the Tart run's groups, each
  once, in that order, one at a time, lanes agmx1, agmx2, agmx3, agmx1.
- **Machine:** UTM guest "macOS 3" only ("macOS 2" is never touched; no Tart VM runs at the same time). The campaign is
  provisioned once; each group gets a wiped lane, a fresh product profile and project and a fresh tester (the UTM qual
  method); `restore.sh` reverses the provisioning at the end and the guest is stopped.
- **Build:** the Tart run's candidate exactly: main `8c73c8ea80`, `bin/corbanu` SHA-256 `15cb32e9…4c4ae6` (the kept
  signed binary `corbanu.prev-8c73c8ea80`) in run 1's helper package; `package-files.txt` identical to the Tart run's.
- **Case texts:** packet `v4u` = the Tart run's packet `v4p` byte for byte (frozen cases, amendments v2-v4, navigation,
  group prompts, screens), except one added line in every group prompt, required by the brief (the qwen-rerun2 lesson):
  look at the screen about every 30 seconds while a product turn runs. It changes no case text, expected result or rule.
- **Harness:** the Tart run's guest files unchanged (`agms-*`, mediator, provision, probes: 99, gate check), the same
  network path (the guest's mediator sends Z.AI requests over one SSH reverse tunnel to the host relay, which adds the
  key; no real key is placed in the guest), the same product configuration and fixture. UTM-only parts: start/stop with
  `utmctl`, root actions through the UTM admin helper, the guest's own seal instead of `seal-guest.sh`, `/etc/codex`
  moved aside and restored, `restore.sh`. The guest's pre-existing transport key file from earlier UTM runs is left in
  place and not used.
- **Models:** product GLM 5.3 Flash on Z.AI (GLM 5.2 for MOD-21, amendments-v3 R-1); testers GLM 5.3 Flash on Z.AI
  (the candidate's `corbanu exec`), one fresh tester per group.

## What it is compared with

The Tart attempts on the same build (`moderate-tart-parity-20261020`): T09 and T09-replay (MOD-11), T17 and
T17-replay-postcap (MOD-21), T18 (MOD-22), T21 and T21-replay-postcap (MOD-26, MOD-29). The MOD-11 tie-break ran
another build (`df3d8c5d1c`) and is not a reference. Tart dispositions (first attempts): MOD-11 failed, MOD-21, MOD-22,
MOD-26 and MOD-29 blocked.

Unit of comparison, as in the Tart method: each expected result (passed / failed / not exercised or inconclusive), then
the case disposition, judged with the same rules (G-1, the owner decisions D39, D40, D41 and D43 only where the raw
outcome repeats the accepted limit; executor verdicts corrected where the evidence contradicts them). Also compared,
because the Tart record ties its differences to them: MOD-11 items 1-2 (is there a product message naming the level, and
did the product's model request escalation), and MOD-21 (does the task reach the child labelled as untrusted data, read
from the child's words and, by the integrator, from the product's session file). Wording or timing differences that no
expected result judges are noted, not counted.

## Classes

- **matched:** the same disposition and outcome for every expected result as the Tart first attempt.
- **within Tart's range:** not matched, but the same as a Tart replay on the same build.
- **differs**, with one attribution, each needing positive evidence: **environment** (a UTM-specific mechanism shown by
  the record: machine, network path, accounts, console state), **tester variance** (the tester deviated from the case
  text), **product-model variance** (the product's own model behaved differently where the case text leaves that to
  the model), or **undetermined**. There is no "product change" class: the build is the same. A change on Z.AI's side
  between 2026-10-20 and this run cannot be told apart from model variance; it is named where it could apply.
- **Transport check:** a variance attribution caused by a model or a slow reply needs the guest mediator log and the
  host relay log of that window to show every upstream request completed (200, no `session_error` or timeout) and no
  tunnel reconnect; otherwise environment or undetermined.

## Replays (bounded)

For a difference attributed to tester or product-model variance, or undetermined: at most one replay of that group on
UTM with a fresh tester, the same prompt, label `utm-replay`; cap 2 replay groups in total, blocker cases first. The
first attempt counts. A variance attribution stands only if the replay is matched or within Tart's range; if the
replay differs the same way, the difference is undetermined (or environment, with positive evidence).

## Decision rule (recommendation on switching milestone runs to Tart)

Recommend switching if:
1. the UTM gate passed (99 probes ok, `AGMS-SMOKE-OK`, relay control 200), every group's preflight and every tester's
   own probe run reported `ok: true`, and the native-prompt guard saw no `SECURITYAGENT-SEEN`;
2. no blocker case (MOD-11, MOD-21, MOD-22) differs as environment or undetermined;
3. advisory cases (MOD-26, MOD-29) that differ as environment or undetermined are listed and explained, and the
   recommendation says whether that matters for milestone use.
If 2 fails, the recommendation is not to switch yet and says what would settle it. New product failures are filed as
issues and reported as findings, not as parity results.

## Environment manifest

UTM side from this run (`vm-start.txt`: macOS version, `csrutil`, `spctl`, guest and host clocks, tool versions,
console state, CPUs and memory; per-group host memory; mediator latency); Tart side from the Tart record. Host memory:
no group starts below 25% free (brief).

## Record

Under `qa/security-levels/milestones/`, as the Tart record: evidence per group, product homes and logs not exported
(listed by size and SHA-256), `pty.raw` and tester events gzipped, each fixture's `.git` renamed `dot-git`, key scan on
everything exported.
