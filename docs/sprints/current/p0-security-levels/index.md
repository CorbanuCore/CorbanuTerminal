# P0 security-level execution sprints

These are the current mechanical records for the active
[P0 `/security` levels](../../../plans/active/p0-security-levels.md) plan.
Each sprint links exactly one feature. Work only from one record's
**Remaining** checklist; completed records move to the excluded archive.

Seven feature-worktree commits predate this complete map. PF-15 through PF-21
record code presence honestly but remain unfinished until final-tree evidence is
collected. PF-15-S01 is the only `ready` sprint. All later records are `draft`
and cannot execute until their listed dependencies are completed and archived.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [PF-15-S01](pf-15-s01-security-level-domain-foundation.md) | Reconcile level domain | ready | — |
| 2 | [PF-16-S01](pf-16-s01-authorization-decision-contract.md) | Reconcile authorization contract | draft | PF-15-S01 |
| 3 | [PF-17-S01](pf-17-s01-bounded-delegation-grants.md) | Reconcile bounded grants | draft | PF-16-S01 |
| 4 | [PF-18-S01](pf-18-s01-human-mandates-and-receipts.md) | Reconcile mandates/receipts | draft | PF-16-S01 |
| 5 | [PF-19-S01](pf-19-s01-revocation-contract.md) | Reconcile revocation | draft | PF-17-S01, PF-18-S01 |
| 6 | [PF-20-S01](pf-20-s01-versioned-security-persistence.md) | Reconcile persistence | draft | PF-15-S01 |
| 7 | [PF-21-S01](pf-21-s01-permissive-compatibility-baseline.md) | Reconcile Permissive baseline | draft | PF-16-S01, PF-20-S01 |
| 8 | [PF-22-S01](pf-22-s01-runtime-policy-and-agent-inheritance.md) | Runtime policy/inheritance | draft | PF-19-S01, PF-20-S01, PF-21-S01 |
| 9 | [PF-13-S01](pf-13-s01-vault-backed-exact-host-credential-substitution.md) | Typed credential capability/store | draft | PF-16-S01, PF-17-S01, PF-19-S01, PF-22-S01 |
| 10 | [PF-13-S02](pf-13-s02-scoped-vault-resolver.md) | Zeroizing vault resolver | draft | PF-13-S01 |
| 11 | [PF-13-S03](pf-13-s03-openai-exact-host-proxy-substitution.md) | Exact OpenAI proxy substitution | draft | PF-13-S02 |
| 12 | [PF-13-S04](pf-13-s04-authority-lifecycle-and-raw-secret-bypass.md) | Lifecycle and bypass closure | draft | PF-13-S03 |
| 13 | [PF-13-S05](pf-13-s05-credential-boundary-adversarial-qualification.md) | Credential adversarial proof | draft | PF-13-S04 |
| 14 | [PF-23-S01](pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) | Moderate enforcement | draft | PF-13-S05, PF-22-S01 |
| 15 | [PF-23-S02](pf-23-s02-aggressive-deny-and-grant-enforcement.md) | Aggressive deny/grant | draft | PF-17-S01, PF-23-S01 |
| 16 | [PF-23-S03](pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | Downgrade/restart/inheritance | draft | PF-19-S01, PF-20-S01, PF-23-S02 |
| 17 | [PF-24-S01](pf-24-s01-security-command-and-profile-view.md) | Command and profile view | draft | PF-20-S01, PF-22-S01 |
| 18 | [PF-24-S02](pf-24-s02-security-confirm-cancel-and-downgrade.md) | Confirm/cancel/downgrade | draft | PF-23-S03, PF-24-S01 |
| 19 | [PF-25-S01](pf-25-s01-temporary-grant-tui.md) | Temporary grant TUI | draft | PF-17-S01, PF-23-S02, PF-24-S02 |
| 20 | [PF-25-S02](pf-25-s02-revocation-and-kill-switch-tui.md) | Revoke/kill/recover TUI | draft | PF-19-S01, PF-23-S03, PF-25-S01 |
| 21 | [PF-26-S01](pf-26-s01-security-harnesses-and-standards-crosswalk.md) | Harnesses and crosswalk | draft | PF-13-S05, PF-21-S01, PF-23-S03, PF-25-S02 |
| 22 | [PF-26-S02](pf-26-s02-true-tui-and-live-repository-qualification.md) | True-TUI/live-repo proof | draft | PF-26-S01 |
| 23 | [PF-26-S03](pf-26-s03-human-acceptance-finished-docs-and-release-evidence.md) | Human acceptance/docs/release evidence | draft | PF-26-S02 |

## Machine check

```bash
python3 docs/sprints/check.py --json
```
