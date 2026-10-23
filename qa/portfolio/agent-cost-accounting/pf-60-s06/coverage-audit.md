# Sessions with no thread of their own, 2026-10-14 (PF-60-S06)

*Updates the [PF-60-S05 audit](../pf-60-s05/acct-coverage-audit-20261009.md) for #400 and #401. Same method: every
place a session is made with `ephemeral = true`, or spawned without a saved thread, and where its paid requests go
(`Session::accounting_owner`, `codex-rs/core/src/session/mod.rs`).*

A session with no thread of its own records under the conversation it works for, labelled with what it is. When
that conversation is not saved either, its own owner is used and the labels nest, so the work is counted once under
the saved conversation at the root. Labels are ordinary turn labels; the ledger format is unchanged.

| Session kind | Where it is made | Recorded under | Label |
| --- | --- | --- | --- |
| Guardian approval review (trunk and ephemeral forks) | `core/src/guardian/review_session.rs` | the conversation it reviews | `review:` |
| Guardian review of a tool call inside `/side` | same | the conversation `/side` came from | `side:review:` (#400) |
| Memory phase-2 consolidation agent | `memories/write/src/phase2.rs` | the conversation that started it | `consolidation:` |
| TUI `/side` conversation | `tui/src/app/side.rs` (`side_fork_config`); the fork request names it with thread source `side` (`tui/src/app_server_session.rs`) | the conversation it came from | `side:` (#401: only this kind) |
| Any other ephemeral fork of a saved conversation (another client's `thread/fork` with `ephemeral`, a fork in an ephemeral TUI, another feature's fork) | app-server `thread/fork` | the conversation it came from | `fork:` (#401) |
| Sub-agent with no thread of its own (spawned from an ephemeral conversation) | `core/src/agent/control/spawn.rs` | the conversation that spawned it | `agent:`; from `/side`: `side:agent:` (#400) |
| `/review` task with no thread of its own | `core/src/tasks/review.rs` via `codex_delegate.rs` | the conversation that started it | `code-review:` |
| `exec --ephemeral` | the exec CLI | not collected | one `accounting.excluded = "ephemeral_session"` log line |
| Anything working for an `exec --ephemeral` conversation | — | not collected | one `accounting.excluded = "parent_not_persisted"` line |
| Prompt debug | `core/src/prompt_debug.rs` | not inference | — |
| `thread-manager-sample` | sample binary | not shipped | — |

A parent that is not saved is followed while it is live in the same process, at most four levels deep. If it has
already ended (a sub-agent that outlives its `/side` conversation), the work is excluded with
`parent_not_persisted`, as before.

Sessions that have a saved thread of their own (an ordinary sub-agent of a saved conversation) still record under
that thread; this sprint does not change them.

The sequence above is pinned by `codex-rs/core/tests/suite/accounting_side_forks.rs` and
`codex-rs/core/src/session/tests/guardian_tests.rs` (`accounting_owner_of_*`).

**Side effect of the `/side` marker.** The thread source `side` also travels where the thread source already went:
the turn-metadata request header sent to the provider and the analytics events. A `/side` turn used to report
`user` there and now reports `side`. Nothing branches on the value; ephemeral threads are never saved, so thread
lists and resume are unaffected.
