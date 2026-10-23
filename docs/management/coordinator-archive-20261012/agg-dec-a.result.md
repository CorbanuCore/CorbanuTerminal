PR #520 is merged into main (`6ebfa1f47c`). It ships both remaining decisions, with every check green when it merged. All four decisions are recorded on #434, #440 and #442, and there's a summary comment on #442.

## What shipped
**Decision 3: `.env` files created after an Aggressive session starts are protected too.** The Aggressive sandbox now blocks `.env`, `.env.*` and `.envrc` by name, in any letter case. The explicit denials of the files found at start are still there.
- **macOS:** agent commands can't read, create, link, rename to or overwrite such a file anywhere. Folders with these names (a `.env` virtualenv) still work.
- **Linux:** full parity isn't possible, so it fails closed:
  - every command hides the matching files that exist when it starts;
  - a `.env` that is a link, a pipe or has extra hard links refuses every command until it's removed;
  - a file with such a name that appears while a command runs stops that command.
- **Linux limits:**
  - a process already waiting for the file can read it in the moment before the stop;
  - only the workspace is covered;
  - folders on network or FUSE filesystems refuse the command.
- **Windows:** not fixed, only documented. A file created while a command runs is readable to that command; the next command can't read it.
- The launch check, the denial explanation and the /security and inspector wording now cover new files. ".env files found at start" is replaced by ".env, .env.* and .envrc files, also ones created later, on Windows from the next command on".

**Decision 4: the project's `.codex` and `.agents` are always read-only under Aggressive.** "Grant once" no longer lifts them; the grant review, inspector and /security say so.

**Finding:** the real Aggressive sandbox already kept `.git`, `.codex` and `.agents` read-only whatever the grant. So on main, Grant once also never made `.git/hooks` or `.git/config` writable. The tmux check showed this before and after. The grant review now says so.

## Gate evidence
- **Regression tests:** they fail before and pass after:
  - real `sandbox-exec` runs on macOS;
  - real bwrap runs on glitch, for a file created by an agent command, by the person between commands, during a running command, and through a planted link;
  - the launch check;
  - core grant tests (before: `codex=0 agents=0`).
- **Permissive:** a test shows it unchanged.
- **`just test`:** green with and without developer-accounting, apart from three tests that also fail on main in this local environment (I checked on a main worktree).
- **Clippy:** `-D warnings` is clean on macOS and on glitch, for both feature sets.
- **tmux check:** GLM 5.3 Flash, disposable home, Aggressive, before and after.
  - Before (main): `.env` files created mid-session by the agent and by the person were printed.
  - After: the agent can't create one, the person's is denied with the explanation, and `.codex` is denied after Grant once.
- **Review:** Opus 5.5 High, read-only, five rounds. Rounds 1–3 asked for changes, all fixed: planted links and pipes, letter-case aliases, false stops on temporary folders, network filesystems, a musl build break, links that could be re-pointed. Rounds 4 and 5 approved.
- **Videos:**
  - [.env created mid-session](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-new-env-file-denied-04fa31adcd7f-2026-10-11.mp4)
  - [.codex after Grant once](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-codex-always-read-only-04fa31adcd7f-2026-10-11.mp4)
- **Overlaps:** I merged origin/main six times, picking up #499 and the bwrap, #245, #513, #521, #522 and #531 changes.
  - #513 had a test that expected Grant once to lift `.agents`; I changed it to the new rule. Its runtime-only scope for Moderate is unchanged.
  - The longer Sandbox row pushes "Nested agents: refuse" below a 36-row screen. I changed four demo specs to wait for the review title instead; `nested-launch-*`, `registry-read-only` and `standalone-nested-refused` may also need it.
- **CI:** one run of the new core security job failed on an unrelated grant-revocation test; #541 has since fixed that race.
- **Cleanup:** the worktrees, the local build target (142 GB) and the glitch build directories (68 GB) are removed. Evidence is in `.codex-work/targets/agg-dec-env-evidence` (3 MB), and a report is in `workers-20261002/agg-dec-env.result.md`.
- The Anthropic content filter did not interrupt this work.

## For Travis to decide
1. **`.env.*` breadth:** `.env.example`, `.env.sample` and `.env.template` are now unreadable, and agent commands can't create them. So a `git checkout` or `npm install` that writes one fails under Aggressive. I recommend exempting those template names.
2. **Linux race:** accept stopping the command when such a file appears, or fund a much larger supervisor that is safe against the race.
3. **Windows:** a file created during a command stays readable to it. Add a watcher that stops the command, as on Linux, or accept it.
4. **Grant once:** under the real Aggressive sandbox it lifts almost nothing. Should it lift `.git/hooks` and `.git/config`, or be described as lifting nothing?
5. **`.env` links:** a `.env` that is a link (e.g. `.env -> ../shared/.env`) makes Aggressive refuse to start. On Linux it also refuses every command while it exists. Keep this, or allow some links?