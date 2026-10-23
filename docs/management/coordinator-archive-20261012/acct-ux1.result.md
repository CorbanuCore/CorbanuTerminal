PR #339 is merged into main (merge commit `c261d7b2e5`). It has both changes: the grayed date hint for `/cost` and the plain billed-cost wording. It only affects builds with `developer-accounting`; default builds look the same to users.

## Every changed line, before → after

**Composer**
- Typing `/cost ` used to show nothing after the space. Now it shows, grayed: `[YYYY-MM-DD]  or  START END hour|day|week|month  (UTC)`. It disappears once you type an argument.
- The hint uses a small mechanism any slash command can declare; there wasn't one before. Only `/cost` declares a hint so far.
- It is hidden when it wouldn't fit on the line in full, in multi-line or bash input, and where `/cost` can't be sent (for example a thread owned by a parent agent).

**`/cost` screens**
- A new grayed line sits above `↑↓ scroll · Enter open · Esc back/close` on every page: `Other dates: /cost [YYYY-MM-DD]  or  START END hour|day|week|month  (UTC)`.
- **First page of a day:** the two jargon lines are gone. The page now has one line about billing:
  - Before: `Costs are estimates from published prices; your provider's bill is the final amount.`
  - After: `Costs are estimates from published prices; Z.AI doesn't report its charges — check Z.AI's bill.`
  - The old wording stays when no provider can be named, for example an empty day.
- **Provider, request, attempt, attempt-group and unknown-parent pages:** the two jargon lines are replaced by one of these:
  - The provider stated its charge: unchanged, `Billed cost: $X — as stated by the provider with each response`.
  - The provider never reports charges: `Billed cost: Z.AI doesn't report its charges, so any cost here is an estimate — check Z.AI's bill.`
  - The provider normally reports charges (OpenRouter, Vercel, the Corbanu API) but didn't on these responses: `Billed cost: OpenRouter stated no charge for this work, so any cost here is an estimate — check OpenRouter's bill.`
  - Both kinds on one page: `Billed cost: Z.AI doesn't report its charges and OpenRouter stated no charge for this work, so any cost here is an estimate — check their bills.`
  - A refused request: `Billed cost: none stated — OpenRouter reported no usage here; a refused request is normally not charged, but if it stopped mid-response, check OpenRouter's bill.`
  - No provider recorded: `Billed cost: not reported, so any cost here is an estimate — check your provider's bill.`
  - The amount is too large to add up exactly: `Billed cost: unavailable — exact arithmetic overflow`.
  - Subscription-only or empty pages: no billing line at all, where before they had `Billed cost: unavailable — no settlement evidence`.
- **Pages that show no figures** (needs refresh, missing thread, range too large, and similar): `Billed cost: unavailable — no settlement evidence` is removed.
- **Range header:**
  - Before: `Collection coverage: unknown. Range estimate covers root and resolved descendants; unknown ancestry stays separate in bucket breakdowns. Billed cost: unavailable — no settlement evidence.`
  - After: the same line without the last sentence.
  - A complete range now gets one billing line in the forms above. A partial range gets none, because it shows no total.
- **Docs:** no user docs quote the old lines. The research notes and older QA records that do are dated records, so I left them unchanged.

## Gate
- **Tests:** the focused codex-tui tests (cost screens, composer, slash commands) pass with and without `developer-accounting`: 555 each. Snapshots were updated deliberately.
- **Full codex-tui test run:** two tests fail on both builds. Both fail regardless of this change:
  - the command-popup snapshot, which is out of date on macOS;
  - `default_daemon_auto_connect_probes_socket_only`, because the test socket path was too long.
- **Linux clippy** on the RTX box with `-D warnings`: clean with and without the feature, on the final commit `de4df99`.
- **Review:** Opus 5.5 High through installed `corbanu exec`, read-only.
  - **Round 1:** asked for changes, with one Major. A provider that normally reports charges, such as OpenRouter, was described as never reporting them when one response had no charge. I fixed it.
  - **Round 2:** approved with nits, and I fixed those too.
- **TUI check:** a freshly built debug binary, in a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, after a real GLM 5.2 turn on zai. The key was only on the `exec` command, and a scan found it in no file.
  - The screen shows 0.02937828 USD, which matches my own calculation.
  - Every page I opened has exactly one billing line.
  - The captures, with paths redacted, and both reviews are in `qa/portfolio/agent-cost-accounting/pf-60-s03/cost-hint-wording-20261009/`. The captures come from `914024b6e1`; the final commit only adds the round-2 fixes, which none of the captured pages show.
- **CI:** all checks on the PR passed.
- **Cleanup:** my build target, worktree, scratch homes and RTX folder and scripts are removed. The local branch `ux/cost-date-hint-billed-wording` was not deleted.

## Still open
- **The list of providers that report charges now exists twice.** The screens keep their own copy of the list in `core` (OpenRouter, Vercel, the Corbanu API); the two must be kept in step when a provider is added.
- **Local models (for acct-s05):** the screens now leave local-model work out of the billing line and the "1 of N attempts stated a charge" count, since there's no bill for it. The first page of a day with only local-model work still uses the general "your provider's bill" wording. That page's wording belongs to acct-s05's local-model work.
- **When a provider stated charges for only some attempts**, the line still reads "at least $X (1 of 3 attempts stated a charge)". It doesn't name whose bill covers the rest; I kept that line as you asked.
- **CI never runs the codex-tui tests or clippy with `developer-accounting` on**, so I ran them myself.