# #555 code-blind test design (recorded before human testing)

- **Designer:** a fresh `corbanu exec` session (claude-opus-5-5-plan, high, read-only).
- **Input:** only `packet/`. That is the user intent (`intent.md`) and text captures of the candidate and of the earlier build. It saw no code, tests or implementation notes.
- **Isolation:** instruction-only. `designer-session.md` lists the commands it ran; it read only the packet.
- **Frozen output:** the response is unedited in `original-proposal.md`. `design.json` normalizes it into 33 cases (24 blocker, 9 advisory) and records SHA-256 hashes.
- **Not done:** no separate code-blind executor has run these cases. The evidence below was collected by the implementer. A handoff to human testing must therefore say that this acceptance run is still open.

| Case | Evidence or disposition |
| --- | --- |
| FK-01 `/fork` on the source account | Live: `../fork-after-fork-status.txt` (only `work` has usage). Video `pf84-555-fork-account`. App-server test `fork_of_a_loaded_thread_runs_on_its_account`. |
| FK-02 fork of a resumed conversation | Live: the same run resumes `--last`, then forks (`../fork-after-fork-status.txt`). App-server test `fork_of_a_stored_thread_runs_on_its_recorded_account`. |
| FK-03 edit an earlier prompt | Not run live. It uses the same `thread/fork` (`beforeTurnId`) that the app-server tests cover. **Open.** |
| FK-04 `/side` | Live: `../fork-after-side-status.txt`. |
| FK-05 `corbanu fork <id>` | tmux test `tmux_fork_runs_on_the_recorded_account`. |
| FK-06 explicit `--account` beats the source | App-server test `an_explicit_fork_account_beats_the_source_account`; TUI unit test (fork params carry `--account`). Not run live. |
| FK-07 fork after the source account was removed | Not run. **Open.** |
| FK-08 fork of a fork | Not run (advisory). |
| FK-09 worker inherits the parent's account | Existing test `client_started_worker_inherits_the_parent_account` (unchanged behaviour). |
| RS-01 other provider, credentials present | Live: `../resume-after-cross-*.txt`. tmux test `tmux_resume_on_the_recorded_provider_without_its_onboarding`. Video `pf84-555-cross-provider-resume`. |
| RS-02 other provider, credentials missing | tmux test `tmux_resume_names_a_recorded_provider_without_credentials` (message, exit 1, no onboarding). |
| RS-03 only `default` enrolled for the recorded provider | Not run. **Open.** |
| RS-04 picker matches resume by id | Live run uses the picker; the tmux test uses the id. |
| RS-05 resume twice | Not run (advisory). |
| GH-01 `ghost` in `[provider_accounts]`, recorded account exists | Live: `../resume-after-ghost-2-status.txt`. tmux test `tmux_resume_ignores_a_missing_provider_accounts_entry`. Video `pf84-555-provider-accounts-ghost`. |
| GH-02 recorded account also missing | The same tmux test (names `gone`, not `ghost`, exit 1). |
| GH-03 new session with `ghost` | Unchanged #426 refusal. Not run here. **Open.** |
| MSG-01 in-TUI wording | Live: `../resume-after-missing-1-resume.txt`. tmux test `tmux_in_app_resume_on_a_named_account`. Video `pf84-555-in-app-resume-providers`. |
| MSG-02 recover through `/providers` without restarting | Not run (advisory). |
| MSG-03 command-line wording | tmux tests (`corbanu account add` / `--account` text, exit 1). |
| MSG-04 session after a failed in-app resume | Live capture: the session stays usable. Partial. |
| ESC-01 Esc in the `corbanu resume` picker | Live: `../resume-after-esc-*.txt`. tmux test `tmux_esc_in_the_resume_picker_does_not_open_onboarding`. Video `pf84-555-resume-picker-esc`. |
| ESC-02 Ctrl+C in that picker | Not run (advisory; unchanged). |
| ESC-03 Esc in `/resume` inside a session | Live: `../resume-after-inapp-1-esc.txt`; tmux test `tmux_in_app_resume_on_a_named_account`. |
| ESC-04 normal onboarding not broken | Existing onboarding tmux suites in CI. Not run here. **Open.** |
| PR-01 full precedence order | Unit tests: `resume_launch::tests`, and the core config-loader precedence tests (existing). Not run live. |
| PR-02 `--account` for another provider | Unit test `an_explicit_account_is_not_the_recorded_account`. |
| REG-01 `named_accounts` off | Not run live. The changed paths check the feature first. **Open.** |
| REG-02 no secrets shown | The captures and the diff are leak-scanned for the real key (0 hits). The demo tool's scan passed for every video. |
| REG-03 footer vs `/status` | "via zai default" in the footer is the reasoning effort, not the account (observation). |
| REG-04 usage totals add up | Not run (advisory). |
| REG-05 "Account:" line | The captures show it unchanged. |
| REG-06 both platforms | macOS: live runs and tmux tests. Linux: unit tests on glitch; tmux suites in CI Ubuntu tmux smoke. |
