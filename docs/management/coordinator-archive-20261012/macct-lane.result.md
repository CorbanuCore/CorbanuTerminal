PF-84-S01 and S02 are merged to main and set to `ready`, and neither is archived. I stopped before S03 because my context is getting large, so S03–S05 are still drafts and need a fresh worker.

**What shipped**

| PR | What it does | State |
|---|---|---|
| #407 | Activates the plan in the slot freed by accounting. S01 and S02 got their own worktrees, branches, base and non-overlapping write scopes. D1–D3 are recorded as "recommended defaults, Travis may revisit", and PF-76-S01 is reconciled: a profile selects a named account rather than adding its own credential store. | merged |
| #408 | **S01:** `corbanu` now keeps a home the caller set, in both the release wrapper and the dev launcher. The dev launcher is now in the repo as `scripts/dev/corbanu-launcher.sh`. When the home variables disagree, corbanu prints one warning naming the home it uses. | merged `d700f926c2`, S01 `ready` |
| #409 | **S02:** named accounts per provider, behind the default-off `named_accounts` flag. They live in the existing vault and add no keychain items; today's credentials become the `default` account. Covers API-key providers, Claude Plan (token or Claude Code login), and `auth.command` providers. Accounts are chosen with `[provider_accounts]` in config or `-c provider_accounts.<id>=<name>`, and managed with `corbanu account list/add/remove`. A missing or wrong account fails with an error and never falls back to another account. | merged `0fa45b54f0`, S02 `ready` and marked merged-behind-flag so S03 can start |

**Gate evidence** (full details in `qa/provider-auth/pf-84/s01-gate.md` and `s02-gate.md`)
- **Tests:** focused `just test` passes; for S02 that includes runs with and without developer-accounting. S02 also has per-account canary tests and a check that an existing single-account home's vault file is unchanged until the first named account is added.
- **Live runs:** each was a tmux run driven by GLM 5.3 Flash on the real default Z.AI key, all in disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
  - S01: a worker started through the launcher with another home used that home. A fake OpenAI key there got a 401, which proves the right account was used.
  - S02, Z.AI: the named account holding the real key replied `pong`, and the account holding a fake key got a 401.
  - S02, Kimi (second provider): same result.
  - The disposable homes holding real keys were deleted after recording.
- **Videos** (leak scan passed):
  - [S01 home override](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s01-pf84-home-override-84f2797b624b-2026-10-10.mp4)
  - [S01 OpenAI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s01-pf84-home-override-openai-84f2797b624b-2026-10-10.mp4)
  - [S02 Z.AI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s02-pf84-account-isolation-dcbe51a94e33-2026-10-10.mp4)
  - [S02 Kimi](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s02-pf84-account-isolation-kimi-dcbe51a94e33-2026-10-10.mp4)
- **Reviews (Opus 5.5 High, read-only):** S01 had one pass and S02 had three. Each pass asked for changes, and each finding was either fixed or written up with a reason in the gate file. The only one-review-per-sprint overrun is S02's two follow-ups, which checked the security fixes.
- **Linux clippy:** S01 passed on the RTX box. For S02, the RTX box passed clippy (with and without developer-accounting) and a full workspace check at an earlier commit. It then became unreachable over Tailscale before I could rerun the later fixes there. Instead, the PR's own Ubuntu clippy check passed on the final commit, and all checks were green at merge. This is noted in a comment on #409.

**Open items**
- **Not done here:**
  - The independent code-blind acceptance for S01/S02, which this lane doesn't run.
  - Cleaning up what I created on the RTX box, because it is unreachable: `~/corbanu-rtx/pf84-wt` (a worktree of `basewt`), `pf84-target`, `pf84-clippy.sh` and the `pf84-*` logs. Local targets and worktrees are removed.
- **Moved from S02 to S04** (plan and sprint records updated): named ChatGPT/OpenAI sign-ins and AWS profile accounts. Choosing one of these today gives a config error instead of using the default account.
- **Known limits in S02:**
  - A running session only sees accounts added or removed with `corbanu account` after a restart.
  - The TUI's worker pre-check still tests the default account's command. It never passes along an account that isn't set up, and the fix belongs with S03's spawn work.
  - Account names belong to one provider id, so a `zai` account isn't visible to `zai-anthropic`. It does carry over when a model correction moves to that route, because both use the same key.
- **Operator step:** install `scripts/dev/corbanu-launcher.sh` as `~/.local/bin/corbanu` on the Mac so workers get the S01 fix. With S01's change the launcher keeps a `CODEX_HOME` exported by your login shell instead of overriding it, which matches the release wrapper.
- **Next worker:** S03 (account per session, `corbanu exec --account`, spawn `account`, resume). It needs a worktree allocated in a small docs PR first; the S02 dependency is already satisfied.

**For Travis to decide**
- D1–D3 are running on the recommended defaults: one vault per home, no automatic import from extra homes, and spawned agents may only name accounts already set up, shown at spawn and needing approval under Aggressive. Change any of these if you want.
- Whether ChatGPT login and AWS accounts in S04 are worth it, or OpenAI should stay limited to API keys.