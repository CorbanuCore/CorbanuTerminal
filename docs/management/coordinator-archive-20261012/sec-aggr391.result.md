I've merged #413 (as `9ead7a5755`, all checks green), so `broker_model_auth` is now on by default under Aggressive on macOS, Linux and Windows. Issue #391 closed itself on merge and I added the evidence comment. Linux had no live run and no RTX clippy because the RTX box was offline the whole time; Linux clippy ran in Docker instead.

**What changed**
- **Default:** Permissive and Moderate are unchanged. Other systems stay off because the broker isn't supported there. Aggressive means either Core's level or the level `/security` saves.
- **Settings that still win:** your own config, `-c`, and `--enable`/`--disable` beat the default, and a managed policy outranks those. Two choices I made beyond the spec, from review findings:
  - A project's `.codex/config.toml` can't turn the default off, so a repository can't lower the protection.
  - A level saved in `/security` while Corbanu is running takes effect at the next start. Applying it mid-session would have broken the open session.
- **Fail closed:** if the broker can't start, model requests are refused and nothing is sent directly. The message names `broker_model_auth`, says to choose Permissive in `/security`, and gives the config line that turns it off. When config or a managed policy turned it on, the message says that instead.
- **`/security` view:** the Aggressive review has a new "Model keys" row that also lists what the broker doesn't cover. A broker that didn't start shows as degraded. If config keeps it off under Aggressive, the level shows as partial and a startup warning appears.
- **Docs:** a new "Model key broker" section in `docs/config.md`, the P1 plan row, and `qa/security/issue-391/README.md`.

**Gate**
- **Tests:**
  - macOS: full `codex-core` library suite (2809 passed), including the level × OS × own-setting × project-setting matrix, config loading, a managed-policy pin, the messages, and a real broker that can't start.
  - Two tests that run outside unit-test builds: a separate test binary, and one that runs the real `codex-exec` binary.
  - TUI `/security` tests (116, snapshots updated) and the model-provider tests also pass.
  - The new core tests also pass on the real Windows 11 machine.
- **Clippy `-D warnings`:** clean on the real Windows machine (`C:\CorbanuQA\a391`), on Linux in Docker (Debian, arm64), and on macOS.
- **Review:** Opus 5.5 High via the installed `corbanu exec`, four rounds. Rounds 1–3 asked for changes, all fixed; round 4 was APPROVE WITH NITS, and I fixed those too.
- **Live tmux runs with GLM 5.3 Flash:** on macOS and the real Windows 11 machine, requests went through the broker by default.
  - **Fail closed:** I made the broker fail to start by running Corbanu inside another macOS sandbox, and inside a one-process job object on Windows (`jobrun391.ps1`). Turns failed at once with the new message.
  - **Explicit off:** setting it off by hand under Aggressive sent requests directly.
- **Videos:** four SOP demos, leak-scanned, listed in `qa/demos/index/ISSUE-391.md`:
  - [macOS: brokered by default](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-391-sec391-aggressive-broker-default-295c20b8004f-2026-10-10.mp4)
  - [macOS: fails closed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-391-sec391-aggressive-broker-fails-closed-295c20b8004f-2026-10-10.mp4)
  - [Windows: brokered by default](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-391-sec391-win-aggressive-broker-default-295c20b8004f-2026-10-10.mp4)
  - [Windows: fails closed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-391-sec391-win-aggressive-broker-fails-closed-295c20b8004f-2026-10-10.mp4)

**Open**
- **Early credential uses:** model requests made before the first session starts the broker (e.g. the first model-catalog refresh, `corbanu doctor`) fail with an error that doesn't mention Aggressive. Left as a follow-up.
- **Local model servers:** under Aggressive, providers on plain HTTP, an IPv6 address, or a URL with a query string (e.g. a local Ollama) are refused. This is documented and the message says how to turn the broker off.
- **#412:** I filed a separate issue the review found, not caused by this change. Config loading can't tell when a managed policy pins `shell_snapshot` on, so the output-gate safety check never triggers.
- **Windows TUI tests:** 17 TUI security tests fail on Windows. The same 17 fail on unmodified `origin/main`.
- **My mistake:** one early macOS test run used the agent's own home because `CORBANU_HOME` was still set. It left one session file there. The keychain wasn't touched because keyring isolation was on.

**Cleanup:** I removed `C:\CorbanuQA\a391` and the Z.AI key file on Windows, and left `C:\CorbanuQA\target` and `l398` alone. On the Mac I removed my build target, worktree, local and remote branches, temp files and Docker volume, and quit Docker Desktop, which I had started. The full report is in `.codex-work/workers-20261002/sec-aggr391.result.md`.