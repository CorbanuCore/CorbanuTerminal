# Coherence Audit — One Authority Per Fact

Date: 2026-07-25
Branch: `feat/multimodel-native-orchestration`
Head at audit: `a23ee3aa7`

## Why this exists

Eleven qualification resets found eleven different bugs. They are one bug: two
parts of the system each own a piece of one decision and disagree. Section 6.1
of the mandate already requires "one authority per fact." Nobody enumerated the
facts, so violations were found one at a time by running 45-minute sessions and
waiting for a symptom.

This document enumerates the facts, names every current owner, and gives the
action. It is finite. It does not require a live session to check.

Prior resets, restated as ownership conflicts:

| Reset | Fact with two owners                                                     |
| ----- | ------------------------------------------------------------------------ |
| 8A    | what history reaches the provider (native transport vs. chat adapter)    |
| 8C    | who is in a crew (CrewSpec vs. recovery role inference)                  |
| 8D    | what runtime a thread uses (CrewSpec vs. resuming parent pane)           |
| 8E    | whether a turn may run (execution limiter had no control-plane tier)     |
| 8I    | what the model may request (tool schema vs. validator)                   |
| 8L    | provider request shape (repair gated to one transport, not the class)    |
| 8O    | what runtimes exist (picker catalog vs. tool contract vs. validator)     |
| 8S    | whether a completion was delivered (native mailbox vs. TUI report queue) |
| 8U    | recovery scope (children reconciled, roots not)                          |
| lease | whether a turn may run (execution admission vs. provider lease)          |

---

## Fact 1 — Whether a turn may run right now

**Should have one owner. Currently has five independent gates, none aware of the
others.**

| Gate               | Location                                 | Scope                              | Knows who is asking?   |
| ------------------ | ---------------------------------------- | ---------------------------------- | ---------------------- |
| Execution limiter  | `core/src/agent/control/execution.rs:53` | active turns vs `max_threads`      | yes — sub-agent only   |
| Residency limiter  | `core/src/agent/control/residency.rs:97` | resident threads, LRU unload       | no                     |
| Spawn registry cap | `core/src/agent/registry.rs:87`          | total threads per session          | no                     |
| Provider lease     | `core/src/session/turn.rs:2414`          | one large request per provider key | yes, as of `a23ee3aa7` |
| Provider cooldown  | `core/src/session/turn.rs:2222`          | post-429 backoff per key           | no                     |

Three of the five raise the same `CodexErr::AgentLimitReached`, so the user
cannot tell which limit was hit.

This is the direct cause of two resets. Phase 8E taught the execution limiter
that human input is control-plane work. The provider lease had never heard of
that idea, so a human turn was admitted and then blocked one layer down —
exactly the failure reported tonight, sixteen hours later. Commit `a23ee3aa7`
taught the lease the same lesson. Three gates still do not know.

**Action.** Collapse to one admission decision that takes the requester
(human control plane vs. autonomous worker) and the resource being claimed
(execution slot, residency slot, thread budget, provider key), and returns one
typed answer. Each existing limiter becomes a resource policy behind it, not an
independent veto. Distinguish the error cases so "thread budget exhausted" and
"provider key busy" are not both `AgentLimitReached`.

Cooldown should remain everyone-applies: it is a real server backoff, not local
contention. That distinction is a property of the resource, which is exactly
what a single admission decision can express and five scattered ones cannot.

---

## Fact 2 — What runtime (provider / model / effort) a thread uses

**Should have one owner. Currently written in five places.**

| Owner                                  | Location                             |
| -------------------------------------- | ------------------------------------ |
| `CrewSpec` member runtime request      | `protocol/src/crew.rs`               |
| TUI map `spawn_native_runtime_by_node` | `tui/src/app.rs:762`                 |
| `SpawnThreadStateMetadata` to state DB | `tui/src/spawn_orchestration.rs:111` |
| Rollout session metadata               | per-thread `.jsonl`                  |
| Live `TurnContext`                     | `core/src/session/turn.rs`           |

Phase 8D fixed one disagreement between these (bind/resume overwrote the crew
runtime with the focused parent pane's). Phase 6C fixed another. Both were
symptoms of five writers.

**Action.** The persisted thread record is the single owner. `CrewSpec` states
_intent_ at creation; it must not be read back as the live answer.
`spawn_native_runtime_by_node` becomes a read-through cache or is deleted —
23 production references.

---

## Fact 3 — Who is in a crew, and its shape

**Should have one owner. Currently four.**

| Owner                                                      | Location                        |
| ---------------------------------------------------------- | ------------------------------- |
| `CrewInstanceState` logical-to-native mapping              | `tui/src/crew_state.rs`         |
| TUI maps `spawn_parent_by_node` / `spawn_parent_by_thread` | `tui/src/app.rs:760-761`        |
| TUI map `spawn_native_endpoint_by_node`                    | `tui/src/app.rs:764`            |
| Native registry `AgentMetadata.agent_path`                 | `core/src/agent/registry.rs:36` |

`agent_path` already encodes parentage canonically. `spawn_parent_by_node` has
74 production references restating it. Phase 8C's leak — an unrelated native
root reclassified as a Troll — was possible only because a second parentage map
existed to leak into.

**Action.** Native `agent_path` owns parentage. `CrewInstanceState` owns
membership and display identity only. Delete `spawn_parent_by_node`,
`spawn_parent_by_thread`, and `spawn_native_endpoint_by_node` as authorities;
derive them if a lookup index is still wanted.

---

## Fact 4 — Whether a message was delivered and applied

**One owner, achieved in Phase 8T.** The durable mailbox with stable
`completion:<thread>:<turn>` identity is the only native completion transport.

Residual TUI bookkeeping still exists and should be confirmed dead or deleted:

| Field                                | Production refs |
| ------------------------------------ | --------------- |
| `spawn_dispatch_acks_by_target_task` | 7               |
| `spawn_next_dispatch_seq`            | 15              |
| `spawn_processed_dispatch_seq_ids`   | 23              |
| `spawn_processed_dispatch_origins`   | 8               |
| `spawn_processed_terminal_turns`     | 7               |
| `spawn_parent_reports_by_node`       | 11              |

Phase 7C deleted the pump; `tui/src/dispatch_queue.rs` survives at 247 lines and
is still imported by `spawn_orchestration.rs`, `claude_panes/`, and `app.rs`.
Some of that is the legitimate external-Claude edge adapter. The native paths
through it are the duplicate.

**Action.** Separate the external-Claude adapter from native residue, then
delete the native half. This is the same deletion Phase 8S ordered; it was done
for the report queue but not for the surrounding dispatch bookkeeping.

---

## Fact 5 — What runtimes exist and what the model may request

**Should have one owner. Currently three, reconciled by hand.**

Phase 8O added `canonical_catalog_provider` to the shared provider registry and
pointed both the tool planner and the TUI model picker at it. That was the right
move and is mostly done.

Remaining: the tool schema, the spawn validator, and the picker still derive
their views separately. 8I (schema hid runtime fields, so an agent silently ran
Opus while labeled Kimi) and 8O (contract omitted provider IDs, so an agent
could not name a runtime that the backend could run) were both this fact.

**Action.** One registry function returns the requestable runtime set. Schema,
validator, and picker all call it. Regression: every runtime the validator
accepts is present in the schema, and vice versa.

---

## Fact 6 — Whether a provider is usable

**One owner, achieved in commit `2f6eb1e48`.** All three creation paths await
`ensure_native_spawn_provider_ready`, which now executes the provider's
configured auth command rather than only checking OpenAI account auth and vault
keys.

No action.

---

## Fact 7 — What may be spent

**Owner: `agents.provider_allowlist`, as of commit `3996f17a9`.** Partially
closed; the spend _cap_ remains unowned.

What was repaired:

- `agents.provider_allowlist` is operator policy, enforced at
  `ensure_native_spawn_provider_ready` — the single chokepoint already shared by
  `/spawn` crew creation, custom-crew members, and native `spawn_agent` task
  agents. Native task agents were previously outside `CrewPolicy` entirely,
  which is the path that actually spent the money.
- `crew_state.rs::add_ready_member` no longer pushes a member's provider into
  the allowlist before validating. A member on an unauthorized provider is
  refused and crew state is left untouched.
- A custom crew declares `authorized_spawn_providers()` rather than the first
  runtime a model happened to request.
- Unset remains unrestricted, so existing configurations are unchanged.

What remains open:

- `maximum_spend_usd` still has zero production readers. Enforcing a dollar cap
  needs per-turn cost accounting that does not exist yet; provider authorization
  was the reachable half.
- There is still no cost signal in any watcher or status surface.

Original finding, retained:

`CrewPolicy` declares the fields and nothing enforces them:

- `maximum_spend_usd` — `protocol/src/crew.rs:126`. Zero production readers.
  Hardcoded `None` at `crew_presets.rs:87` and `custom_spawn_crew.rs:54`.
- `provider_allowlist` — enforced at `crew.rs:164` and `crew_state.rs:119`, but
  built from the provider the model just requested at
  `custom_spawn_crew.rs:53`. The model effectively grants its own allowlist.

Mandate section 12 requires that a model cannot broaden a provider allowlist or
spend cap. That boundary does not exist.

Consequence: after Phase 8O exposed all 23 picker-visible runtimes to native
`spawn_agent`, unsupervised agents could select metered API routes freely. Six
qualification sessions ran `anthropic / claude-opus-5` and
`anthropic / claude-fable-5`, some at `xhigh`, on metered API keys. That spend
was not authorized. The section 15.3 invariant watcher has no cost signal, which
is why eleven resets never surfaced it.

**Remaining action.** Add per-turn cost accounting and enforce
`maximum_spend_usd` against it, or delete the field so it stops implying a
guarantee that does not exist.

---

## Ordered work

1. ~~**Fact 7, spend.**~~ Provider authorization closed in `3996f17a9`. Spend
   cap still unowned.
2. **Fact 1, admission.** Five gates to one decision. Retires the failure family
   that produced 8E and tonight's lease block, and stops the next one.
3. **Fact 3, parentage.** Delete two of three parentage maps.
4. **Fact 4, dispatch residue.** Finish the Phase 8S deletion.
5. **Fact 2, runtime.** Demote `CrewSpec` to intent, persisted record to truth.
6. **Fact 5, catalog.** One registry function behind schema, validator, picker.

## On the qualification gate

Three consecutive 45-60 minute clean sessions cannot converge while the artifact
must change after every finding, and it measures luck when this many facts have
multiple owners — a passing session says nothing about the owners it did not
touch. Two of the eleven resets (8S, 8U) came from post-hoc audits of sessions
that had already passed live.

Structural checks are the better gate: for each fact above, one owner, named,
with a regression proving the former duplicates cannot decide it. That is
checkable without a live session and does not reset.
