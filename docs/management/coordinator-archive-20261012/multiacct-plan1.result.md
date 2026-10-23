The named-accounts plan is merged: **PR #285**, merge commit `2543dfa370`. All checks passed (tmux smoke included) and both checkers pass. It's planning docs only; no code changed.

I added the feature as a new **PF-84 — Named accounts per provider** section in the proposed `unified-provider-auth.md` plan rather than a new plan. That plan is still deferred, so the five sprints are unallocated drafts. I also updated the scope, sprint map, dependency graph, acceptance flows, decisions table, both sprint indexes and the docs navigation.

## Sprint list

| Sprint | Outcome | Depends on |
|---|---|---|
| **PF-84-S01** | Fixes the wrapper problem: `~/.local/bin/corbanu` resets `CORBANU_HOME`, and an inherited `CORBANU_HOME` silently overrides `CODEX_HOME`. The fix: wrappers keep a home the caller set, the dev launcher gets checked into the repo, and the binary prints a warning when the home variables disagree. | none |
| **PF-84-S02** | Each account is a (provider, name) pair stored in the existing vault. Covers every sign-in type: API keys, Claude Plan tokens, Claude Code login, ChatGPT login, `auth.command` and AWS profiles. Today's credentials become the `default` account with no change. | none |
| **PF-84-S03** | Choosing the account: per session, `corbanu exec --account`, and an `account` option on spawned agents. Workers can run on a different account than the coordinator, in-process or in tmux. A missing or failed account is refused; it never falls back to another account. Resume uses the recorded account. | S02 |
| **PF-84-S04** | Adding, renaming, removing and switching accounts in `/providers` and onboarding. Also fixes the duplicate provider rows bug ("Anthropic Claude Account" and "Claude Account" both shown as configured). | S03 |
| **PF-84-S05** | Usage and rate limits recorded per account; `/status` and usage views grouped by provider and account. | S03, PF-60-S04 |

Each sprint includes:
- **Tests:** focused `just test` runs, a tmux test following the tmux harness guide, and a GLM 5.2 run. The GLM run uses a real `default` Z.AI key plus a named account holding a fake key; the fake account's expected 401 proves the right account was used. Then demo videos and one Opus 5.5 High review.
- **Security notes:** no new keychain items per account, and test runs never touch your real keychain. Only account names and short fingerprints appear in logs, never key values. Fake per-account canary values prove one account can't read another's credentials.

Existing single-account setups carry over unchanged with no re-login. Homes from the separate-home workaround aren't imported automatically; you'd re-add that credential as a named account.

## Decisions for you
Each has my recommendation in the plan; none blocks drafting.
- **D1 (storage):** one vault per home with account-specific labels (recommended), or one home per account. The second is proven to work but splits the vault, sessions and usage.
- **D2 (import):** no automatic import from the extra homes; re-enter the credential instead (recommended).
- **D3 (model-chosen account):** spawned agents may only name accounts that are already configured. The chosen account shows when the agent is spawned. Under the Aggressive security level it needs approval.

## Other notes
- The P0 sprint PF-76-S01 (saving provider sign-ins per named Corbanu profile) overlaps with this. Whichever starts first needs to reconcile the two.
- `multiacct1.result.md` on disk only holds the short summary. The full report with the file:line references is in `multiacct1.log`. The references are at commit `63ea3d0cbd`, which was the base for this PR, and I spot-checked several.
- The worktree is removed. The local branch wasn't deleted because Git doesn't yet see it as merged locally; nothing else was left behind.