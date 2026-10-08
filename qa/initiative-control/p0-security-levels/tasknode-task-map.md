# Task Node task map: P0 security levels

Links each Task Node task to the sprints, merged PRs, gate records and evidence record behind it. Use it to check
any evidence sent against these tasks. Plan: [p0-security-levels.md](../../../docs/plans/active/p0-security-levels.md),
sprints: [index](../../../docs/sprints/current/p0-security-levels/index.md).

- Rules (Travis): Task Node moves tasks through stages; Corbanu only submits evidence of completed work. Nothing is
  submitted against closed or rewarded tasks. Work that no open task fits gets new, right-sized tasks, mapped here.
- Account: GitHub `IridiumMaster`, account `acct_oauth_73e1ab2f9f7cd01d3ee6230e`, `~/.corbanu` default profile.
- Inventory before this round (2026-10-07 23:5xZ): 1 Proposed (`task_2bd6a15…`, a PostfiatL1V2 node bug; not this
  work), 2 Refused, 11 Rewarded, 0 Accepted, 0 in verification. No open task matched the security work.
  `task_865f75c…` ("Corbanu workstream 1: PF-13 security and protected credentials") was Rewarded on 2026-09-15 and
  gets no further submissions.
- Created: 2026-10-07, one `tasknode request create --body-file` per track, source title "P0 security levels task
  map", each tagged `Tracking ref: P0SEC-TN-NN` and sent once (a server-side check for the tag ran before each send).
  Task Node generated the eight tasks itself, all **Proposed** at 2026-10-08 00:16Z. PR #261 (PF-25-S02) merged at
  00:23Z, so P0SEC-TN-09 was requested the same way at 01:10Z. Travis accepts or refuses the tasks;
  Corbanu did not accept, refuse or move any of them. Evidence is submitted only after a task reads Accepted.
- Every task asks for a committed evidence record; those are under
  [docs/research/tasknode-integration/](../../../docs/research/tasknode-integration/) (`p0sec-tn-NN-*.md`). Task 01 named its
  own file path, which is why its date is 20261006.
- States below were read on 2026-10-08 at 01:15Z.

## Tasks

| Ref | Task | Reward shown | State | Sprints | PRs (merge commit) | Evidence record |
| --- | --- | --- | --- | --- | --- | --- |
| P0SEC-TN-01 | `task_169f5d2052343f51777060e4ebb34e69`<br>Compile the P0SEC-TN-01 Broker-Lane Sprint Evidence Record | 2 PFT | Proposed | PF-27-S04, PF-27-S02, PF-27-S05 | [#180](https://github.com/CorbanuCore/CorbanuTerminal/pull/180) `13cf4a2d0c`, [#191](https://github.com/CorbanuCore/CorbanuTerminal/pull/191) `699bd4a82f`, [#229](https://github.com/CorbanuCore/CorbanuTerminal/pull/229) `c5bdadd322`, [#237](https://github.com/CorbanuCore/CorbanuTerminal/pull/237) `5656e997d2` | [p0sec-tn-01-broker-lane-evidence-20261006.md](../../../docs/research/tasknode-integration/p0sec-tn-01-broker-lane-evidence-20261006.md) |
| P0SEC-TN-02 | `task_1e60ecf27d3611270ba86aa46343226b`<br>Compile the P0SEC-TN-02 Secret Output Gate Sprint Evidence Record | 2.5 PFT | Proposed | PF-28-S01, PF-28-S02 | [#208](https://github.com/CorbanuCore/CorbanuTerminal/pull/208) `24a57e38c4`, [#216](https://github.com/CorbanuCore/CorbanuTerminal/pull/216) `b9f215ec50`, [#225](https://github.com/CorbanuCore/CorbanuTerminal/pull/225) `8cf46179f5` | [p0sec-tn-02-secret-output-gate-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-02-secret-output-gate-evidence-20261007.md) |
| P0SEC-TN-03 | `task_94f31f7ebd6ba20445da0291ca77f945`<br>Compile the P0SEC-TN-03 Egress Destination Policy Evidence Record | 2.5 PFT | Proposed | PF-33-S01, PF-33-S02 | [#210](https://github.com/CorbanuCore/CorbanuTerminal/pull/210) `8dd531714a`, [#215](https://github.com/CorbanuCore/CorbanuTerminal/pull/215) `a0d96aea4b`, [#224](https://github.com/CorbanuCore/CorbanuTerminal/pull/224) `6d55f6ae8f` | [p0sec-tn-03-egress-destination-policy-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-03-egress-destination-policy-evidence-20261007.md) |
| P0SEC-TN-04 | `task_8ce36190b7954b6d5d4f1bcd691886cc`<br>Compile the P0SEC-TN-04 Protected-Mode Sprint Evidence Record | 3 PFT | Proposed | PF-29-S01, PF-29-S02 | [#228](https://github.com/CorbanuCore/CorbanuTerminal/pull/228) `b751254169`, [#235](https://github.com/CorbanuCore/CorbanuTerminal/pull/235) `65d42158d7` | [p0sec-tn-04-protected-mode-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-04-protected-mode-evidence-20261007.md) |
| P0SEC-TN-05 | `task_b97153cdd989d5a800b05e1068c6dfe2`<br>Compile the P0SEC-TN-05 Untrusted-Content Lane Evidence Record | 3.5 PFT | Proposed | PF-30-S01, PF-30-S02, PF-30-S03 | [#178](https://github.com/CorbanuCore/CorbanuTerminal/pull/178) `b96b23344b`, [#190](https://github.com/CorbanuCore/CorbanuTerminal/pull/190) `7b2a04ea41`, [#198](https://github.com/CorbanuCore/CorbanuTerminal/pull/198) `743a7c22da`, [#204](https://github.com/CorbanuCore/CorbanuTerminal/pull/204) `38516a5b22`, [#212](https://github.com/CorbanuCore/CorbanuTerminal/pull/212) `55339d5b24` | [p0sec-tn-05-untrusted-content-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-05-untrusted-content-evidence-20261007.md) |
| P0SEC-TN-06 | `task_22ba027b6f6a85dcd4cb47bf29888891`<br>Compile the P0SEC-TN-06 Security Levels Enforcement Evidence Record | 3.5 PFT | Proposed | PF-23-S01, PF-23-S02, PF-23-S03 | [#223](https://github.com/CorbanuCore/CorbanuTerminal/pull/223) `f79887d182`, [#233](https://github.com/CorbanuCore/CorbanuTerminal/pull/233) `64137b7189`, [#243](https://github.com/CorbanuCore/CorbanuTerminal/pull/243) `6b1c8b8873`, [#250](https://github.com/CorbanuCore/CorbanuTerminal/pull/250) `a230f20821`, [#246](https://github.com/CorbanuCore/CorbanuTerminal/pull/246) `95b5f34a55`, [#247](https://github.com/CorbanuCore/CorbanuTerminal/pull/247) `242f4d3bde`, [#248](https://github.com/CorbanuCore/CorbanuTerminal/pull/248) `7fc064e593`, [#249](https://github.com/CorbanuCore/CorbanuTerminal/pull/249) `4f09d7af99`, [#256](https://github.com/CorbanuCore/CorbanuTerminal/pull/256) `213698e607` | [p0sec-tn-06-enforcement-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-06-enforcement-evidence-20261007.md) |
| P0SEC-TN-07 | `task_627354898804e21841f81a559fe63462`<br>Compile the P0SEC-TN-07 Security TUI Lane Evidence Record | 3.5 PFT | Proposed | PF-24-S03, PF-24-S02, PF-25-S01 | [#186](https://github.com/CorbanuCore/CorbanuTerminal/pull/186) `023355670a`, [#253](https://github.com/CorbanuCore/CorbanuTerminal/pull/253) `e4d17dbdc6`, [#258](https://github.com/CorbanuCore/CorbanuTerminal/pull/258) `b2d2583e0f`, [#260](https://github.com/CorbanuCore/CorbanuTerminal/pull/260) `bb609b449a` | [p0sec-tn-07-security-tui-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-07-security-tui-evidence-20261007.md) |
| P0SEC-TN-08 | `task_4c84685243b6fd68facf8c2667fbaa15`<br>Compile the P0SEC-TN-08 Security Levels Convergence Evidence Record | 4 PFT | Proposed | PF-41-S01, PF-13-S07 | [#259](https://github.com/CorbanuCore/CorbanuTerminal/pull/259) `8b3ac213c4`, [#242](https://github.com/CorbanuCore/CorbanuTerminal/pull/242) `8a8afb2384`, [#244](https://github.com/CorbanuCore/CorbanuTerminal/pull/244) `824ce30b05`, [#255](https://github.com/CorbanuCore/CorbanuTerminal/pull/255) `2bc2c0bd7f` | [p0sec-tn-08-convergence-evidence-20261007.md](../../../docs/research/tasknode-integration/p0sec-tn-08-convergence-evidence-20261007.md) |
| P0SEC-TN-09 | `task_8e353c0cb1ee43239c5a7691668087ab`<br>Compile the P0SEC-TN-09 PF-25-S02 Sprint Evidence Record | 3 PFT | Proposed | PF-25-S02 | [#261](https://github.com/CorbanuCore/CorbanuTerminal/pull/261) `43b21fc899` | [p0sec-tn-09-revocation-kill-switch-evidence-20261008.md](../../../docs/research/tasknode-integration/p0sec-tn-09-revocation-kill-switch-evidence-20261008.md) |

Request IDs:

- P0SEC-TN-01: `req_3ec829e5ace25114ce8152ea021da7122c1a598ea0b7750ac0395258fa8045a2`
- P0SEC-TN-02: `req_a707b82b15b32003ee6ecf54448460e1354261dd7d85a31a8225b2b7c0a87ad3`
- P0SEC-TN-03: `req_6ab71c71b9e9eaf14e8909d191cdb28ce2a0a6bf7bd2bdad2ea9e09fdc3ef4d6`
- P0SEC-TN-04: `req_8d71397c5c9650ec18a81ecd6c45a4be8ef6d230a1ff1065fd848e2529757f86`
- P0SEC-TN-05: `req_97a3a32b993160d35d6c41842b2e9ca5612e705ef9298e2090efcd7187f839ad`
- P0SEC-TN-06: `req_e2242c029b999635535c05d33b5ec720f7aec22d05fb526be868f8fecf0db588`
- P0SEC-TN-07: `req_36bf02981ba06ce3ab13933094e79f0c326ee0aa673ec759713634c5e716e776`
- P0SEC-TN-08: `req_eff6fbe134688692e4b24b2eb5d94cf1872c7421411406e337ada6eb747ade1a`
- P0SEC-TN-09: `req_05dcfc83d07c336d3feb70b9642dd4b6a06b4661dc5dd29820b49e50558d8abb`

## Sprint index

| Sprint | Task | Sprint record | Gate record | Videos |
| --- | --- | --- | --- | --- |
| PF-23-S01 | P0SEC-TN-06 | [pf-23-s01-moderate-ingress-and-disclosure-enforcement.md](../../../docs/sprints/archive/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md) (completed (archived)) | [slice-1-gate.md](../../security-levels/sprints/PF-23-S01/slice-1-gate.md), [slices-2-3-gate.md](../../security-levels/sprints/PF-23-S01/slices-2-3-gate.md) | [9](../../demos/index/PF-23-S01.md) |
| PF-23-S02 | P0SEC-TN-06 | [pf-23-s02-aggressive-deny-and-grant-enforcement.md](../../../docs/sprints/archive/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md) (completed (archived)) | [gate.md](../../security-levels/sprints/PF-23-S02/gate.md) | [4](../../demos/index/PF-23-S02.md) |
| PF-23-S03 | P0SEC-TN-06 | [pf-23-s03-downgrade-restart-and-inheritance-enforcement.md](../../../docs/sprints/archive/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md) (completed (archived)) | [gate.md](../../security-levels/sprints/PF-23-S03/gate.md) | [4](../../demos/index/PF-23-S03.md) |
| PF-24-S02 | P0SEC-TN-07 | [pf-24-s02-security-confirm-cancel-and-downgrade.md](../../../docs/sprints/archive/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-24-S02/README.md) | [8](../../demos/index/PF-24-S02.md) |
| PF-24-S03 | P0SEC-TN-07 | [pf-24-s03-flagged-security-picker.md](../../../docs/sprints/archive/p0-security-levels/pf-24-s03-flagged-security-picker.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-24-S03/README.md) | [18](../../security-levels/sprints/PF-24-S03/demos/index.md) |
| PF-25-S01 | P0SEC-TN-07 | [pf-25-s01-temporary-grant-tui.md](../../../docs/sprints/archive/p0-security-levels/pf-25-s01-temporary-grant-tui.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-25-S01/README.md) | [3](../../demos/index/PF-25-S01.md) |
| PF-25-S02 | P0SEC-TN-09 | [pf-25-s02-revocation-and-kill-switch-tui.md](../../../docs/sprints/archive/p0-security-levels/pf-25-s02-revocation-and-kill-switch-tui.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-25-S02/README.md) | [3](../../demos/index/PF-25-S02.md) |
| PF-27-S02 | P0SEC-TN-01 | [pf-27-s02-secretless-agent-launch.md](../../../docs/sprints/archive/p0-security-levels/pf-27-s02-secretless-agent-launch.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-27-S02/README.md) | [5](../../demos/index/PF-27-S02.md) |
| PF-27-S04 | P0SEC-TN-01 | [pf-27-s04-isolated-credential-broker.md](../../../docs/sprints/archive/p0-security-levels/pf-27-s04-isolated-credential-broker.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-27-S04/isolated-broker-20261006/README.md) | [3](../../demos/index/PF-27-S04.md) |
| PF-27-S05 | P0SEC-TN-01 | [pf-27-s05-model-client-auth-broker.md](../../../docs/sprints/archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-27-S05/README.md) | [4](../../demos/index/PF-27-S05.md) |
| PF-28-S01 | P0SEC-TN-02 | [pf-28-s01-central-secret-output-gate.md](../../../docs/sprints/archive/p0-security-levels/pf-28-s01-central-secret-output-gate.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-28-S01/README.md) | [4](../../demos/index/PF-28-S01.md) |
| PF-28-S02 | P0SEC-TN-02 | [pf-28-s02-reflected-secret-response-scrubbing.md](../../../docs/sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md) (merged behind flag; record current (milestone items open)) | [README.md](../../security-levels/sprints/PF-28-S02/README.md) | [5](../../demos/index/PF-28-S02.md) |
| PF-29-S01 | P0SEC-TN-04 | [pf-29-s01-protected-mode-inventory.md](../../../docs/sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md) (merged behind flag; record current (milestone items open)) | [README.md](../../security-levels/sprints/PF-29-S01/README.md) | [5](../../demos/index/PF-29-S01.md) |
| PF-29-S02 | P0SEC-TN-04 | [pf-29-s02-human-secret-migration.md](../../../docs/sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md) (merged behind flag; record current (milestone items open)) | [README.md](../../security-levels/sprints/PF-29-S02/README.md) | [3](../../demos/index/PF-29-S02.md) |
| PF-30-S01 | P0SEC-TN-05 | [pf-30-s01-typed-source-envelope.md](../../../docs/sprints/archive/p0-security-levels/pf-30-s01-typed-source-envelope.md) (completed (archived)) | [labelled-ingress-gate.md](../../security-levels/sprints/PF-30-S01-typed-source-envelope/labelled-ingress-gate.md) | [3](../../demos/index/PF-30-S01.md) |
| PF-30-S02 | P0SEC-TN-05 | [pf-30-s02-persistent-taint-and-memory.md](../../../docs/sprints/archive/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md) (completed (archived)) | [persistent-origins-gate.md](../../security-levels/sprints/PF-30-S02/persistent-origins-gate.md), [slice-2-gate.md](../../security-levels/sprints/PF-30-S02/slice-2-gate.md) | [7](../../demos/index/PF-30-S02.md) |
| PF-30-S03 | P0SEC-TN-05 | [pf-30-s03-post-taint-authority-checks.md](../../../docs/sprints/archive/p0-security-levels/pf-30-s03-post-taint-authority-checks.md) (completed (archived)) | [post-taint-gate.md](../../security-levels/sprints/PF-30-S03/post-taint-gate.md), [slice-2-gate.md](../../security-levels/sprints/PF-30-S03/slice-2-gate.md) | [7](../../demos/index/PF-30-S03.md) |
| PF-33-S01 | P0SEC-TN-03 | [pf-33-s01-url-dns-and-redirect-policy.md](../../../docs/sprints/archive/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md) (completed (archived)) | [README.md](../../security-levels/sprints/PF-33-S01/README.md) | [7](../../demos/index/PF-33-S01.md) |
| PF-33-S02 | P0SEC-TN-03 | [pf-33-s02-connection-pinning-and-bypass.md](../../../docs/sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) (merged behind flag; record current (milestone items open)) | [README.md](../../security-levels/sprints/PF-33-S02/README.md) | [5](../../demos/index/PF-33-S02.md) |
| PF-41-S01 | P0SEC-TN-08 | [pf-41-s01-effective-security-inspector.md](../../../docs/sprints/current/p0-security-levels/pf-41-s01-effective-security-inspector.md) (merged behind flag; record current (milestone items open)) | [README.md](../../security-levels/sprints/PF-41-S01/README.md) | [3](../../demos/index/PF-41-S01.md) |
| PF-13-S07 | P0SEC-TN-08 | [pf-13-s07-integrated-credential-boundary-qualification.md](../../../docs/sprints/current/p0-security-levels/pf-13-s07-integrated-credential-boundary-qualification.md) (qualification merged; record ready (archive after Aggressive milestone)) | [README.md](../../security-levels/sprints/PF-13-S07/evidence/README.md) | [2](../../demos/index/PF-13-S07.md) |

## Not covered yet

- Smaller security follow-up PRs (for example #199, #200, #202, #203, #209, #219, #220, #226, #230-#232) are not in
  any task; they are fixes outside the 20 core sprint records.
- Milestone gates (code-blind VM run, human sign-off, flag removal) are not claimed by any task here.

## Evidence log

| Date | Ref | Event | Receipt |
| --- | --- | --- | --- |
| 2026-10-07 | 01-08 | Requests created; tasks generated (Proposed) | request IDs above |
| 2026-10-08 | 01-08 | Evidence records and this map merged (PR #262) | merge `319f7695f6` |
| 2026-10-08 | 09 | PF-25-S02 request created; task generated (Proposed) | request ID above |
| 2026-10-08 | 01-09 | Records list PR URLs as bare links (clean for Task Node's URL scan); PF-25-S02 added | merge `7e6ef740ef` (PR #264) |
| 2026-10-08 | 01-09 | Travis accepted; initial evidence submitted for all nine (receipts ok) | Task Node receipts (local) |
| 2026-10-08 | 08 | PF-13-S07 route count settled from the v5 result files: 11 BLOCKED of 13 rows per run (sprint record and #255 say 10) | this PR |
