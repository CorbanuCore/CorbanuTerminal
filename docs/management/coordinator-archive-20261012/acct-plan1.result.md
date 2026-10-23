I've written the new sprint, PF-60-S05, and the options memo for the "both" question. PR #330 merged into main (merge `7ac8fcb6cb`) with all checks passing. The repo's plan and sprint checkers and the checker's 24 unit tests pass.

- **Sprint:** `docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md`
  - It is a draft, blocked on your decision about "both". It now runs fourth.
  - PF-60-S04 moves to fifth and now depends on S03 and S05. I updated the plan, the sprint index and the docs menu to match.
  - S05 depends only on S02, so it doesn't wait for S03 to be accepted.
- **Options memo:** `qa/portfolio/agent-cost-accounting/pf-60-s05/both-behaviour-options.md`
- **Default billing table:** `qa/portfolio/agent-cost-accounting/pf-60-s05/billing-basis-defaults.md`. It lists all 21 built-in providers, each with its official source.
  - **Subscription:** Claude Plan, Kimi Code, Z.AI's Anthropic route, and OpenAI when signed in with ChatGPT.
  - **Local, no charge:** Ollama and LM Studio.
  - **Pay per use:** everything else, including OpenAI with an API key and the Corbanu API.
  - **Needs a live check:** Z.AI's Anthropic route. Z.AI's docs say it bills a balance only for accounts that never bought a Coding Plan, so the sprint includes a real-provider test for it.
- **S03:** untouched. Its to-do list still lists the five problems; S05 says to mark them as covered there when it finishes.

**For Travis: what happens when one provider can be paid two ways**

Sometimes the same company can charge you two ways. You might have a membership and a separate pay-as-you-go balance (Kimi Code and Kimi's developer platform, for example). Or a membership can keep running on paid extras after you hit its limit. Claude, for example, lets you switch to pay-as-you-go at standard API rates instead of being blocked when you reach a limit. Kimi offers a paid "Extra Usage" balance, but only to subscribed members. The terminal can't tell, request by request, whether your plan or the extra balance paid.

The example day in the memo uses made-up prices. You actually spent $6.75: $3.00 on an Anthropic API key and $3.75 on Claude extras after the plan limit. Your Z.AI Coding Plan work was covered by the plan.

- **A. One label per provider.** If you point Z.AI at its Coding Plan address, you also have to mark it as a subscription yourself; otherwise the plan work shows up as spending. Extras aren't counted. Shows $4.84 spent if you forget to mark it, $3.00 if you don't.
- **B. The label follows the key and address actually used (recommended).** Coding Plan address means subscription, the normal address means pay per use, and so on. You set nothing up. An address we don't know shows "not declared" rather than a guess. Extras aren't counted, and the screen says so. Shows $3.00 spent.
- **C. B, plus a warning line.** For a subscription with extras switched on, it adds "your plan may have charged up to $X extra today". The warning figure is usually far too high: up to $15.00 in the example, against $3.75 actually charged.
- **D. Count anything uncertain as spent.** It never shows too little, but it shows subscription work as spending again, which is the bug we're fixing. Shows $19.84 spent.

**I recommend B.** It gets the common case right with no setup, it follows a declared rule instead of guessing from how you logged in, and it says plainly what it can't see. We can add C later if a provider ever reports, request by request, when extras paid.