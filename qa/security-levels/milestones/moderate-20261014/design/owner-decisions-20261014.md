# Owner decisions for the Moderate milestone run (as of 2026-10-14)

Verbatim owner-decision comments, then the questions still open with the owner as they were put to him. Nothing
else (no code, issues' technical analysis or test results).

## #750: Moderate decisions D1–D7 (Travis, 2026-10-14, recorded 07:05Z)

> Owner decision (Travis, 2026-10-14): Moderate milestone decisions: D1 (a) ship with today's controls, unbuilt ones
> shown 'not available'; D2 (a) credential broker on by default (own setting wins); D3 (b) Moderate never runs
> outside the sandbox and stops offering it (#747); D4 (b) approvals at confirmation like #662, the rest at next
> start; D5 (b) child agents allowed with Moderate enforced; D6 (a) keep Permissive-era prefix rules for ordinary
> commands; D7 (a) macOS-only run with a stand-in project, Claude-pane cases out of scope, Linux + real TensorCash
> before flag removal. Implement the Moderate picker sprint (#750 with #747 #749).

The options the decisions chose from, as put to the owner (readiness assessment, section 5):

| # | Decision | Options |
| --- | --- | --- |
| D1 | What Moderate ships with | (a) the controls that exist today, with the unbuilt ones (financial preview and mandates, isolated retrieval, screened search, browser login, quarantine, detectors, Agent Sweep) shown "not available"; (b) wait for those P1 sprints |
| D2 | Model keys through the broker under Moderate | (a) on by default, as under Aggressive, a setting you make still wins; (b) stay off |
| D3 | Commands a person approves to run outside the sandbox | (a) let them run under Moderate; (b) Moderate never runs outside the sandbox, and the prompt, inspector and docs stop offering it |
| D4 | When Moderate applies after confirming | (a) like Aggressive today: some at once, the rest at the next start; (b) approvals at confirmation, the rest (sandbox, network, environment, broker, child-agent rules) at the next start; (c) everything at confirmation |
| D5 | Agents started by an agent under Moderate | (a) refuse, as Aggressive's default; (b) allow, with Moderate enforced in the new agent |
| D6 | "Don't ask again" rules saved under Permissive, used under Moderate | (a) keep them for ordinary commands; after untrusted content they already don't count for protected ones; (b) suspend them as under Aggressive |
| D7 | Scope of the milestone run | (a) macOS only, synthetic project as the TensorCash stand-in, Claude-pane cases out of scope, limits named; (b) add a Linux run on glitch and real TensorCash access |

## #447: model-decline rule (Travis, 2026-10-14, recorded 07:04Z; excerpt of the comment)

> Model-decline rule accepted for Aggressive and Moderate: a case passes when the model declines a step but the
> product's text still meets the expected wording; blocked otherwise.

## Open with the owner (recorded as named limits for this run unless answered before execution)

1. **#766, nested `corbanu exec` under Moderate (D5).** Agents started inside Corbanu Terminal inherit Moderate. When
   an agent command itself starts `corbanu exec`: with the same Corbanu home it stops at start; pointed at another
   Corbanu home it isn't held to Moderate. Options put to the owner: (a) carry Moderate into nested runs (needs
   security sign-off); (b) refuse nested launches under Moderate (contradicts D5); (c) accept today's behaviour as a
   named limit for the milestone run. Recommendation: (c) now, then (a).
2. **#783, Moderate on Windows.** Out of scope for this macOS-only run (D7).
3. **D4 reading.** Core can't switch on Moderate's approval rules alone, so confirming raises Core's whole level at
   once: the sandbox rule and the protected-file rules also apply at confirmation, which is stricter than a literal
   "everything else at the next start", never weaker. The credential broker and the "active" label wait for the
   restart. Recommendation: keep this reading.
