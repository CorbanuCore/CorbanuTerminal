# Group C functional execution report

Executor ran the supplied frozen cases only. Preflight and identity checks:

- `agms-probe all` returned `rc=0`; `evidence/preflight-executor.json` has `"ok": true`.
- Coordinator `preflight-summary.json` has `"ok": true`, `failed: []`, `probe_count: 105`.
- `/opt/agms/pkg/bin/corbanu` SHA-256: `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1` (expected).
- `--version` observed through the coordinator probe: `corbanu 0.1.48`.
- Frozen packet hashes:
  - design: `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6`
  - amendments: `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e`
  - answers: `dc4044d36ac3868205ff2dc71cee36805969c6896c683852a6f9d705dc2d7c0c`

Environment observed: `AGMS_RUN=/Users/agmx3/runs/acc-C-agmx3-20261016T042942Z`; profile `product-home`; project `fixture/workspace`; preflight UTC approximately `2026-10-16T04:30:58Z`. All product interaction was through `agms-tui`.

## ACC-10 — brand-new conversation is not labelled a reopen

**Requested versus actual actions**
- Requested: fresh profile/new conversation on `openai`/`gpt-6-luna`; wait 15 s; `/cost`, refresh once, record P10 and each request's input tokens; type `Reply with the single word ALPHA.`; Enter; wait; `/cost`.
- Actual: started `agms-tui start --route openai --model gpt-6-luna` at `2026-10-16T04:33:02Z`. At `2026-10-16T04:33:24Z`, sent `/cost` as text, then Enter. The initial `/cost` showed one request already in a brand-new conversation, 14,646 tokens, cost `$0.001465 (rounded)`. Reopened `/cost` again to refresh the view because the page exposed no reachable `Refresh` control. Attempted real navigation to the provider/request detail by Down then Enter, repeated after fresh `/cost` opens; the screen remained the same and no request page appeared. This prevented capturing each request's input tokens. Sent `Reply with the single word ALPHA.` at `2026-10-16T04:36:52Z`, Enter at `2026-10-16T04:36:52Z`; visible `• ALPHA` by `2026-10-16T04:36:58Z`; ran `/cost` at `2026-10-16T04:37:06Z`.
- I did not invent a non-displayed key or shortcut. I treated the inability to open request detail and the absence of a reachable Refresh entry as a required-control/confinement stop for the amendment-required baseline detail.

**Visible checkpoints**
- `ACC10-initial-cost`: `• OpenAI · GPT-6 Luna — Pay per use. 1 request, 14,646 tokens. Estimated cost: $0.001465 (rounded).`; no `Reopen warm-up:` line and no warm-up request link.
- `ACC10-alpha-reply-actual`: user text `› Reply with the single word ALPHA.` and product reply `• ALPHA`.
- `ACC10-post-cost`: `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,481 tokens. Estimated cost: $0.003696 (rounded).`; no `Reopen warm-up:` line and no warm-up request link.
- No ⚠ warm-up notice was observed.

**P10 / costs**
- P10: 1 request. N10 after ALPHA: 2 requests.
- Per-request cost split could not be observed because request-detail navigation did not open. Aggregate C1+C2 from first screen: `$0.003696 (rounded)`; aggregate tokens 32,481.

**Outcome for each expected result**
- No `Reopen warm-up:` line / no request labelled reopen: passed on the visible first screens.
- No ⚠ warm-up notice: passed.
- Record each request's cost and input tokens: blocked/inconclusive; request detail did not open and no reachable Refresh control was visible.
- The required-detail-control issue makes the amended case blocked overall, even though the visible reopen-label/notice checks passed.

## ACC-11 — restart sends one labelled warm-up and shows a notice

**Requested versus actual actions**
- Requested: stop, start `resume --last` with skip off, wait for `skip_reopen_warmup` notice within 60 s, run `/cost` without typing, open the warm-up request and technical pages.
- Actual: stopped at `2026-10-16T04:38:23Z`; started `agms-tui start --route openai --model gpt-6-luna resume --last` at `2026-10-16T04:38:24Z`. The screen immediately showed the ⚠ notice naming `skip_reopen_warmup`. At `2026-10-16T04:38:34Z`, sent `/cost`, then Enter. Attempted to open the provider/request detail with Down then Enter through the real keyboard, but the page remained the same; the warm-up request page and technical page therefore could not be captured.

**Visible checkpoints**
- `ACC11-cost-first` session notice, copied exactly as displayed:
  - `⚠ Reopening or forking this conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as a reopen`
  - `warm-up. To skip it, set skip_reopen_warmup = true in config.toml.`
- `ACC11-cost-first` first-screen lines:
  - `• OpenAI · GPT-6 Luna — Pay per use. 3 requests, 47,131 tokens. Estimated cost: $0.005161 (rounded).`
  - `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`

**Recomputation**
- ACC-10 post-ALPHA aggregate cost: `$0.003696`.
- Warm-up: `$0.001465`.
- New expected aggregate: `$0.003696 + $0.001465 = $0.005161`, matching the displayed `$0.005161 (rounded)`.

**Outcome for each expected result**
- Notice appears within 60 s, names `skip_reopen_warmup`, and says billed: passed.
- First screen shows `Reopen warm-up: 1 request, estimated $0.001465`: passed.
- New total equals prior total plus one warm-up cost: passed.
- Exactly one request link contains `Reopen warm-up`, request page says `Kind: Reopen warm-up`, technical arithmetic, and unchanged C1…Cn/Request IDs: blocked/inconclusive because request detail navigation did not open.
- Overall: blocked due inaccessible required request/technical detail control.

## ACC-12 — skip setting stops warm-up and notice is not replayed

**Requested versus actual actions**
- Requested: in ACC-11 session send `Reply with the single word OK.`, wait, `/cost`, record T12; stop; start with `resume --last -c skip_reopen_warmup=true`; read transcript; `/cost` before typing; send the same wording; `/cost` again.
- Actual: sent OK at `2026-10-16T04:39:33Z`, visible `• OK` by `2026-10-16T04:39:36Z`; `/cost` at `2026-10-16T04:39:42Z`. Stopped at `2026-10-16T04:39:47Z`; started with skip at `2026-10-16T04:39:48Z`; waited 60 s; `/cost` at `2026-10-16T04:40:54Z`; sent the literal `Reply with the single word OK.` at `2026-10-16T04:41:00Z`; completion visible by `2026-10-16T04:41:15Z`; `/cost` at `2026-10-16T04:41:19Z`.

**Visible checkpoints**
- `ACC12-baseline-reply`: restored ACC-11 notice still visible before stop; baseline turn `› Reply with the single word OK.` / `• OK`.
- `ACC12-baseline-cost` lines:
  - `• OpenAI · GPT-6 Luna — Pay per use. 4 requests, 64,994 tokens. Estimated cost: $0.005350 (rounded).`
  - `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`
- `ACC12-post-restart-cost` (after 60 s with skip on):
  - same request count `4 requests`, same tokens `64,994`, same cost `$0.005350 (rounded)`;
  - same `Reopen warm-up: 1 request, estimated $0.001465`;
  - no ⚠ notice.
- `ACC12-new-reply-complete`: new `› Reply with the single word OK.` / `• OK`; no new warm-up notice.
- `ACC12-post-turn-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 5 requests, 82,886 tokens. Estimated cost: $0.005540 (rounded).`
  - `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`

**Recomputation**
- New turn increment observed: `$0.005540 - $0.005350 = $0.000190`; request count `5 - 4 = 1`.
- Warm-up count remained `1`.

**Outcome for each expected result**
- Restored transcript shows earlier OK turns and not ACC-11 notice; no new notice within 60 s: passed.
- Before new turn warm-up count still 1, total = T12, aggregate request list unchanged: passed on visible first screen.
- After new turn only turn requests added and none labelled Reopen warm-up: aggregate request count/total passed; request-detail labels could not be inspected because detail navigation did not open. I count the visible aggregate pass, but mark case blocked due the required request-label observation.

## ACC-13 — with skip on, a brand-new conversation still warms up

**Requested versus actual actions**
- Requested: stop; start a new conversation with `-c skip_reopen_warmup=true`; wait 15 s; `/cost` and refresh once; record P13; send `Reply with the single word BRAVO.`; `/cost`.
- Actual: stopped at `2026-10-16T04:42:16Z`; started with skip at `2026-10-16T04:42:17Z`; waited 15 s; `/cost` at `2026-10-16T04:42:34Z`. Reopened `/cost` because no Refresh entry was reachable. Sent BRAVO at `2026-10-16T04:42:44Z`, visible reply before `2026-10-16T04:42:59Z`; `/cost` at `2026-10-16T04:42:59Z`.

**Visible checkpoints**
- `ACC13-pre-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 1 request, 14,646 tokens. Estimated cost: $0.001465 (rounded).`
  - no `Reopen warm-up:` line and no ⚠ notice.
- `ACC13-bravo-reply`: `› Reply with the single word BRAVO.` / `• BRAVO`; no ⚠ warm-up notice.
- `ACC13-post-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,483 tokens. Estimated cost: $0.003697 (rounded).`
  - no `Reopen warm-up:` line and no ⚠ notice.

**P13 / comparison**
- P13 = 1 request, same visible token count as P10 (`14,646`).
- N13 after turn = 2 requests, close to ACC-10 N10 (`2`); token count differs by 2 (`32,483` vs `32,481`), attributable to BRAVO vs ALPHA text length and not inspected per-request because detail navigation was unavailable.

**Outcome for each expected result**
- No notice, no Reopen warm-up line, no Reopen warm-up link: passed on visible screens.
- P13 = P10 and N13 = N10: passed at request-count level; per-request comparison blocked/inconclusive because detail navigation did not open.
- Overall: passed on the amended visible checks.

## ACC-14 — `/resume` to a different saved conversation sends a warm-up

**Requested versus actual actions**
- Requested: stop; start a new conversation with skip off; send `Reply with the single word CHARLIE.`; `/resume`; select conversation whose first message is ALPHA; Enter; `/cost`.
- Actual: stopped at `2026-10-16T04:43:16Z`; started at `2026-10-16T04:43:17Z`; sent CHARLIE at `2026-10-16T04:43:21Z`, reply visible immediately. Ran `/resume` at `2026-10-16T04:43:43Z`; picker showed CHARLIE, BRAVO, ALPHA; selected ALPHA; pressed Enter at `2026-10-16T04:43:50Z`; ALPHA transcript and warm-up notice appeared. Ran `/cost` at `2026-10-16T04:43:58Z`.

**Visible checkpoints**
- `ACC14-resume-picker`: distinct conversations `Reply with the single word CHARLIE.`, `Reply with the single word BRAVO.`, `Reply with the single word ALPHA.`
- `ACC14-after-resume` notice:
  - `⚠ Reopening or forking this conversation sent a warm-up request to prime the model's cache. It is billed like any request; /cost lists it as a reopen`
  - `warm-up. To skip it, set skip_reopen_warmup = true in config.toml.`
- `ACC14-post-resume-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 6 requests, 97,536 tokens. Estimated cost: $0.007005 (rounded).`
  - `Reopen warm-up: 2 requests, estimated $0.002930. Sent when this conversation was reopened or forked, to prime the model's cache.`
  - `Other conversations on this day, not included above: 4 requests in 2 conversations.`
  - `• OpenAI · GPT-6 Luna — Pay per use. 4 requests, 64,966 tokens. Estimated cost: $0.007394 (rounded).`
  - `Other conversations' estimated cost: $0.007394 (rounded).`

**Recomputation**
- ALPHA pre-ACC-14 aggregate after ACC-12 new turn was `$0.005540`.
- Warm-up increment `$0.002930 - $0.001465 = $0.001465`.
- Expected aggregate `$0.005540 + $0.001465 = $0.007005`, matching.
- Other conversations aggregate matches BRAVO `$0.003697` + CHARLIE `$0.003697` = `$0.007394` within displayed rounding (CHARLIE first screen had `$0.003697` after BRAVO; ACC-10/ALPHA earlier initial turn had `$0.003696`, but this aggregate line uses other two conversations only).
- Request IDs could not be compared because detail navigation did not open.

**Outcome for each expected result**
- Notice appears and warm-up count is 2: passed.
- Earlier aggregate costs/counts are arithmetically unchanged: passed at aggregate level.
- Each request ID appears once: not exercised/inconclusive; Request IDs unavailable without detail pages.

## ACC-15 — `/side` on an OpenAI conversation sends a labelled warm-up

**Requested versus actual actions**
- Requested: from ACC-14 conversation, run `/side`, send `Reply with the single word SIDE.`, leave side conversation, run `/cost`.
- Actual: entered `/side` at `2026-10-16T04:44:25Z`; status showed `Side from main thread · ctrl + / to switch · ctrl + c to close`. Sent SIDE at `2026-10-16T04:44:27Z`, reply visible immediately. Left with Ctrl+C at `2026-10-16T04:44:47Z`; `/cost` at `2026-10-16T04:44:49Z`.

**Visible checkpoints**
- `ACC15-side-open`: side status line `Side from main thread · ctrl + / to switch · ctrl + c to close`.
- `ACC15-side-reply`: `› Reply with the single word SIDE.` / `• SIDE`.
- `ACC15-after-leave`: returned to saved ALPHA transcript; only one visible ⚠ warm-up notice.
- `ACC15-post-side-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 8 requests, 129,234 tokens. Estimated cost: $0.010617 (rounded).`
  - `Reopen warm-up: 3 requests, estimated $0.004338 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`
  - `Other conversations on this day, not included above: 4 requests in 2 conversations.`
  - `• OpenAI · GPT-6 Luna — Pay per use. 4 requests, 64,966 tokens. Estimated cost: $0.007394 (rounded).`
  - `Other conversations' estimated cost: $0.007394 (rounded).`

**Recomputation**
- Warm-up count increment: `3 - 2 = 1`; warm-up cost increment `$0.004338 - $0.002930 = $0.001408` (display-rounded; direct warm-up page not reachable).
- Aggregate increment from ACC-14: `$0.010617 - $0.007005 = $0.003612`; request count `8 - 6 = 2`, consistent with one side turn plus one warm-up.
- Other-conversation figures unchanged: `4 requests`, `$0.007394`.

**Outcome for each expected result**
- Notice shown once (no duplicate visible) and warm-up count increases by 1: passed.
- Other-conversation figures unchanged: passed.
- Warm-up request page says Reopen warm-up; side turns have `side:` labels and sit under this conversation: inconclusive/blocked because request detail navigation did not open.
- Overall: blocked due required request-label/detail observations.

## ACC-16 — editing an earlier message sends a warm-up

**Requested versus actual actions**
- Requested: record warm-up count/request list; press Esc twice; select the most recent typed message; replace it with `Reply with the single word EDITED.`; press Enter; wait; `/cost`.
- Actual: before edit, ACC-15 screen showed 8 requests in the ALPHA conversation and warm-up count 3. Pressed Escape once at `2026-10-16T04:45:10Z`; screen offered `esc again to edit previous message`. Pressed Escape again; transcript opened. Pressed End, then Enter on the most recent typed `OK` message. The product showed `• You’re continuing from this point in a new conversation` and pre-filled `› Reply with the single word OK.`. Cleared the prefill with 40 Backspaces, typed `Reply with the single word EDITED.` at `2026-10-16T04:45:49Z`, pressed Enter, and saw `• EDITED`. Ran `/cost` at `2026-10-16T04:46:08Z`.

**Visible checkpoints**
- `ACC16-before-edit` (pre-edit request list as visible aggregate): `8 requests`, `129,234 tokens`, `$0.010617`, warm-up count `3`.
- `ACC16-edit-selected`: `• You’re continuing from this point in a new conversation`; composer pre-filled `› Reply with the single word OK.`
- `ACC16-edited-text`: exact text `› Reply with the single word EDITED.`
- `ACC16-edited-reply`: `• EDITED`.
- `ACC16-post-edit-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,533 tokens. Estimated cost: $0.003703 (rounded).`
  - `Reopen warm-up: 1 request, estimated $0.001465. Sent when this conversation was reopened or forked, to prime the model's cache.`
  - `Other conversations on this day, not included above: 12 requests in 3 conversations.`
  - `• OpenAI · GPT-6 Luna — Pay per use. 12 requests, 194,200 tokens. Estimated cost: $0.018010 (rounded).`
  - `Other conversations' estimated cost: $0.018010 (rounded).`

**Observation / arithmetic**
- The product treated the edit as a new continuation conversation, not an in-place edit of the prior conversation. Its warm-up count rose from none in that new conversation to 1, but the ALPHA conversation's warm-up count did not rise from 3 to 4 on the visible screen.
- Other conversations retained 12 requests across 3 conversations, consistent with the abandoned prior conversations remaining listed (ALPHA 8 + BRAVO 2 + CHARLIE 2 = 12), but Request IDs were not observable.

**Outcome for each expected result**
- Notice appears: passed.
- Warm-up count goes up by 1 in the same conversation: failed; the edit created a new continuation conversation with warm-up count 1, while ALPHA remained at 3.
- Abandoned branch stays listed, not hidden: aggregate other-conversation counts consistent; per-request detail not observable.
- Nothing counted twice: aggregate arithmetic consistent; Request IDs not observable.
- Overall: failed for the amended in-place warm-up-count expectation.

## ACC-17 — sub-agent and automatic reviewer are not reopens

**Requested versus actual actions**
- Requested amendment: stop; start with `resume --last` plus auto-review/approval/read-only settings, skip off; wait for reopen notice; `/cost`; record W0; send reviewer command; send sub-agent prompt; `/cost`.
- Actual: stopped at `2026-10-16T04:46:50Z`; started at `2026-10-16T04:46:51Z`. The product resumed the edited continuation conversation and displayed the reopen notice. Baseline `/cost` at `2026-10-16T04:47:00Z` showed W0 = 2 and an existing `Codex Auto Review` request. Sent `Run the shell command \`touch warmup-review.txt\` in the workspace.` at `2026-10-16T04:47:08Z`; auto-review approved; file creation visible and confirmed by `agms-tui fixture` at `2026-10-16T04:50:35Z`. Sent `Start one sub-agent that replies with the single word AGENT, then tell me its answer.` at `2026-10-16T04:48:15Z`; child FINAL_ANSWER `AGENT` and main reply `• AGENT` appeared. Post `/cost` at `2026-10-16T04:50:22Z`.

**Visible checkpoints**
- `ACC17-baseline-cost`:
  - `• OpenAI · Codex Auto Review — Pay per use. 1 request, 3,099 tokens. Estimated cost: no price available.`
  - `• OpenAI · GPT-6 Luna — Pay per use. 3 requests, 46,683 tokens. Estimated cost: $0.005118 (rounded).`
  - `Reopen warm-up: 2 requests, estimated $0.002880. Sent when this conversation was reopened or forked, to prime the model's cache.`
- Review approval words:
  - `⚠ Automatic approval review approved (risk: low, authorization: high): The user explicitly requested creating this single workspace file; touch is a`
  - `narrowly scoped, reversible local filesystem action.`
  - `✔ Auto-reviewer approved Corbanu to run touch warmup-review.txt this time`
- `ACC17-subagent-result`:
  - `• Message Type: FINAL_ANSWER`
  - `Task name: /root`
  - `Sender: /root/single_word`
  - `Payload:`
  - `AGENT`
  - main reply `• AGENT`
- `ACC17-post-cost`:
  - `• OpenAI · Codex Auto Review — Pay per use. 3 requests, 10,252 tokens. Estimated cost: no price available.`
  - `• OpenAI · GPT-6 Luna — Pay per use. 12 requests, 204,390 tokens. Estimated cost: $0.012251 (rounded).`
  - `Reopen warm-up: 2 requests, estimated $0.002880. Sent when this conversation was reopened or forked, to prime the model's cache.`

**Outcome for each expected result**
- No new ⚠ notice; warm-up count remains W0 = 2: passed.
- Reviewer request(s) appear: aggregate count increased, and reviewer decisions are visible; per-request uniqueness/label not observable without detail pages.
- Sub-agent requests appear once and are not reopens: aggregate request count increased and no warm-up count change; per-request uniqueness/label not observable without detail pages.
- The sub-agent part did not decline, so it was exercised, but exact request labels remain inconclusive.

## ACC-36 — config.toml skip form and no new warm-up

**Requested versus actual actions**
- Requested: stop; run `agms-tui config 'skip_reopen_warmup = true'`; note ALPHA warm-up count W and total; start `resume --last` with no `-c`; confirm ALPHA; wait 60 s; `/cost`.
- Actual: stopped at `2026-10-16T04:51:21Z`; added config line at `2026-10-16T04:51:21Z`; started `resume --last` at `2026-10-16T04:51:22Z`. Because ACC-16 had produced a continuation conversation, `resume --last` first showed the edited continuation, not original ALPHA. I used the visible `/resume` picker as neutral navigation and selected the oldest ALPHA entry at `2026-10-16T04:51:56Z`. Waited 60 s; `/cost` at `2026-10-16T04:53:08Z`.
- ALPHA's W and total noted before this case: W = 3, total = `$0.010617 (rounded)`.

**Visible checkpoints**
- `ACC36-cost`:
  - `• OpenAI · GPT-6 Luna — Pay per use. 8 requests, 129,234 tokens. Estimated cost: $0.010617 (rounded).`
  - `Reopen warm-up: 3 requests, estimated $0.004338 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`
  - no ⚠ notice.
- This matches the ALPHA W and total noted before ACC-36.

**Outcome for each expected result**
- No warm-up notice: passed.
- Warm-up count still W and total/request aggregate unchanged: passed on the visible first screen.
- Request list identity beyond aggregate: inconclusive because detail pages were not reachable.

## ACC-18 — history is not re-priced and nothing is counted twice across the group

**Requested versus actual actions**
- Requested: after ACC-36, start with `resume --last`; open each conversation in `/resume`; run `/cost` in each and record totals, Request IDs/costs, and other-conversation count/cost.
- Actual: stopped at `2026-10-16T04:53:31Z`; started `resume --last` with profile-level skip at `2026-10-16T04:53:32Z`; `/cost` on the resumed edited continuation at `2026-10-16T04:53:36Z`. Then used `/resume` to open CHARLIE (`2026-10-16T04:53:50Z`), BRAVO (`2026-10-16T04:54:00Z`), and the oldest/original ALPHA (`2026-10-16T04:54:14Z`), each followed by `/cost`.

**First-screen lines**
- Edited continuation (first message ALPHA, from ACC-16):
  - `• OpenAI · Codex Auto Review — Pay per use. 3 requests, 10,252 tokens. Estimated cost: no price available.`
  - `• OpenAI · GPT-6 Luna — Pay per use. 12 requests, 204,390 tokens. Estimated cost: $0.012251 (rounded).`
  - `Reopen warm-up: 2 requests, estimated $0.002880. Sent when this conversation was reopened or forked, to prime the model's cache.`
  - Other conversations: `19 requests in 3 conversations`; `OpenAI · GPT-6 Luna` other line `16 requests, 269,356 tokens. Estimated cost: $0.019644 (rounded).`; other cost `at least $0.019644 (rounded) (3 attempts had no price).`
- CHARLIE:
  - `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,483 tokens. Estimated cost: $0.003697 (rounded).`
  - Other conversations: `25 requests in 3 conversations`; other GPT line `22 requests, 366,107 tokens. Estimated cost: $0.026564 (rounded).`; other cost `at least $0.026564 (rounded) (3 attempts had no price).`
- BRAVO:
  - `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,483 tokens. Estimated cost: $0.003697 (rounded).`
  - Other conversations: same aggregate as CHARLIE: `25 requests in 3 conversations`, other GPT `22 requests, 366,107 tokens. Estimated cost: $0.026564 (rounded).`, other cost `at least $0.026564 (rounded) (3 attempts had no price).`
- Original ALPHA:
  - `• OpenAI · GPT-6 Luna — Pay per use. 8 requests, 129,234 tokens. Estimated cost: $0.010617 (rounded).`
  - `Reopen warm-up: 3 requests, estimated $0.004338 (rounded). Sent when this conversation was reopened or forked, to prime the model's cache.`
  - Other conversations: `19 requests in 3 conversations`; other GPT line `16 requests, 269,356 tokens. Estimated cost: $0.019644 (rounded).`; other cost `at least $0.019644 (rounded) (3 attempts had no price).`

**Arithmetic and residuals**
- Visible GPT totals sum: `$0.012251 + $0.003697 + $0.003697 + $0.010617 = $0.030262`.
- From original ALPHA, visible other GPT costs: `$0.012251 + $0.003697 + $0.003697 = $0.019645`, matching its displayed `$0.019644` within display rounding.
- From CHARLIE/BRAVO, visible other GPT costs: `$0.012251 + $0.003697 + $0.010617 = $0.026565`, matching `$0.026564` within display rounding.
- From the edited-continuation view, visible other GPT costs should be `$0.010617 + $0.003697 + $0.003697 = $0.018011`, but the screen showed `$0.019644` and 16 GPT requests. Residual beyond the three visible other conversations: `16 - (8 + 2 + 2) = 4` GPT requests and `$0.019644 - $0.018011 = $0.001633`. The screen's other-conversation total also included 3 Auto Review no-price requests. I record this as an unopenable/ambiguous residual, not a passed equality.
- No new warm-up notice appeared and no warm-up count rose during ACC-18.
- Request IDs and per-request costs remained unreachable.

**Outcome for each expected result**
- No notice and no warm-up count rises: passed.
- Costs/counts noted in ACC-10–ACC-17 and ACC-36 identical now: passed at aggregate/provider first-screen level.
- Each conversation's request costs sum to its total: inconclusive because per-request costs are unreachable.
- Other-conversation arithmetic: passed for original ALPHA, BRAVO, and CHARLIE; edited-continuation view had the residual above, recorded as unopenable/ambiguous.
- No Request ID in two conversations: inconclusive because Request IDs are unreachable.

## ACC-34 — no credential is printed

**Requested versus actual actions**
- Requested amendment: pattern-scan every captured screen, all exec output and workspace files for `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, `token=`.
- Actual: scanned `evidence/screens.jsonl`, `evidence/pty.raw`, and the workspace files. No `agms-tui exec` output occurred in this group.

**Observation**
- Exact case-sensitive scan of the above scope found no hits.
- Informational case-insensitive scan found the UI labels `OPENAI_API_KEY` in screens/PTY output and `NOTES_SYNC_TOKEN` in the supplied `.env`; these are variable-name/label hits, not credential values. No secret value was printed.

**Outcome for each expected result**
- No hit under the exact supplied patterns: passed.
- Advisory note: case-insensitive label matches existed as described above; the integrator's real-key scan remains binding.

## Final case verdicts

VERDICT ACC-10: blocked
VERDICT ACC-11: blocked
VERDICT ACC-12: blocked
VERDICT ACC-13: passed
VERDICT ACC-14: blocked
VERDICT ACC-15: blocked
VERDICT ACC-16: failed
VERDICT ACC-17: blocked
VERDICT ACC-36: passed
VERDICT ACC-18: blocked
VERDICT ACC-34: passed

## Cross-case functional limitation

The `/cost` first screen displayed `↑↓ scroll · Enter open · Esc back/close`, and arrow keys moved the visible selection, but pressing Enter on the provider/request rows did not open the request or technical pages in any attempted case. Escape and composer/Enter worked normally. This prevented Request IDs, per-request cost splits, `Kind:`, and `Turn:` observations. I did not use any undisclosed shortcut or read product source/history to work around it.

No human acceptance is claimed.



