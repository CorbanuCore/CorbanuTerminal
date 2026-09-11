# Task Node workstream targets — 2026-09-11

Travis explicitly authorized three dedicated workstream tasks. Created once each
using installed Corbanu 0.1.36's native `tasknode request create --kind personal`,
then reconciled durable requests and read the generated tasks in the linked
IridiumMaster / @iridiumeagle account. All three are **Proposed**, not Accepted,
submitted, completed or rewarded. Generated wording was inspected and matches
coordination-only scope; no implementation or security completion is asserted.

| Stream | Generated Task Node task | Mapped sprint IDs |
| --- | --- | --- |
| 1. PF-13 security | `task_865f75c6911f953c6586cdc1f4531e4f` — Corbanu workstream 1: PF-13 security and protected credentials | PF-13-S07 |
| 2. Accounting | `task_b3e8506327fb906173fd68b2f642221b` — Corbanu workstream 2: unified agent cost and usage accounting | PF-60-S01–S04 |
| 3. Task Node | `task_789a0f3bd75b41d1eca20cae698f04cf` — Corbanu workstream 3: Task Node integration and beta-test coordination | PF-80-S01, PF-79-S01–S02 |

Open [Task Node](https://tasknode.postfiat.org) under the linked account to inspect
the proposed offers. These personal coordination tasks are not the future public
beta assignments. Their target IDs are recorded in manager-owned `control.json`;
mapping does not accept a task or enable posting. PF-35 and unrelated old tasks
remain unmapped and untouched.

Main integration note: the table now uses receiving PF-80-S01 for delivery
control. The live operations source still uses its historical PF-76-S01 mapping;
the manager must reconcile aliases and the queued batch before source cutover.
Main's provider-persistence PF-76 must never inherit that mapping. No server
configuration or durable event was changed by this documentation migration.

## Durable request receipts

- PF-13: `req_78cc3c281b31001633db0dd30e4c11799e696112dac12dd55c75b8397bbbe876`.
- Accounting: `req_37bbb471f60f3db6405bcac01e1a24604afcdb24eb3458607b6222d6fc378e8b`.
- Task Node: `req_ab214d6a17dea80268125f0fdab734251440251dfa9a9e5167094e482bec577b`.

Each request reached `proposed` / `offer_published` with the corresponding
generated task ID. Reconcile these IDs on uncertainty; repeating the create
command with a fresh generated idempotency key can duplicate work.

## Remaining boundary

Automatic progress remains OFF: remote publisher credential permission,
entitlement/enrollment, compatible progress lifecycle and a reviewed first-event
receipt are pending. Check whether proposed tasks are valid progress targets;
if acceptance is required, ask Travis rather than silently accepting. No seed,
password, wallet signing, transaction, verification or reward action was needed.
The [beta program](tasknode-beta-program.md) is planned, not publicly launched.
