# P0 security-level execution sprints

The [P0 security plan](../../../plans/active/p0-security-levels.md) owns these
20 records: the rest of the 20-sprint core from Travis's 2026-10-06 decisions (19 named
sprints plus the flagged picker), the broker-lane additions PF-27-S05 and PF-27-S06
(coordinator, 2026-10-06), and two hosted non-security records. Up to three
core sprints may be reserved at once, one per lane, with disjoint `write_scope`.
PF-83-S01 closed on 2026-10-06 and is [archived](../../archive/p0-security-levels/pf-83-s01-permission-confirmation.md);
PF-30-S01 completed (PR #178) and is [archived](../../archive/p0-security-levels/pf-30-s01-typed-source-envelope.md);
PF-24-S03 completed (PR #186) and is [archived](../../archive/p0-security-levels/pf-24-s03-flagged-security-picker.md);
PF-30-S02 completed (PRs #190, #198) and is [archived](../../archive/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md);
PF-30-S03 completed (PRs #204, #212) and is [archived](../../archive/p0-security-levels/pf-30-s03-post-taint-authority-checks.md).
PF-27-S04 completed on 2026-10-06 and is [archived](../../archive/p0-security-levels/pf-27-s04-isolated-credential-broker.md);
its open items moved to PF-27-S02, the new PF-27-S05 and PF-27-S06, and PF-23-S03.
Deferred sprints moved to the [P1 hardening sprints](../p1-security-hardening/index.md).
Earlier allocation notes are in the [plan history](../../../plans/history/p0-security-levels-2026-10-06.md).

| Order | Sprint | Outcome | Lane | Depends on |
| ---: | --- | --- | --- | --- |
| 29 | [PF-27-S02](pf-27-s02-secretless-agent-launch.md) | Secretless agent launch and bypass containment (in progress, macOS+Linux) | broker | PF-27-S04 (archived) |
| 30 | [PF-28-S01](pf-28-s01-central-secret-output-gate.md) | Central secret and protected-output gate (merged behind `secret_output_gate`) | broker | PF-27-S02 |
| 31 | [PF-28-S02](pf-28-s02-reflected-secret-response-scrubbing.md) | Reflected-secret response scrubbing | broker | PF-28-S01 |
| 32 | [PF-33-S01](pf-33-s01-url-dns-and-redirect-policy.md) | URL DNS and redirect policy (merged behind `url_destination_policy`) | first free | PF-27-S02, PF-33-S03 |
| 33 | [PF-33-S02](pf-33-s02-connection-pinning-and-bypass.md) | Connection pinning and alternate-egress denial (merged behind `url_destination_policy`, #215; open items recorded) | first free | PF-33-S01 (merged behind flag) |
| 35 | [PF-29-S01](pf-29-s01-protected-mode-inventory.md) | Protected-mode inventory and activation preflight | first free | PF-28-S02, PF-20-S02 |
| 36 | [PF-29-S02](pf-29-s02-human-secret-migration.md) | Human-reviewed credential migration and recovery | first free | PF-29-S01, PF-24-S01 |
| 40 | [PF-23-S01](pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) | Moderate ingress and disclosure enforcement (next in the lane; takes PF-30-S03's moved items) | untrusted content | PF-13-S05, PF-22-S02, PF-30-S03 (archived) |
| 41 | [PF-23-S02](pf-23-s02-aggressive-deny-and-grant-enforcement.md) | Aggressive deny and grant enforcement | untrusted content | PF-17-S01, PF-23-S01 |
| 42 | [PF-23-S03](pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) | Downgrade, restart, and inheritance enforcement | untrusted content | PF-19-S02, PF-20-S02, PF-23-S02 |
| 43 | [PF-24-S02](pf-24-s02-security-confirm-cancel-and-downgrade.md) | Security confirm, cancel, and downgrade | tui | PF-24-S03, PF-23-S03, PF-24-S01, PF-29-S02 |
| 44 | [PF-25-S01](pf-25-s01-temporary-grant-tui.md) | Temporary grant TUI | tui | PF-17-S01, PF-23-S02, PF-24-S02 |
| 45 | [PF-25-S02](pf-25-s02-revocation-and-kill-switch-tui.md) | Revocation and kill-switch TUI | tui | PF-19-S02, PF-23-S03, PF-25-S01 |
| 46 | [PF-27-S05](pf-27-s05-model-client-auth-broker.md) | Core model-client auth and vault labels through the broker | broker | PF-27-S02 |
| 47 | [PF-27-S06](pf-27-s06-windows-broker-and-launch.md) | Windows broker and secretless launch | broker | PF-27-S02 |
| 71 | [PF-41-S01](pf-41-s01-effective-security-inspector.md) | Effective security inspector and degradation state | convergence | PF-23-S03, PF-29-S02, PF-24-S02 |
| 73 | [PF-13-S07](pf-13-s07-integrated-credential-boundary-qualification.md) | Integrated credential boundary qualification | convergence | PF-13-S05, PF-13-S06, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02 |
| 79 | [PF-77-S01](pf-77-s01-tasknode-reliability.md) | Task Node durable command and transport recovery | hosted, not security | none |
| 83 | [PF-76-S01](pf-76-s01-provider-profile-persistence.md) | Provider profile persistence | hosted, not security | none |

Order is topological, not a schedule. Each record's front matter gives its status
(`in_progress` for the active lane sprints, `draft` otherwise).

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
