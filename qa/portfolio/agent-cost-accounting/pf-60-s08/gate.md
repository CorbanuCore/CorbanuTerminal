# PF-60-S08 gate — `/cost` page clarity (#396) and the reopen warm-up (#403)

Travis's 2026-10-14 decision on #403: option (C). The reopen warm-up is shown in `/cost` and on reopen, with a setting
to skip it, off by default, so behaviour stays as today. #396's two display fixes are in the same sprint. Code-blind
execution is PF-60-S10's job.

## What changed

- **Reopen warm-up (#403).**
  - A conversation the user resumes or forks (`/resume`, `resume --last` after a restart, `/side`, editing an earlier
    message, app-server `thread/fork`) labels its startup warm-up `prewarm:reopen:<id>`. It is still a `prewarm:`
    label, so anything that reads warm-ups still reads it. A sub-agent, guardian review or internal worker started
    with copied history is not a reopen (`session_startup_prewarm::is_reopen`; review M1): its warm-up stays an
    ordinary one, with no notice, and the setting does not skip it.
  - Once the warm-up has gone out over the WebSocket and completed, the session shows: "Reopening or forking this
    conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as
    a reopen warm-up. To skip it, set skip_reopen_warmup = true in config.toml." A warm-up that fell back to HTTP sent
    nothing and says nothing. One cut short (timeout, interrupted first turn) says nothing but is still listed in
    `/cost`. The notice isn't written to the conversation, so a later reopen doesn't replay it.
  - `/cost` shows "Reopen warm-up: N request(s), <cost>. Sent when this conversation was reopened or forked, to prime
    the model's cache." on the first screen. The request's link and page are named "Reopen warm-up", and the page says how to
    skip it.
  - `skip_reopen_warmup = true` (default `false`; config schema and `docs/config.md`) sends no warm-up on reopen. Auth
    is still prewarmed. A new conversation still warms up.
- **#396.1.** A component with no rate states no money. A 0-token one reads `none — 0 tokens` instead of `$0.000000`.
  Components with a rate keep their figure, zero included.
- **#396.2.** The basis-only price line reads `records only its billing basis: pay per use; set in your config`, with
  no parentheses nested inside parentheses. A component's rate counts its long-context tier too.

## Evidence

| Check | Result |
| --- | --- |
| Fail-before | The S08 sources were reverted to the base with the tests kept. The core reopen test (run without the new setting) fails: no notice on reopen. The three TUI tests fail: `accounting_reopen_warmup_is_named_with_its_cost`, `accounting_unpriced_zero_token_component_states_no_money` and the updated `accounting_inspect_basis_only_request_shows_no_price_metadata`. |
| Pass-after | `accounting_reopen_warmup_is_labelled_announced_and_skippable` (core suite) covers both settings. On reopen the gate counts one more warm-up, one `prewarm:reopen:` attempt and the notice. With skip set, none of the three. A new conversation warms up either way. `only_the_users_resume_or_fork_is_a_reopen` pins which sessions are reopens (sub-agents, guardian and review forks, internal workers are not). `reopen_prewarm_label_is_stable` pins the prefix the TUI matches. The three TUI tests and `config_schema_matches_fixture` pass. |
| `just test -p codex-core -E 'test(accounting) \| test(spawn) \| test(prewarm) \| test(websocket) \| test(config) \| test(reopen)'` | 1115/1115 with `developer-accounting`; 985/985 without. Final tree after the merge from main (review Minor 3). |
| `just test -p codex-tui --features developer-accounting -E 'test(tokens) \| test(charge) \| test(cost) \| test(resume)'` | 308/309. The failure is the `cost_scope` tmux test, which fails only on this Mac (see the PF-60-S07 gate). |
| Clippy `-D warnings --tests` | `codex-core`, `codex-tui` and `codex-config`, with and without `developer-accounting`: clean on macOS and on Linux (glitch) for the final code. |
| tmux, real OpenAI key (vault `openai-api-key`), disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1` | GPT-6 Luna. [Restart with `resume --last`](tmux-20261014/restart-notice.txt) shows the notice (final build). `/cost` then shows `Reopen warm-up: 1 request, estimated $0.001465` and `Request 3 · Reopen warm-up` ([first screen](tmux-20261014/restart-first-screen.txt), [request](tmux-20261014/restart-reopen-warmup-request.txt)). `/resume` of the open conversation does nothing ([capture](tmux-20261014/resume-same-conversation.txt)). With `skip_reopen_warmup = true`, a restart shows no notice and sends no warm-up: after one more turn, `/cost` still lists one reopen warm-up, and requests 1–3 keep their figures ([screen](tmux-20261014/skip-reopen-screen.txt), [first screen](tmux-20261014/skip-first-screen.txt)). Fast tier (GPT-5.6 Luna, `service_tier = "priority"`, unpriced): `Cache read cost: none — 0 tokens` ([technical](tmux-20261014/fast-tier-technical.txt)). Declared basis (custom provider at `api.openai.com/v1` with `billing = "pay_per_use"`), re-captured on the final build: `records only its billing basis: pay per use; set in your config` ([technical](tmux-20261014/declared-basis-technical.txt)). The Fast-tier, skip and `/resume` captures are from the first S08 build; their output did not change after it. |
| Key scan | OpenAI key: 0 hits outside the disposable homes' own `auth.json` (written by `login --with-api-key`). Those homes were deleted afterwards. |
| Video | [Reopen warm-up via `/side`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s08-pf60s08-reopen-warmup-cost-92f8a4790940-2026-10-14.mp4) (31 s, GPT-6 Luna, real key, final build) |

Spend: about ten GPT-6 Luna and GPT-5.6 Luna requests, roughly $0.02 in all.

## Observations

- **Custom provider pointed at `api.openai.com`.** Every turn is refused with HTTP 400 ("collaboration.followup_task is
  reserved"). Filed as #820; not caused by this change. The declared-basis capture is that session's one recorded request.
- **A reopen can cost two warm-ups when approvals go to the auto-reviewer.** The reopen's warm-up path also starts the
  reviewer's session early, and that new session sends its own ordinary warm-up (`review:prewarm:`). Only the first
  is named and announced. With the skip setting, the reviewer's session starts at the first review instead.
- **Profiles** can't set `skip_reopen_warmup`; the top level and `-c` can (`docs/config.md`).
