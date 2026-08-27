# P0 security-level execution sprints

These records execute the [active P0 plan](../../../plans/active/p0-security-levels.md).
PF-15 through PF-22 and PF-13-S01 through S04 remain completed and archived.
PF-13-S05 and PF-27-S01 are `in_progress` in separate qualification/contracts lanes.

The plan permits up to three independent active lanes under the
[sprint concurrency contract](../../index.md#bounded-concurrency).
Drafts need completed dependencies, allocated worktrees/branches/base commits,
and disjoint write scopes before activation. Display priority is not a
dependency: PF-27 can overlap PF-13 qualification; read-only inspector and early
harness work need not wait for all enforcement code.

| Priority | Sprint | Lane | Status | Hard dependencies |
| ---: | --- | --- | --- | --- |
| 13 | [PF-13-S05](pf-13-s05-credential-boundary-adversarial-qualification.md) | qualification | in_progress | PF-13-S04 |
| 14 | [PF-27-S01](pf-27-s01-shared-security-contracts.md) | contracts | in_progress | PF-17-S01, PF-19-S01, PF-20-S01, PF-22-S01 |
| 15 | [PF-26-S01](pf-26-s01-security-harnesses-and-standards-crosswalk.md) | harness | draft | PF-21-S01, PF-27-S01 |
| 16 | [PF-28-S01](pf-28-s01-confidentiality-and-safe-environments.md) | confidentiality | draft | PF-13-S05, PF-27-S01, PF-26-S01 |
| 17 | [PF-29-S01](pf-29-s01-source-envelopes-and-ingress.md) | content | draft | PF-27-S01, PF-26-S01 |
| 18 | [PF-29-S02](pf-29-s02-derived-taint-and-action-context.md) | content | draft | PF-29-S01 |
| 19 | [PF-30-S01](pf-30-s01-isolated-acquisition-runtime.md) | browser | draft | PF-27-S01, PF-26-S01 |
| 20 | [PF-30-S02](pf-30-s02-acquisition-integration-and-recovery.md) | browser | draft | PF-30-S01, PF-29-S01 |
| 21 | [PF-23-S01](pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) | enforcement | draft | PF-13-S05, PF-22-S01, PF-27-S01, PF-28-S01, PF-29-S02, PF-30-S02 |
| 22 | [PF-23-S02](pf-23-s02-aggressive-deny-and-grant-enforcement.md) | enforcement | draft | PF-17-S01, PF-23-S01 |
| 23 | [PF-23-S03](pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | lifecycle | draft | PF-19-S01, PF-20-S01, PF-27-S01, PF-23-S01 |
| 24 | [PF-24-S01](pf-24-s01-security-command-and-profile-view.md) | inspector | draft | PF-20-S01, PF-22-S01, PF-27-S01 |
| 25 | [PF-24-S02](pf-24-s02-security-confirm-cancel-and-downgrade.md) | inspector | draft | PF-23-S02, PF-23-S03, PF-24-S01 |
| 26 | [PF-25-S01](pf-25-s01-temporary-grant-tui.md) | grant-ui | draft | PF-17-S01, PF-23-S02, PF-24-S02 |
| 27 | [PF-25-S02](pf-25-s02-revocation-and-kill-switch-tui.md) | revoke-ui | draft | PF-19-S01, PF-23-S03, PF-24-S02 |
| 28 | [PF-26-S04](pf-26-s04-final-automated-qualification.md) | qualification | draft | PF-26-S01, PF-23-S02, PF-23-S03, PF-25-S01, PF-25-S02, PF-28-S01, PF-29-S02, PF-30-S02 |
| 29 | [PF-26-S02](pf-26-s02-true-tui-and-live-repository-qualification.md) | qualification | draft | PF-26-S04 |
| 30 | [PF-26-S03](pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | qualification | draft | PF-26-S02 |

## Integration checkpoints

1. PF-27 resolves the upstream-touch baseline and lands shared schemas/module,
   manifest, and test-registration seams; PF-26-S01 builds early fixtures.
2. PF-28 confidentiality, PF-29 content, and PF-30 browser backend are separate
   lanes; PF-30-S02 joins isolated acquisition with PF-29-S01 ingress.
3. PF-23-S01 joins the protected boundaries. PF-23-S02 enforcement and S03
   lifecycle can then run independently.
4. PF-24-S02 lands shared UI/event plumbing before separate PF-25 grant and
   revoke view lanes. Each interactive sprint supplies its own actual-key proof.
5. PF-26-S04 freezes the integrated automated candidate; S02 repeats true-TUI
   proof in both live repositories; S03 records human/docs/release acceptance.
   Platform/repository runs can overlap at one candidate, not substitute
   different lane commits for integrated evidence.

Browser parallelism is the approved scheduling intent, not an active allocation:
prioritize PF-30-S01 alongside PF-29 after their shared prerequisites. Add PF-28
when eligible and a slot is free. PF-13-S05 still consumes a slot while active.
PF-30-S02 joins the content contract later; shared facade changes serialize.
Every lane consumes the plan's upstream-touch record and adapter tests. The
remote Linux/tmux report is tracked separately under `qa/reliability/` and is not
evidence that a reconnect fix or security guarantee has been delivered.

## Machine check

```bash
python3 docs/sprints/check.py --json
```
