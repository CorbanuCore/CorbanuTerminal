I've finished S03 and allocated S04, but I'm stopping before implementing S04 because my context is getting large. S03 is merged and set to `ready` (not archived). S04 is allocated and marked in progress, but has no code yet, and S05 is still a draft.

**What shipped**

| PR | What | State |
|---|---|---|
| #411 | Allocates S03's worktree, branch, base and write scope | merged |
| #421 | **S03**, behind the `named_accounts` flag (details below) | merged `0558c52cc3`; S03 `ready` |
| #423 | Allocates S04 (`worktrees/pf84-s04-providers`, branch `feat/pf-84-s04-provider-accounts-ui`, base `0558c52cc3`) and records the ChatGPT/AWS question for you | merged `da0630b460` |

S03 changes:
- **Choosing an account:** `--account [<provider>:]<name>` now works on `corbanu exec`, the TUI and `corbanu resume`. The order of precedence is `--account` or the spawn `account`, then the account recorded in the thread, then `[provider_accounts]`, then `default`.
- **Never falls back:** any session, resume or spawned agent on a missing account is refused with recovery text and is never run on another account. A named `--account` with the flag off is an error.
- **Resume:** it reuses the account the thread recorded. The rollout stores only the account name.
- **Spawned agents:** the spawn tool takes an `account` argument, limited to accounts already set up for that provider. Spawned agents inherit the parent's account unless one is given, including role children and workers the TUI starts.
- **D3:** under Aggressive (the level actually in force), switching account needs your approval, and is refused when approvals are off.
- **Known limit fixed:** the TUI worker pre-check now tests the worker's own account, not the default one.

**Gate evidence** (`qa/provider-auth/pf-84/s03-gate.md`)
- **Tests:** focused `just test` passes on the final tree, including runs with and without developer-accounting. Clippy is clean on macOS.
- **Linux clippy:** the RTX box was still offline (ssh timed out), so I relied on the PR's Ubuntu clippy job. Its first run caught one missing field in `thread-manager-sample`, which I fixed, and it then passed. All 28 checks were green at merge.
- **Live runs:** all driven by GLM 5.3 Flash in tmux, with the coordinator on the real default Z.AI key:
  - Workers through the checked-in launcher: `--account main` (real key) replies `pong`, `--account fake` gets a 401, resuming that session stays on `fake` (401), resuming with `--account main` replies `pong`, and `--account gone` is refused as not configured.
  - The same five results on Kimi as the second provider.
  - A spawned agent on `fake` gets a 401 and the coordinator still replies `pong`.
  - `corbanu --account fake` in the TUI shows a visible 401.
  - The run folders holding real keys were deleted after recording.
- **Videos** (leak scan passed): [exec](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-exec-account-047d9e10392d-2026-10-10.mp4), [exec Kimi](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-exec-account-kimi-047d9e10392d-2026-10-10.mp4), [spawn](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-spawn-account-047d9e10392d-2026-10-10.mp4), [TUI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-tui-account-047d9e10392d-2026-10-10.mp4).
- **Opus 5.5 High review:** the first pass asked for 4 blocking fixes, all made:
  - role children kept the parent's account;
  - D3 now checks the level actually in force;
  - `--account` is stored with its provider;
  - TUI-started workers inherit the parent's account.

  A scoped second pass approved the fixes. It is the one review over budget, which I authorized as integrator.

**Open items**
- **Known S03 limits:**
  - The TUI's spawn line and `/status` don't show the account yet (S04 and S05 own those).
  - A resumed v1 child runs on its parent's account rather than its recorded one. This never widens access.
  - Fork doesn't carry the account.
  - Workers never see the coordinator's environment key, so a worker on the default account has no key unless the home stores one.
- **Follow-up tests** the second review asked for: a level raised mid-session, TUI-started worker inheritance, and the running-thread mismatch check.
- **RTX box cleanup** of `~/corbanu-rtx/pf84-wt`, `pf84-target`, `pf84-clippy.sh` and the `pf84-*` logs is still not done because the box is offline.
- **Another worker's worktree** `pf84-accept-fixes-20261010` (branch `fix/pf-84-acceptance-bugs`) is active. The S04 worker should coordinate with it.
- **Next worker:** S04 in its allocated worktree. I've already traced the duplicate-row bug: once Claude Plan is set up, onboarding (`tui/src/onboarding/auth.rs`) shows both the setup row and a "use existing provider" row for the same entry, and configured API-key providers look affected the same way. S05 comes after S04.
- **Code-blind acceptance** for S03 hasn't been run; that's a separate acceptance step, not the implementer's.
- My worktrees, branches and build target are removed.

**For Travis to decide**
1. **Named ChatGPT sign-ins and AWS profile accounts** (both refuse with an error today). I recommend splitting them into a new PF-84-S06 after S05 rather than putting them in S04.
   - AWS is small: the profile name isn't secret, and Bedrock reads one `aws.profile`. But I can't prove it live without AWS credentials.
   - ChatGPT is much bigger: each account needs its own sign-in store in the vault, refreshed tokens saved back, and a login flow aimed at that account. Today there is one `Codex Auth` keychain item or `auth.json` per home.
   - Options: (a) S06 with AWS first and ChatGPT after (recommended); (b) AWS only, with OpenAI staying API-key only; (c) drop both.
2. **D1–D3** are still on the recommended defaults. D3 is now implemented as described above.