# #368 item 1: OpenRouter requests priced from the reported charge

Requirement: PF-60-S07 "Remaining", AC1 (`docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s07-price-catalogue-refresh.md`):
"a request is priced from the charge OpenRouter reports for it when present, labelled as the provider's charge;
without one, the catalogue rate is shown only as a labelled estimate, or 'no price'". This is a bounded fix under
Travis's 2026-10-14 decision on #368, PR #784. The code-blind functional cases for it run in PF-60-S10.

Travis's decision, 2026-10-14: an OpenRouter request is priced from the charge OpenRouter reports with the
response (`usage.cost`, recorded as `billed_usd`). The catalogue rate is shown only as a labelled fallback
estimate when no charge is reported, and "no price" is shown when neither exists. `/cost` and the request page
say which basis was used.

## Fail-before / pass-after

`codex-rs/tui/src/chatwidget/tokens/charge_tests.rs` builds attempts from mocked OpenRouter response bodies:
one with `usage.cost`, one without, one bring-your-own-key (`is_byok: true`, which is not the request's charge),
and one with a charge on a model the catalogue doesn't list. It reads them with the same `chat_body_usage`
parser the accounting path uses.

| Tree | Result |
| --- | --- |
| Base `dcfcb2048a` plus only the first 7 tests | 6 of 7 fail: no "charged by OpenRouter", no `Price basis`, and the day total uses the charged requests' catalogue estimates. The parser guard passes. |
| This change | All 13 pass. The first 7 plus 6 added in review: partly priced attempts, a $0 charge, refused attempts, other conversations and ranges. |

The core suite test `accounting_chat_openrouter_records_the_reported_charge` streams a mocked OpenRouter Chat
response with and without `usage.cost`. The ledger observation records `billed_usd` 0.000123 and 0.000015, and
none when no cost is reported.

## Real OpenRouter requests (tmux, disposable home, vault label `provider/openrouter_api_key`)

| Model (upstream) | OpenRouter `total_cost` (generation API) | Ledger `billed_usd` | `/cost` | Catalogue estimate (shown as not used) |
| --- | --- | --- | --- | --- |
| `moonshotai/kimi-k3` (Phala), 8,461 in / 26 out | 0.02190705 | 0.02190705 | `Cost: $0.021907 (rounded), charged by OpenRouter`; exact 0.02190705 | $0.025773 |
| `z-ai/glm-5.3-flash` (InferenceNet), 8,866 in / 21 out | 0.00088975 | 0.00088975 | `Cost: $0.000890 (rounded), charged by OpenRouter` | $0.0013404 |

Generation ids: `gen-1791977664-L7qc5doi4aXhsXZwGOp3` and `gen-1791977204-WjHp05v927B9iNAho00h`. Three more
GLM 5.3 Flash runs were served at the catalogue rate (Friendli and others), so their charge equalled the
estimate.

Video: PF-60-S07 `pf60s07-openrouter-charge-cost` (see `qa/demos/index/PF-60-S07.md`).

## Found on the way (not changed here)

A real `minimax/minimax-m3` response reported `reasoning_tokens` 37 with `completion_tokens` 36. The ledger
rejects that ("reasoning exceeds output"), so the request is sent unrecorded and `/cost` omits it. Filed as #786.
