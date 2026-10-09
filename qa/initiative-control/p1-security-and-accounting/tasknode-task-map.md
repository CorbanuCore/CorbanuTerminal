# Task Node task map: P1 security, accounting and CI (round 4)

Sibling of the [P0 security levels task map](../p0-security-levels/tasknode-task-map.md). Links each Task Node task to
the sprints, merged PRs, records and evidence record behind it. Use it to check any evidence sent against these tasks.
Plans: [P1 security hardening](../../../docs/plans/active/p1-security-hardening.md),
[agent cost accounting](../../../docs/plans/active/portfolio-agent-cost-accounting.md).

- Rules (Travis): Task Node moves tasks through stages; Corbanu only submits evidence of completed work. Nothing is
  submitted against closed or rewarded tasks. Work that no open task fits gets new, right-sized tasks, mapped here.
- Account: GitHub `IridiumMaster`, account `acct_oauth_73e1ab2f9f7cd01d3ee6230e`, `~/.corbanu` default profile.
- Inventory before this round (2026-10-09 15:31Z): 1 Accepted (`task_2bd6a15…`, a PostfiatL1V2 node bug; not this
  work), 0 Proposed, 0 in verification, 2 Refused, 23 Rewarded (P0SEC-TN-01..12 among them). No open task matched.
  `task_b3e8506…` ("Corbanu workstream 2: unified agent cost and usage accounting") was Rewarded on 2026-09-15 and
  gets no further submissions.
- Created: 2026-10-09, one `tasknode request create --body-file` per group, source title "Security, accounting and
  CI task map (round 4)", each tagged `Tracking ref: SECACCT-TN-NN` and sent once (a local receipt and a server-side
  tag check ran before each send). Requests sent 15:37Z; Task Node generated all eight by 15:41Z, all
  **Proposed**. Travis accepts or refuses them; Corbanu did not accept, refuse or move any.
- Scope notes: TN-01 also covers PF-27-S07's record and slices (#280-#284), and TN-04/TN-05 also cover #291 and #292;
  those PRs were not in any earlier task. PF-27-S08 (TN-03) is in progress pending Travis's key-path decision, and
  PF-60-S05 (TN-07) awaits his acceptance; both tasks say so and claim only what shipped.
- Every task asks for a committed evidence record; those are under
  [docs/research/tasknode-integration/](../../../docs/research/tasknode-integration/) (`secacct-tn-NN-*.md`).
- States below were read on 2026-10-09 at 15:41Z.

## Tasks

| Ref | Task | Reward shown | State | Scope | PRs (merge commit) | Work status | Evidence record |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SECACCT-TN-01 | `task_95e3a96c397ef87cbbba10a3036f91ef`<br>Compile the PF-27-S06 and S07 Windows Gate Closure Evidence Record | 3.5 PFT | Proposed | PF-27-S06 rerun, PF-27-S07 (P1) | [#298](https://github.com/CorbanuCore/CorbanuTerminal/pull/298) `b2ac6e6f00`, [#302](https://github.com/CorbanuCore/CorbanuTerminal/pull/302) `661b5c6a48`, [#312](https://github.com/CorbanuCore/CorbanuTerminal/pull/312) `8a566c88c4`, [#280](https://github.com/CorbanuCore/CorbanuTerminal/pull/280) `a3d470a8a5`, [#281](https://github.com/CorbanuCore/CorbanuTerminal/pull/281) `23838bbe2a`, [#282](https://github.com/CorbanuCore/CorbanuTerminal/pull/282) `97d8ae109e`, [#284](https://github.com/CorbanuCore/CorbanuTerminal/pull/284) `f755628967`, [#316](https://github.com/CorbanuCore/CorbanuTerminal/pull/316) `f4da7b2674`, [#329](https://github.com/CorbanuCore/CorbanuTerminal/pull/329) `8165c1375f` | Accepted by Travis with known limits; archived | [secacct-tn-01-windows-gates-closure-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-01-windows-gates-closure-evidence-20261009.md) |
| SECACCT-TN-02 | `task_9e695438561fca22216b86eccb784f09`<br>Compile the SECACCT-TN-02 Seven-Fix Security Evidence Record | 3.5 PFT | Proposed | Security fixes #307 #320 #300 #301 #304 #341 #310; Windows clippy | [#321](https://github.com/CorbanuCore/CorbanuTerminal/pull/321) `9810e07e31`, [#327](https://github.com/CorbanuCore/CorbanuTerminal/pull/327) `7a11f9068b`, [#331](https://github.com/CorbanuCore/CorbanuTerminal/pull/331) `62f997f3ba`, [#326](https://github.com/CorbanuCore/CorbanuTerminal/pull/326) `6b94db40f4`, [#343](https://github.com/CorbanuCore/CorbanuTerminal/pull/343) `a141b3e749`, [#317](https://github.com/CorbanuCore/CorbanuTerminal/pull/317) `5933e2eb0d`, [#313](https://github.com/CorbanuCore/CorbanuTerminal/pull/313) `8131beefb2` | Merged; issues closed | [secacct-tn-02-security-fixes-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-02-security-fixes-evidence-20261009.md) |
| SECACCT-TN-03 | `task_ee52fe14f4a911a695cf1849a3d1d6e9`<br>Compile the SECACCT-TN-03 PF-27-S08 Broker Token Evidence Record | 3.5 PFT | Proposed | PF-27-S08 slice 1 (P1) | [#333](https://github.com/CorbanuCore/CorbanuTerminal/pull/333) `d0544c1c91` | Accepted by Travis with known limits (2026-10-09, after decision (c), #360); archived | [secacct-tn-03-pf27-s08-broker-token-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-03-pf27-s08-broker-token-evidence-20261009.md) |
| SECACCT-TN-04 | `task_7e2280a41804bcd4f1e98af473c78937`<br>Compile the SECACCT-TN-04 Eight-Fix Cost Accounting Evidence Record | 3.5 PFT | Proposed | PF-60-S03 acceptance fixes, /cost wording | [#291](https://github.com/CorbanuCore/CorbanuTerminal/pull/291) `c76e6d0845`, [#299](https://github.com/CorbanuCore/CorbanuTerminal/pull/299) `1f7d8f2eec`, [#303](https://github.com/CorbanuCore/CorbanuTerminal/pull/303) `a73d40e0af`, [#305](https://github.com/CorbanuCore/CorbanuTerminal/pull/305) `f0d71e62b6`, [#306](https://github.com/CorbanuCore/CorbanuTerminal/pull/306) `b294864916`, [#318](https://github.com/CorbanuCore/CorbanuTerminal/pull/318) `290bb28532`, [#322](https://github.com/CorbanuCore/CorbanuTerminal/pull/322) `ccc38bfc03`, [#339](https://github.com/CorbanuCore/CorbanuTerminal/pull/339) `c261d7b2e5` | Merged; confirmed by re-runs | [secacct-tn-04-cost-acceptance-fixes-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-04-cost-acceptance-fixes-evidence-20261009.md) |
| SECACCT-TN-05 | `task_196d0de5eb848ffc237328b28aadfd02`<br>Compile the SECACCT-TN-05 PF-60-S03 Acceptance Evidence Record | 3.5 PFT | Proposed | PF-60-S03 independent acceptance, archive | [#292](https://github.com/CorbanuCore/CorbanuTerminal/pull/292) `a682edc61e`, [#311](https://github.com/CorbanuCore/CorbanuTerminal/pull/311) `886f19c873`, [#315](https://github.com/CorbanuCore/CorbanuTerminal/pull/315) `ef512a9d7c`, [#328](https://github.com/CorbanuCore/CorbanuTerminal/pull/328) `df44211c88`, [#335](https://github.com/CorbanuCore/CorbanuTerminal/pull/335) `0c3ad37a10` | Accepted by Travis 2026-10-09; archived | [secacct-tn-05-pf60-s03-acceptance-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-05-pf60-s03-acceptance-evidence-20261009.md) |
| SECACCT-TN-06 | `task_2fe7008e72409e148dc44f7c176c3a10`<br>Compile the SECACCT-TN-06 Cost Accounting Reel Evidence Record | 3.5 PFT | Proposed | PF-60-S03 demo videos, accounting reel | [#332](https://github.com/CorbanuCore/CorbanuTerminal/pull/332) `ec1c3b5796`, [#336](https://github.com/CorbanuCore/CorbanuTerminal/pull/336) `9c36c54808` | Published; narration voice and Breeze terms open | [secacct-tn-06-accounting-demos-reel-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-06-accounting-demos-reel-evidence-20261009.md) |
| SECACCT-TN-07 | `task_05f44e6ca71a4a02867e640f71ad72e5`<br>Compile the SECACCT-TN-07 PF-60-S05 Awaiting-Acceptance Evidence Record | 3.5 PFT | Proposed | PF-60-S05 collection correctness | [#330](https://github.com/CorbanuCore/CorbanuTerminal/pull/330) `7ac8fcb6cb`, [#334](https://github.com/CorbanuCore/CorbanuTerminal/pull/334) `8ec1aee7ab`, [#337](https://github.com/CorbanuCore/CorbanuTerminal/pull/337) `6b4b24829b`, [#338](https://github.com/CorbanuCore/CorbanuTerminal/pull/338) `2ecf6fdbe8`, [#342](https://github.com/CorbanuCore/CorbanuTerminal/pull/342) `08034c2946`, [#344](https://github.com/CorbanuCore/CorbanuTerminal/pull/344) `38362eaf57`, [#346](https://github.com/CorbanuCore/CorbanuTerminal/pull/346) `d7846e29d5`, [#355](https://github.com/CorbanuCore/CorbanuTerminal/pull/355) `3254a302fd` | **Awaiting Travis's acceptance** | [secacct-tn-07-pf60-s05-awaiting-acceptance-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-07-pf60-s05-awaiting-acceptance-evidence-20261009.md) |
| SECACCT-TN-08 | `task_646f61b2412189a65917a65719be72d6`<br>Compile the SECACCT-TN-08 CI Health Three-Fix Evidence Record | 3.5 PFT | Proposed | CI health | [#309](https://github.com/CorbanuCore/CorbanuTerminal/pull/309) `265172beed`, [#319](https://github.com/CorbanuCore/CorbanuTerminal/pull/319) `adbe1b0f81`, [#350](https://github.com/CorbanuCore/CorbanuTerminal/pull/350) `a202df29c0` | Merged; postmerge-ci green | [secacct-tn-08-ci-health-evidence-20261009.md](../../../docs/research/tasknode-integration/secacct-tn-08-ci-health-evidence-20261009.md) |

Request IDs:

- SECACCT-TN-01: `req_c40ba35afbad67a117e4a553581bdce33c6f9425aaf39d186531f7c7b13315e5`
- SECACCT-TN-02: `req_43bfc8ff66425019e2be5999b4510ee044588d8e3dc2d08d9e69d2614aab6748`
- SECACCT-TN-03: `req_4703b65bc4e8503889f1603a5e3c42eddc2036158c623dd81ffc0ece796fb8d3`
- SECACCT-TN-04: `req_2437417115760b6676ad84e648299ad76a75e2731e97c5407a8a4e8fc27bcb9f`
- SECACCT-TN-05: `req_73ae166752cda1771fec9ca88fe4473da3ac8b68b1e3753ad6c0bb5d9c657273`
- SECACCT-TN-06: `req_fefe3831866a488ed1930cdfdd8145748fa57c4118d17085b0610519266542d7`
- SECACCT-TN-07: `req_5631b641a410caadc6a1b6e56a4dd31f3e58cb4c5aafc6418eaa805de979c97c`
- SECACCT-TN-08: `req_46339cf77077ac8a28965e5a41219572f33cd9f005c726efeb6399b1c2785696`

## Sprint index

| Sprint | Task | Sprint record | Gate record | Videos |
| --- | --- | --- | --- | --- |
| PF-27-S06 | SECACCT-TN-01 (rerun, defects); code in P0SEC-TN-10 | [pf-27-s06-windows-broker-and-launch.md](../../../docs/sprints/archive/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md) (completed, archived; accepted with known limits) | [README.md](../../security-levels/sprints/PF-27-S06/README.md) | [7](../../demos/index/PF-27-S06.md) |
| PF-27-S07 | SECACCT-TN-01 | [pf-27-s07-windows-hardening-follow-ups.md](../../../docs/sprints/archive/p1-security-hardening/pf-27-s07-windows-hardening-follow-ups.md) (completed, archived; accepted with known limits) | [README.md](../../security-levels/sprints/PF-27-S07/README.md) | [6](../../demos/index/PF-27-S07.md) |
| PF-27-S08 | SECACCT-TN-03 | [pf-27-s08-windows-broker-restricted-token.md](../../../docs/sprints/archive/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md) (completed, archived; accepted with known limits) | [README.md](../../security-levels/sprints/PF-27-S08/README.md) | [2](../../demos/index/PF-27-S08.md) |
| PF-60-S03 | SECACCT-TN-04, -05, -06 | [pf-60-s03-inspectable-run-and-campaign-totals.md](../../../docs/sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md) (completed, archived; accepted 2026-10-09) | [independent acceptance](../../portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md) and three re-runs | [7 current + 3 superseded](../../demos/index/PF-60-S03.md), [reel](../../demos/index/accounting-reel.md) |
| PF-60-S05 | SECACCT-TN-07 | [pf-60-s05-collection-correctness-and-billing-basis.md](../../../docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md) (ready; awaiting Travis) | [independent acceptance](../../portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/README.md) | [3](../../demos/index/PF-60-S05.md) |

## Not covered yet

- PF-27-S08's decision and archive (#360 and its archive PR), and PF-27-S09; PF-60-S04.
- Open follow-ups named in the records: #323, #345 (Windows); #325, #351, #352 (accounting); #347, #348, #349, #353
  (CI).
- Milestone gates (code-blind VM run, human sign-off, flag removal) are not claimed by any task here.

## Evidence log

| Date | Ref | Event | Receipt |
| --- | --- | --- | --- |
| 2026-10-09 | 01-08 | Requests created; tasks generated (Proposed); evidence records and this map added | this PR |
