I added the notice and merged it as PR #394 (merge commit `e2d988e9f8`), with all 29 checks passing and none pending.

**What changed**
- **When the notice shows:** the notice is "Using OPENAI_API_KEY from your environment — billed per use to that API key. Sign in or unset it to change."
  - It appears the first time a thread is billed to `OPENAI_API_KEY`, and again if the credential switches back to it.
  - It shows once, not on every turn.
  - `/review` and manual compaction show it on the thread that started them.
  - A spawned agent shows it in its own thread, because it may use a different provider from its parent.
  - It never shows for a saved sign-in or key, `CODEX_API_KEY`, a session token, or a provider that doesn't use OpenAI sign-in.
- **TUI:** the notice appears as a warning in the conversation. `/status` shows "OPENAI_API_KEY from your environment (billed per use)", but only when the current provider uses OpenAI sign-in. `/providers` is unchanged ("Environment-backed credential").
- **exec:** one `warning:` line on stderr. With `--json` it is one warning item, using the existing warning format, so every output line is still valid JSON.
- **app-server:** clients get the notice through the existing per-thread `warning` notification. I also added two optional fields so `/status` can name the variable: `envVar` in `account/read` and `apiKeyEnvVar` in `account/updated`. Both are `null` for a saved key. The schema files, app-server README and `docs/authentication.md` are updated.

**Gate evidence**
- **Tests:** new tests cover the notice, the exec stderr line, the app-server account fields, and two TUI snapshot tests (the notice and the `/status` line).
  - On Linux (the RTX box), the focused tests passed: 908/908 without `developer-accounting` and 583/583 with it, including the TUI tmux onboarding and re-authentication suites.
  - On the Mac they also pass with and without the feature, apart from three tmux tests that fail on long socket paths under `/Volumes` and pass on Linux.
- **Clippy:** Linux `-D warnings` is clean for every crate I touched, with and without `developer-accounting`, on the final code.
- **Review:** Opus 5.5 High (read-only, installed `corbanu exec`) asked for changes twice and approved on round 3.
  - **Round 1:** `/status` could name the variable on non-OpenAI providers; `/review` repeated the notice; manual compaction gave no notice. All fixed.
  - **Round 2:** my fix silenced spawned agents too, so a child on OpenAI under a non-OpenAI parent would be billed without a notice. Fixed.
- **tmux check** (fresh debug build, disposable home, real key from the vault, gpt-5.4):
  - With only `OPENAI_API_KEY`: the notice showed on turn 1, wasn't repeated on turn 2, and `/status` named the source.
  - With a saved key and `OPENAI_API_KEY` also set: no notice, and `/status` showed the usual "OpenAI API key configured" line.
  - `corbanu exec`: exactly one stderr notice.
  - I repeated the saved-key check on the final code. The screen captures and exec output are in `/tmp/oaenv-func` and were scanned clean.
- **Video:** [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/openai-env-notice-openai-env-key-notice-efd17427ad07-2026-10-09.mp4), re-recorded on the final code and scanned for leaks before publishing. The spec and index row are in `qa/demos/`.

**Still open**
- **Known logging leak (#380):** the video tool records with full trace logging, which wrote the key into a private log. The tool blanked it automatically; the bug itself isn't fixed.
- **Spawned agents:**
  - Each one shows the notice once in its own thread, so a large agent tree could repeat it; the reviewer suggested this could be trimmed later.
  - `corbanu exec` only prints notices from its main thread.
- **No notice in realtime voice sessions.** This is documented.
- **`/logout` closes the TUI,** so "right after logout" in practice means the next launch, which shows the notice. The mid-session switch is covered in core but only unit-tested.
- **The two new account fields behave differently:** `account/read` only names the variable for providers that use OpenAI sign-in; `account/updated` always describes the OpenAI credential. This is documented, and the TUI checks the provider before showing it.
- **The Python SDK's generated models weren't regenerated.** That matches recent practice in the repo, and no check enforces it.

**Cleanup:** the Mac build folder and worktree, the RTX worktree, its build folder and my scripts and logs there, and every disposable home are removed. The local branch `fix/openai-env-key-notice` wasn't deleted because the local `main` is behind; it's harmless. The review files are in `.codex-work/workers-20261002/oaenv-review/`.