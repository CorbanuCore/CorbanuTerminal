You are an accounting UX worker (developer-only /cost screens). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: keychain rule, disk hygiene, per-fix
gate) and acct-fix2.result.md in that folder. Two changes Travis asked for on 2026-10-09, one PR (or two if cleaner):

1. Grayed date-format hint for /cost date bounds. Today the accepted forms are only shown in an error after a wrong
   input ("Usage: /cost [YYYY-MM-DD] (UTC, no future dates). Custom: /cost START END hour|day|week|month; dates or UTC
   timestamps ending Z; end exclusive."). Add dim/gray placeholder hint text that appears while the user is typing the
   command's arguments in the composer (e.g. after `/cost ` show something like `[YYYY-MM-DD]  or  START END day|week|month  (UTC)`),
   and disappears as they type. Find how the composer/slash-command popup renders hints (check whether there is an
   existing argument-hint mechanism for slash commands; reuse it, or add a small generic one that any slash command can
   declare). Also show the same short format line, dimmed, in the /cost range/day page header area or footer if there is
   a natural place, so the bounds are explained on screen. Keep wording short and plain; match the existing style. Only
   in builds where /cost exists (developer-accounting); default builds unchanged.
2. Replace the jargon "Billed cost: unavailable — no settlement evidence" and "Estimate versus billed difference: unknown —
   no settlement evidence" lines. Fact: the provider's actual charge is only collected when it states it with each response
   (OpenRouter and the Corbanu API, see core/src/accounting_chat.rs `billed_usd`); nothing else (no invoices, no billing
   API) is ever collected. New behaviour: when the provider reported its charge, keep "Billed cost: $X — as stated by the
   provider with each response"; otherwise say in plain words that the provider doesn't report what it actually charged,
   so this is an estimate — check the provider's bill (name the provider(s) where known). Don't repeat this line on every
   page if it's noise: one clear line per page is enough. Update every place it appears (tokens.rs incl. range header ~1865,
   tokens/scope.rs, tests, snapshots, user docs that quote it).
Gate: focused codex-tui tests with and without developer-accounting (snapshots updated deliberately), Linux clippy on the
RTX box (ssh ambient@100.99.88.49) -D warnings, one independent Opus 5.5 High review via installed `corbanu exec`
(read-only), and a short TUI capture of the hint and the reworded lines with a freshly built debug binary (disposable home,
CORBANU_TEST_NO_NATIVE_KEYRING=1, GLM 5.2 on zai with `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"`
on the consuming command). Another worker (acct-s05, PF-60-S05) is changing accounting code concurrently; merge
origin/main often and keep this change confined to presentation. New worktree off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main, merge when green (`gh pr merge N --merge --match-head-commit <sha>`;
never force-push; never rm -f). Remove your build target when done. Report with before/after text of every changed line.
