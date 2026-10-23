# #555: forks and side conversations follow the conversation's named account

Disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, `named_accounts` on, GLM 5.3 Flash (`zai`), no default Z.AI key,
`[provider_accounts] zai = "personal"`. Accounts `work` and `personal` hold the same vault key, which was used only to enrol them.
Script: a session on `--account work` answers `ping`; `corbanu resume --last` (recorded on `work`); `/fork`; the fork answers
`forked`; `/status`; `/side reply with the single word side`; `/status`.
Before: `e7be21a1a9` (main). After: this branch.

| Case | Before | After |
| --- | --- | --- |
| `/fork` of a conversation on `work` | `/status` says `account work`, but the fork's tokens are on `personal` (`fork-before-fork-status.txt`) | the fork runs on `work`; `/status` says `account work` and only `work` has usage (`fork-after-fork-status.txt`) |
| `/side` from that fork | side tokens also on `personal` (`fork-before-side-status.txt`) | side conversation on `work` (`fork-after-side-status.txt`) |

Regression tests:
- `codex-app-server` `suite::v2::provider_account_inheritance`: a fork of a loaded thread and of a stored (not loaded) thread runs
  on the source thread's account (both fail on main: they ran on `[provider_accounts]`); an explicit fork account wins.
- `codex-tui` `suite::resume_recorded_account::tmux_fork_runs_on_the_recorded_account`: `corbanu fork <id>` shows `account work`
  (fails on main's binary: `account personal`).

## Resume: other provider, missing `[provider_accounts]` account, Esc, in-app `/resume`

Same disposable-home setup; each conversation first answers `ping` on `--account work` (on Z.AI), then is resumed.
Before: `e7be21a1a9` (main). After: this branch at `32a78c5b46` (after review round 1; re-run of every case).

| Case | Before | After |
| --- | --- | --- |
| Home configured for OpenAI (no credentials); `corbanu resume` → the Z.AI conversation | picker, then OpenAI's provider onboarding (`resume-before-cross-1-picker.txt`, `resume-before-cross-2-resumed.txt`) | resumes on Z.AI; `pong` answered; `/status` says `account work` (`resume-after-cross-2-resumed.txt`, `resume-after-cross-3-status.txt`) |
| `[provider_accounts] zai = "ghost"` (missing); `corbanu resume --last` | refused: `account ghost … is not configured` (`resume-before-ghost-1-resume.txt`) | resumes on `work`; `/status` says `account work` (`resume-after-ghost-2-status.txt`) |
| No default key; `corbanu resume`, Esc in the picker | API-key onboarding (`resume-before-esc-2-after-esc.txt`) | picker footer says `esc quit`; Esc exits with status 0 (`resume-after-esc-1-picker.txt`, `resume-after-esc-2-after-esc.txt`) |
| Session on `personal` (`[provider_accounts]`); `/resume`, Esc, then `/resume` → the `work` conversation | — | Esc returns to the session; the resumed conversation answers `pong` on `work`; `/status` says `account work` (`resume-after-inapp-1-esc.txt`, `resume-after-inapp-2-status.txt`) |
| `work` removed; `/resume` → that conversation | error says `corbanu account add` / `--account` (`resume-before-missing-1-resume.txt`) | error says "add it or pick another account in `/providers`"; stays on the current conversation (`resume-after-missing-1-resume.txt`) |

Regression tests: `codex-tui` `suite::resume_provider_accounts` (tmux, seeded rollouts): all five fail on main's binary and pass
after (the in-app test fails on main only at the `/providers` wording). Unit tests: `resume_launch::tests`,
`codex-model-provider` `tui_recovery_points_to_providers`.
