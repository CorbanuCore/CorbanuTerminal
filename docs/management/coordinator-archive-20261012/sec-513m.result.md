Both PRs are merged with every check green, and #513, #540 and #544 are closed with evidence. One gap stays open by design and needs your call: a command a person approves to run outside the sandbox can still write the runtime files.

| Issues | PR | Merge commit | Review rounds |
|---|---|---|---|
| #513 (Moderate part), #540 | #558 | `82b1b2cf32` | 4: changes requested twice, then APPROVE twice |
| #544 | #559 | `f4461e366c` | 3: changes requested once, then APPROVE twice |

**#513 (Moderate): only a person asked about the command can let it write a protected file**
- **Person asked now:** allowed for that one command. The approval prompt now names the file. "Protected files" means shell and git start-up files, git hooks and config, and the project's `.codex` and `.agents`.
- **Automatic reviewer:** refused before it is asked.
- **Permission hook, approval cache, stored grant:** refused, with an explanation like #526/#537. Grants never carry such a path; it's dropped and the person is told.
- **Runtime files** (the `tmp` launchers, shell snapshots, skills, plugins, packages, AGENTS.md): still never writable.
- **Aggressive:** same as on main. #520 landed while I worked; I merged it, so `.codex` and `.agents` stay never grantable under Aggressive.
- **Permissive:** unchanged (tested).
- **Pinned test:** the old case is updated, with new cases for the hook, the automatic reviewer and grants.
- **Approval cache:** covered by unit tests only. In a real session, any earlier command output counts as untrusted content, so a repeat command asks the person afresh and never reaches the cache.

**#540: runtime files stay read-only in a person-approved command's sandbox**
- **When a person's approval lifts Moderate's rules:** the runtime files stay read-only in that command's sandbox.
  - Under full access the command now gets a sandbox for this. It can write everywhere except those files.
  - Credential reads and other protected writes are lifted as before.
- **Review closed two more ways out of the sandbox:**
  - **Exec-policy allow rules:** a command allowed by a rule no longer skips the sandbox in these runs. This also applies to zsh-fork subcommands.
  - **Retries:** a network approval's retry stays sandboxed. A "retry without sandbox?" after a denial needs a person asked afresh.

**#544: agent questions**
- **Number keys:** they now only highlight; Enter confirms. Typed text, pastes, held keys and chords never answer.
- **Double Enter blocked:** on the next question, a queued request, the "unanswered questions" confirmation, and the question you land on after going back from it.
- **Freeform answers:** an answer box that opens while you're typing (or over another screen) takes no text until you use the arrows. Tab only submits the way Enter would.
- **Changed for users:** `j`/`k`/`h`/`l` and space no longer move or select. They count as typed text, as in the approval prompts.

**Gate evidence**
- **Tests:** every new behaviour test fails on main or with the fix switched off.
  - **macOS:**
    - `security:: tools::`: 802/802 with the default temp folder; `security::` 306/306 with `/tmp`.
    - Related integration suites: 106/106 with the helper binaries built.
    - TUI `request_user_input` and `bottom_pane` tests: 134/134.
  - **Linux (glitch):**
    - `security:: tools::approvals`: 312/312 default, 307/307 with `/tmp`.
    - The #540 test on real bwrap, with all four cases: pass.
    - TUI `bottom_pane`: 927/927.
- **Clippy `-D warnings`:** clean on macOS and on glitch (core plain and with developer-accounting; TUI).
- **tmux checks:** GLM 5.3 Flash, fresh debug binary, disposable home, dummy files, Moderate. All passed the recorder's leak scan.
  - [A person's approval of a git-hook write works](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-513-issue513-moderate-person-approves-hook-write-0407fb9f1887-2026-10-11.mp4)
  - [The same through a stored grant is refused](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-513-issue513-moderate-grant-hook-write-refused-0407fb9f1887-2026-10-11.mp4)
  - [A runtime-file write is refused even after a person approves it](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-540-issue540-runtime-write-refused-after-approval-cf1085c9e845-2026-10-11.mp4)
  - [#544: a number key only highlights](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-544-issue544-question-number-key-highlights-ba23b303a5c4-2026-10-11.mp4)
- **Not verified:** a real Windows sandbox, which uses the same profile path as every protected run, and a remote exec server.

**Decision for you: unsandboxed runs a person approves.** If a person approves running a command outside the sandbox (an escalation request, or "retry without sandbox?"), that command can still write the runtime files. In one GLM run the model tried exactly this as a retry; the demo now tells it not to retry.
- **Recommendation:** keep it as a documented exception. An unsandboxed command can change anything, including `~/.zshrc`.
- **Alternative:** run these escalations inside a sandbox that allows everything except the runtime files. That closes the gap, but commands that escalate to reach the keychain or Docker would break.
- **Alternative:** refuse escalations whose command text names a runtime file. This is only partial, because the text can hide the path.

**Follow-ups filed (both Low)**
- **#562:** on Linux, missing runtime files such as `$CODEX_HOME/AGENTS.md` or `skills/` can be created by a protected command. This predates these changes.
- **#565:** under Aggressive, a `.codex`/`.agents` write asks the person before it is refused as never grantable.

**Cleanup:**
- **Removed:** both worktrees and local branches, both Mac build folders, and glitch's checkouts and build folder.
- **Kept:** small logs on glitch under `~/corbanu-rtx/sec5-*`.
- **Evidence:** review rounds, logs and demo run folders are in `.codex-work/workers-20261012-sec5/`. The main-branch failure log is `.codex-work/workers-20261002/sec5-int-on-main.log`.