Both PRs are merged, and #300, #301 and #304 are closed with evidence comments. Every gate step passed. The one exception: two existing end-to-end probes that launch commands under the elevated sandbox fail on the QA machine, on `main` too, so they're covered by CI instead.

**A. #300: merged as PR #331 (`62f997f3`)**
- **What changed:** on the tool path, a command under the unelevated Windows sandbox is refused up front if its profile blocks reading any file (exact paths or globs, whether or not anything matches yet) or limits which folders can be read. The message says it wasn't run because this Windows sandbox mode can't block reading the protected files. It then names three ways to switch: `/setup-default-sandbox`, `[windows] sandbox = "elevated"`, or `-c windows.sandbox=elevated`. `process_exec_tool_call` now shows the same text. Elevated, macOS and Linux are unchanged. `docs/sandbox.md` is updated.
- **What `main` actually did:** this wasn't a leak. On the real machine, the unelevated backend already refused these commands when starting them, with an unclear error ("Restricted read-only access requires the elevated Windows sandbox backend"). The fix makes the refusal earlier and actionable.
- **Gate:**
  - **Real machine, elevated and normal session:** both new tests fail with the fix reverted and pass with it.
  - **CI:** passes in both sessions.
  - **Clippy:** Windows and Linux clean.
  - **Opus review:** APPROVE; I fixed its low findings.

**B. #301/#304: merged as PR #326 (`6b94db40`), option 1**
- **What changed:** the configured deny-read rules now travel with their expanded paths through the sandboxing crate, core exec, the backends, the wrapper and the elevated setup payload. Each deny entry records which rules created it, and is removed only once none of those rules is configured. #326's safe parts are kept: exact removal, the ownership record in `.sandbox-secrets`, no link following, no removal while a contract is armed.
- **Setup refreshes:** refreshes that carry no rules (read-root refresh, first setup) now add and remove nothing. Before, they passed an empty list, which would have removed every entry once removal worked.
- **S1 test** (a folder held open hides a secret from the scan):
  - **Old path-driven code:** fails on the real machine and on CI, in both sessions; the deny is lost.
  - **This branch:** passes everywhere.
- **Also fixed:** since round 4, CI's elevated step had been failing on a test-only quirk (Windows adds a harmless flag to the folder's permission list). The tests now compare the permission entries only.
- **Gate:**
  - **Real machine:** all 13 `sec_win_` tests pass in both sessions.
  - **Clippy:** Windows (4 crates) and Linux clean.
  - **Opus round 5:** APPROVE; I fixed its low findings.
  - **CI:** all checks green.

**Still open**
- **#323** (sessions on different `CODEX_HOME`s, or with different rules, removing each other's entries) stays open. This fix doesn't make it worse.
- **QA machine:**
  - It was offline for about 3 hours, which slowed the gate.
  - Since it came back, commands under the elevated sandbox time out on startup (connecting to the runner). That happens on `main` too, so I relied on CI's Windows runner for the two existing probes that exercise it.
  - Problems that were already there: five `unified_exec` tests fail on that machine, and one core symlink test needs a privilege a normal session doesn't have. Two macOS seatbelt tests fail locally on the Mac.
- **Cleanup:**
  - **Removed:** `C:\CorbanuQA\win7` and `win5`, the RTX clone and logs, both Mac worktrees and their build folders.
  - **Kept:** `C:\CorbanuQA\target`.
  - I created no Windows users or scheduled tasks. The `CorbanuQA-s08-*` tasks on that machine belong to another worker.

The full report is in `sec-win7.result.md`, and the reviews are in `sec-win7-review-300/` and `sec-win7-review-30x/` in the workers folder.