# Coverage audit, re-run 2026-10-09 (PF-60-S05 AC9)

*Same method as the [21 September audit](../pf-60-s03/acct-coverage-audit-20260921.md): every client that can send a
model request, found by its client construction sites, raw posts to inference-shaped paths and credential holders,
then each named as collected, not collected or not inference. New here: the session level. A client can be wrapped
and still be uncollected because its session has no thread to record under. Checked on branch
`work/pf60-s05-s4-sessions`.*

## Clients

The 21 September table still holds. Every model-client session is created through `new_model_client_session()`:
turns, compaction (local, remote and remote v2, plus the legacy endpoint), startup prewarm and realtime calls. All of
them attach through `attach_turn` or `attach_scopes`. Extensions (image generation, web search) and pane-bridge
reports record through `ExtensionAccounting`. Nothing new posts to an inference path outside those.

## Sessions with no thread of their own

A session created with `ephemeral = true` persists no thread. Before S05 its paid requests were dropped without a log
line (review Major 5). Now:

| Session kind | Where it is made ephemeral | Collected? |
| --- | --- | --- |
| Guardian approval review fork | `core/src/guardian/review_session.rs` (`fork_config.ephemeral = true`) | **Yes**, under the conversation it reviews for, as turns labelled `review:` (`Session::accounting_owner`) |
| `exec --ephemeral` | the exec CLI | No. One `accounting.excluded = "ephemeral_session"` warning per session |
| Side conversation (`/side`) | `tui/src/app/side.rs` (`side_fork_config`) | No. Same named warning |
| Memory phase-2 consolidation agent | `memories/write/src/phase2.rs` | No. Same named warning |
| Prompt debug | `core/src/prompt_debug.rs` | Not inference: it builds a prompt and sends nothing |
| `thread-manager-sample` | sample binary | Not shipped |

The named warning goes to the log (target `codex_core::accounting`) once per session, the first time that session
would have recorded. Collection is developer-only, so this is developer evidence, not a user-facing notice.

## Still not collected, by construction

Unchanged from 21 September: `responses-api-proxy` and credentials brokered to child processes spend from outside
this process.

## Follow-up for the product owner

Side conversations and memory consolidation are real paid inference with no thread of their own. Attributing them to
their parent conversation, as guardian forks now are, is a small change, but it is outside S05's mandate (review
Major 5 named guardian forks). Recommended as an S04 or follow-up item.
