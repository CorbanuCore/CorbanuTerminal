# Manager/review inference model switch — September 14, 2026

Authority: Travis (chat, 2026-09-14 ~23:20 UTC), citing quota pressure: after the
current round of sprints completes, use **Opus 5.0 High** for all independent
reviews and everywhere **Fable 5.1 High** was used. Recorded as coordinator event
`travis-directive:model-switch-opus5-20260914`.

## What changes

| Surface | Before | After |
| --- | --- | --- |
| Fresh manager decision loop (`fable_launcher.MODEL`) | `claude-fable-5-1-plan` | `claude-opus-5-plan` |
| Independent code reviews (`autoreview --model`) | `claude-fable-5-1-plan` | `claude-opus-5-plan` |
| Evidence reviews (`review-fable-high exec --model`) | `claude-fable-5-1-plan` | `claude-opus-5-plan` |
| Provider / effort | `claude-plan` / `high` | unchanged |

`claude-opus-5-plan` is `CLAUDE_PLAN_MODEL` in `codex-rs/model-provider-info`
(upstream `claude-opus-5`). Nothing else about the launcher changes: same
private home, TMUX socket, read-only sandbox, ACK/receipt contract and identity
checks (`manager_cycle` compares the session's recorded model to `f.MODEL`).

## Verification

- `test_fable_launcher.py` fixture header derives its model line from `f.MODEL`
  instead of a literal so the identity check is exercised for the new slug.
- Python suite (`unittest discover -s scripts/initiative_control -p '*test*.py'`,
  SDK interpreter) recorded in the receiving receipt for this change.
- First live decision cycle on the new model is the qualification of this switch;
  its run receipt must show `model="claude-opus-5-plan"`. Until that receipt exists
  the switch is code-landed, not qualified.

Historical receipts, reviews and QA notes that name `claude-fable-5-1-plan` remain
accurate for their dates and are not rewritten.
