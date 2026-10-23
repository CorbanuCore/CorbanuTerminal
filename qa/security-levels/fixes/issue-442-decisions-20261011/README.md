# #442: Travis's decisions of 2026-10-11 (Aggressive)

1. `.env` files **created after** an Aggressive session starts are protected too, on every platform.
2. The project's `.codex` and `.agents` are **always read-only** under Aggressive; "Grant once" no longer lifts them.

(#434 and #440: no change; recorded on the issues.)

## 1. Credential files by name

The Aggressive profile now denies the names `.env`, `.env.*` and `.envrc` (the launch preflight's workspace names,
generalised) by glob, in addition to the explicit per-path denials of the files the preflight found. A folder with such
a name (a `.env` virtualenv) is not a credential file; a link with such a name is refused (For Travis 5).

| Platform | Mechanism | Covers | Limits |
| --- | --- | --- | --- |
| macOS (Seatbelt) | Regex rules from the globs `/**/<name>` and `<workspace>/**/<name>`: no read, and no create, link, rename-to, write or unlink (folders aside) | Any such file anywhere, created at any time by anyone. Agent commands cannot create one, so they cannot plant a link or pipe the person's file would be written through. Case aliases (`.ENV`) are denied too | Agent commands cannot create `.env` files at all (also `.env.example`, see below). A credential name that is not a plain file (a link, extra hard links, a pipe) made before the session would let the person's write land elsewhere: the launch check refuses Aggressive for such entries in the current folder only (not elsewhere on disk) |
| Linux (bubblewrap) | 1. At every command start, every match below the workspace is masked (glob expansion, plus every letter case from the watcher's own walk). 2. A credential name that is not a plain file (any link, a pipe, socket or device, a file with other hard links) refuses the command until it is removed. 3. From before that scan until the command ends, inotify watches every folder below the workspace; a non-folder with such a name created, linked or moved in by anyone stops the command (SIGKILL) | Files created between commands (by the agent or the person) and while a command runs | Bubblewrap can only mask existing paths and seccomp/Landlock cannot match names, so a file created during a command is not hidden from it; the command is stopped instead (fail closed). A process already waiting for the file can read it in the moment before the stop. Only the workspace is covered, not `.env` files elsewhere. A folder reached through a link inside the workspace is not watched. A folder below the workspace replaced by a link to outside it is not followed. Refused: folders on NFS/SMB/CIFS/9p/FUSE/NCP/Ceph/Lustre/AFS/Coda/OCFS2/GFS2/vboxsf (inotify misses changes made elsewhere), more than 100,000 folders, watch-setup failure. Lost events trigger a full rescan. The file is left in place (its creator is unknown); a link planted by a stopped command stays too, and a write through it lands in its target (the refusal names that target). One watcher per command: many concurrent commands on a huge tree can hit `max_user_watches` (refused, fail closed) |
| Windows | Unchanged: globs are expanded to deny ACEs on existing files at each command start | Files that exist when a command starts | Deny ACEs are per object and ACLs cannot match names, so a file created **during** a command inherits the folder's ACL and is readable to that command. Not fixed here (no Windows host to build and test a watcher). The launch check refuses links and pipes, but cannot see extra hard links on Windows |

The launch check refuses Aggressive when the profile lacks these rules (or the current folder's path contains glob
characters, which the rules cannot express), and the denial explanation (#464) covers such files. The /security review
and inspector say ".env, .env.* and .envrc files, also ones created later, on Windows from the next command on" instead
of ".env files found at start".

## 2. `.codex` and `.agents`

Core's protected-path rules keep `<workspace>/.codex` and `.agents` read-only with a grant, like the Corbanu home's
runtime entries. Grant review, inspector ("Agent settings" row) and the /security Sandbox row say so. Only the
workspace's own `.codex`/`.agents` are covered, not ones in subfolders.

Finding: in the real Aggressive profile, `.git`, `.codex` and `.agents` were already read-only through the sandbox's
workspace-metadata protection, which no grant lifts. So on main a confirmed grant did not make `.codex` writable (tmux
check below), and it does **not** make `.git/hooks` or `.git/config` writable either. Only Core's rules (exercised by
the core tests with a plain workspace-write profile) lifted them. The grant review now says `.git` stays read-only.

## For Travis

1. **`.env.*` breadth.** `.env.example`, `.env.sample` and `.env.template` are usually committed templates. Under
   Aggressive they are now unreadable, agent commands cannot create them (macOS) or are stopped when one appears
   (Linux), so `git checkout`, `npm install` or a scaffold that writes one fails. Recommendation: exempt
   `.env.example`, `.env.sample`, `.env.template` and `.env.dist`.
2. **Linux race.** Accept the stop-on-create design (a waiting process can win a race of milliseconds), or fund a
   seccomp user-notify supervisor that opens files on the command's behalf (safe, but a large change).
3. **Windows.** A credential file created during a running command stays readable to that command. Options: a
   directory-change watcher that stops the command (like Linux), or accept and keep the wording.
4. **Grant once under the real profile** lifts almost nothing: git hooks and config live in `.git`, which the profile
   keeps read-only, and the shell start-up files are outside the workspace. Decide whether Grant once should lift
   `.git/hooks` and `.git/config` (weaker) or be described as such.

5. **Credential-file links are refused.** A `.env` that is a link (for example `.env -> ../shared/.env`), has extra
   hard links (Unix only) or is a pipe makes Aggressive refuse to start (current folder, all platforms) and, on Linux, refuses every
   command while it exists. A link can be re-pointed after it is checked, so the person's write could land in a readable
   file. Fail closed as built; the alternative is to allow links whose whole path has no other links, checked at each
   command (weaker against a re-point mid-command).

## Evidence

- Regression tests, failing before:
  - macOS, real `sandbox-exec`: `sandboxing seatbelt::tests::credential_names_cannot_be_read_created_or_planted` (read, case
    alias, overwrite, append, create, hard link, symlink, fifo, rename to, rename away; a `.env` folder works) and
    `tui security::aggressive::tests::issue_442_credential_files_created_later_stay_unreadable_under_seatbelt` (agent
    cannot create; the person's `.env.local`, `app/.env`, `.envrc`, `.env.production`, `../other-project/.env` are
    unreadable to the next command, also via copy, hard link, symlink or rename; Permissive still reads). Before: the
    synthetic values were printed.
  - `issue_442_launch_check_requires_credential_names` and `issue_442_launch_check_names_unsafe_credential_files`
    (launch check); protocol `credential_file_names_globs_and_roots`, `credential_entry_problems`.
  - Linux, real bwrap on glitch (`linux-sandbox` `issue_442_*`, 4 tests): created by a sandboxed command; by the person
    between commands (any letter case); while a command runs; a planted link (refused while dangling and after the
    person wrote through it). Before (watcher off, first version of these tests): created-by-command, through-a-link and
    while-running all failed, the last two leaked the value. Unit tests in `credential_watch_tests.rs` (stops, moved-in folders, unsafe
    entries refused (links of any kind, hard links, pipes), any-case existing files hidden, `.env` folders and temporary
    build folders not stopped).
  - Core: `pf_25_s01::issue_442_a_confirmed_grant_never_lifts_codex_or_agents` (before: `codex=0 agents=0`),
    `read_denials::issue_442_a_grant_keeps_codex_and_agents_read_only`,
    `pf_23_s01::issue_442_env_file_matched_by_name_is_denied_and_explained`,
    `pf_23_s01::issue_442_permissive_is_unchanged`.
- tmux functional check (GLM 5.3 Flash, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, Aggressive after restart),
  [scripts](scripts), [captures](captures):
  - before (main `2e2e2f8d82`): the agent's `.env.local` and the person's `.env.production` were printed.
  - after: the agent cannot create `.env.local`; the person's `.env.production` is denied with the Aggressive
    explanation; `.codex` write denied after Grant once; review text updated.
- Videos (GLM 5.3 Flash, at `04fa31adcd`):
  [.env created mid-session](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-new-env-file-denied-04fa31adcd7f-2026-10-11.mp4),
  [.codex read-only after Grant once](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-codex-always-read-only-04fa31adcd7f-2026-10-11.mp4).
- Review: independent Opus 5.5 High (read-only `corbanu exec`), four rounds; rounds 1-3 requested changes (all fixed,
  see the PR), round 4 approved; its minor findings are fixed too.
