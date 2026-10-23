I recommend accepting S04 behind the flag after one bug, #461, is fixed. Everything in `/providers` passes, but onboarding's "Add another account" quits the TUI when you type `q`, and the rest of a typed key then shows up in clear on the terminal.

I tested current `origin/main` at `c7e6e07cbc` as a macOS debug build, not the packaged release. I compared it against the build just before S04 landed (`2f95b4fc9a`). All runs used disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and real Z.AI (GLM 5.3 Flash), Kimi k3, Claude Plan and OpenAI keys. A local proxy recorded which account each request used. The report is PR #465, merged with all checks green.

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| S04-1 | One row per (provider, account) in /providers | PASS | Rows like `Z.AI · fake  Named account · API key · fp de063d3d0b35`, with "this session" and "default for new sessions" markers that move correctly. |
| S04-2a | Add an account (masked key or Claude token); never replaces an existing one | PASS | Entry is masked for Z.AI, Kimi, Claude and the proxy provider; an existing name is refused, both on add and on rename. |
| S04-2b | Add a Claude Code login folder or Claude sign-in account | NOT VERIFIABLE | The folder path is saved but I never used it for a request; sign-in was not run. |
| S04-3 | Rename | PASS | The key is unchanged (same fingerprint, same key seen by the proxy); renaming this session's account is refused; a default follows the rename. |
| S04-4 | Remove only that account; an account in use needs a replacement | PASS | Asks for a replacement first. The `/vault` list went from `aa1, aa2, quota` to `aa2, quota`. |
| S04-5 | Use for this session | PASS | Starts a new session on that account (proxy saw only that account's key); the saved default is unchanged after restart. |
| S04-6 | Make default | PASS | Written to `[provider_accounts]` in config.toml; after restart `/status` shows `account work` and the proxy saw the work key. |
| S04-7 | Duplicate-row fix | PASS | The old build shows both "Anthropic Claude Account" and "Claude Account" as configured; this build shows one row, with the flag on and off. |
| S04-8 | Onboarding: one row, then use / add another / replace | **FAIL** (#461) | The choices and saving work, but typing `q` in the name or key field quits the TUI and saves nothing. |
| S04-9 | `/status` and spawn line show the account | PASS | `/status`: `· account fake`. Spawn line: `(glm-5.3-flash max · account main)` and `· account fake`. |
| S04-10 | Esc changes nothing | PASS | Esc at every step; config and vault file hashes unchanged in a saved re-run. |
| S04-11 | Fake key gives a 401 naming the account | PASS | "account `fake` was rejected …" on Z.AI, Kimi and Claude Plan; exactly one request, no retry on another account (the proxy provider ran with retries off). |
| S04-12 | Masked entry; no key in any artifact | PASS for /providers; **FAIL** for onboarding (#461) | Scans of every test home, log and capture found no full or partial real key and no test value; the only hit is the OpenAI key in `auth.json`, where it is normally stored. |
| S04-13 | Flag off: unchanged | PASS | No account rows, actions or `/status` suffix; `[provider_accounts]` is ignored with a warning. |
| S04-14 | Existing single-account home untouched | PASS | A home set up by the old build keeps identical file hashes on the new build, flag off and on, and still answers. |
| S04-15 | Missing account fails, never falls back | PASS | Picking an account removed behind the TUI's back is refused and the session keeps its account; a removed default makes the TUI exit before start, with 0 requests. |
| S04-16a | Works across providers | PASS | Z.AI, Kimi, Claude Plan and a custom provider; two Claude accounts survive a restart. |
| S04-16b | Named ChatGPT and AWS accounts stay refused | NOT VERIFIABLE | OpenAI offers no "Add another account" and `--account openai:work` is refused, but ChatGPT sign-in and AWS were not run. |
| S04-17 | Reuses the shared setup code | NOT VERIFIABLE | Internal; not visible from outside. |

**Issues filed:**
- **#461 (medium-low, security-relevant):** typing `q` in onboarding's "Add another account" name or key field quits the TUI, and any remaining key characters land on the terminal in clear. Pastes from most terminals are not affected, and `/providers` is fine.
- **#462 (low):** also happens on the old build, so not caused by S04.
  - `/status` says "OpenAI API key configured" when there is none.
  - Onboarding loops when config.toml already names the provider.

The one independent reviewer (Opus 5.5 High, code-blind) asked for changes to the report's evidence only. I attached the missing outputs, ran the vault listing and split the partial verdicts; REVIEW.md has its text and my response. One limit: the same agent designed and ran the tests, so the sprint's code-blind handoff-checker step and the milestone VM gate are still open. Scratch homes, both build folders and the worktrees are deleted.

**Recommendation for Travis:** accept S04 behind the flag. Adding, renaming, removing, switching and making an account default all work with real providers and survive a restart. The duplicate Claude row is gone, and existing homes are untouched. Fix #461 before the flag is removed; it is small but affects key entry.