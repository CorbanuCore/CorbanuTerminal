PF-84-S05 is merged and set to `ready` (not archived). All CI checks were green when I merged. Per-account usage and throttle state now work, and the usage view lines up exactly with the token counts the providers reported. The usage view only exists in developer-accounting builds; see the first decision below.

**PRs**

| PR | What | State |
|---|---|---|
| #455 | Allocates S05's worktree, branch, base and write scope | merged `c2a7f6d5cd` |
| #469 | S05, behind `named_accounts` | merged `d93f5e639d` at head `81b35b02ee` (27 checks passed) |

**What S05 adds**
- **Usage records:** each accounted request on a named account now stores the account name and its 12-character fingerprint, never the key.
  - Requests on the default account store nothing new, so existing records keep their exact bytes and read as `default`.
  - The record format moves from 3 to 4 only when the first named-account request is written. An older developer build then refuses that file by name, as it already does for any newer format.
- **Rate limits:** the local 429 cooldown and request lease are kept per named account, so one account's 429 never blocks another. The default account's key is unchanged.
- **`/cost`:** rows, provider pages and request links are grouped by provider, account and model. That covers this conversation with its spawned agents, plus `corbanu exec` workers in the same home. Accounts are named only when the view contains named-account work. `/status` already shows the session's account (S04).

**Gate evidence** (`qa/provider-auth/pf-84/s05-gate.md`)
- **Tests on the final merged tree (`just test`):** codex-state 407/407; focused codex-core 199/199 with developer-accounting and 194/194 without.
  - Focused codex-tui: 619/623 with developer-accounting and 619/622 without. Three failures are the known "path must be shorter than SUN_LEN" environment problem on this drive.
  - The fourth failure, one `/cost` tmux test, timed out once with the machine under load. It passed alone and 2/2 on rerun.
- **Clippy `-D warnings`:** clean on macOS and on glitch (`-j10`), with and without developer-accounting.
- **Live tmux run on GLM 5.3 Flash:**
  - Coordinator on the default Z.AI account.
  - Spawned agents: `work` (the real key enrolled under a second name) replied pong; `fake` got a 401.
  - `corbanu exec` workers ran on both accounts; Kimi `alt` was the second provider.
  - `/cost` totals equal the provider-reported totals from each session's log:

    | Account | Tokens |
    |---|---|
    | Z.AI default | 46,115 |
    | Z.AI work | 11,544 |
    | Z.AI fake | 0 (one failed request, no spend) |
    | Kimi default | 7,232 |
    | Kimi alt | 7,246 |

  - The leak scan found no key in any file, and the folders holding real keys were deleted.
- **Video** (leak-scanned, 0 hits): [usage by account](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s05-pf84-usage-by-account-a1921d736bfa-2026-10-11.mp4).
- **Opus 5.5 High review:** approved with nothing blocking. I fixed the non-blocking items:
  - the fingerprint lookup is capped at 2 s;
  - a sampling is refused if the account changes partway through;
  - a fingerprint is reused only for the same provider and account;
  - each `/cost` view names accounts consistently;
  - tests were added.

  The fixes were not re-reviewed.

**Open items**
- **Known limits:**
  - If a named account's key is replaced under the same name, an active 429 cooldown carries over.
  - Rate-limit figures in `/status` come only from ChatGPT sign-ins, which get named accounts in S06.
  - Separate homes are not combined.
- **Skipped review suggestions:**
  - extra end-to-end tests for compaction, websocket and extension requests (they share the binding the unit test covers);
  - a ChatGPT fingerprint that may change on token refresh, which is S02 code, noted for S06.
- **Code-blind acceptance for S05** has not been run; it is a separate step.
- **Clean-up:** my worktrees, branches, build folder and glitch files are removed.

**For Travis to decide**
1. **Per-account usage in release builds.** Under your earlier decision, cost tracking exists only in debug builds with developer-accounting, so release users get per-account rate limits and the account in `/status`, but no per-account usage view.
   - Options: (a) keep it developer-only; (b) add a light per-account token summary to `/status` that needs no cost records; (c) turn on cost tracking in release builds through the PF-60 follow-ups.
   - I recommend (b) as a small follow-up.
2. D1–D3 are still on the recommended defaults; nothing new depends on them.