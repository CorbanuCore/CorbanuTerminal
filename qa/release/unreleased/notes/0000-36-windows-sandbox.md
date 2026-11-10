## Windows sandbox

- **The default (elevated) sandbox's commands can't break away from their
  job** (#1639, D256, #1507). Cancelling or a timeout now ends every process
  they start in their job; a program that asks to break away gets "access
  denied" unless it retries without the flag. The non-admin sandbox already
  worked this way.
- **Default (elevated) sandbox: each command gets its own identity** (D324,
  #1717, #1648), as non-admin sandbox commands already do. Commands running
  at the same time, for example in different projects, can no longer write
  to each other's processes or the helper processes that start them, or
  start programs under them. Projects are still not fully kept apart: a
  command can still borrow another command's access and write in its
  folders (#1560), or, by racing another command's start-up, run code
  outside its own sandbox restrictions (#1721), and can still read and end
  the other's processes. So still avoid running commands
  in several projects at once (see `docs/sandbox.md`).
- **Commands in the default sandbox can no longer read a non-admin sandbox
  command's memory** (D429, #1803). A default (elevated) sandbox command
  could read the memory of a non-admin sandbox command running at the same
  time, such as secrets in its environment, its output and what was typed at
  its prompts. Other accounts (except SYSTEM and administrators using debug
  rights) can now only wait for a non-admin command's processes. One
  non-admin command can still read another's memory (#1838), so still avoid
  running commands in several projects at once.
- **Profiles the Windows sandbox can't enforce are refused, and say why**
  (D331, #1762). A profile with a writable folder inside a read-only one is
  now refused for the agent's commands in the non-admin sandbox too, as it
  already was in the default sandbox and for app-server `command/exec`; in
  our measurements this changes the outcome only under Permissive. On both
  sandboxes the message now names both folders, notes that the protected
  `.git`, `.codex` and `.agents` folders count as read-only, and says to
  change the profile so that no writable folder lies in a read-only one. In
  the non-admin sandbox, `command/exec` (and the exec server) now refuse a
  profile that blocks or limits reading files with a clear message that
  says how to switch to the default sandbox, instead of an unclear error
  from the sandbox.
- **Non-admin sandbox: a working folder reached through a junction no
  longer bypasses a read-only folder** (#1780, #1792). With the working
  folder given as a junction, a read-only folder given by its real path was
  writable, and so was a read-only folder spelled through a junction inside
  a working folder given by its real path. Read-only paths are now matched
  to writable folders by their real (link-resolved) path too. When one
  profile marks the same folder read-only under one spelling and writable
  under another, read-only wins.
- **A `\\localhost` network working folder gets a clear error** (#1863).
  `command/exec` refused a working folder given as a `\\localhost\...`
  network (UNC) path with an unclear "not valid on this host" error (debug
  builds crashed). It still refuses it, in both sandboxes, now saying to use
  the folder's local path instead. Other network paths are unchanged: the
  command starts sandboxed as before.
- **A writable folder inside a read-only one is refused through links too**
  (#1801, D331). D331's refusal compared folders by spelling, so a
  writable folder that lay inside a read-only one only through a junction
  or folder symbolic link (on either folder, or on the working folder)
  wasn't refused, and the non-admin sandbox let commands write there. It
  now compares real paths too (`subst` drives included), for the agent's
  commands in both sandboxes and for `command/exec`. The same folder
  marked read-only and writable under two spellings is still not refused;
  in the non-admin sandbox read-only wins (#1780). Newly refused for the
  same reason: a protected `.git`, `.codex` or `.agents` folder that is a
  link to a folder holding another writable folder of the profile. Replace
  the link with a folder; see "A writable folder inside a read-only one" in
  `docs/sandbox.md`.
- **Non-admin sandbox: `command/exec` runs from a `subst` drive, an 8.3
  short name or a differently cased path** (#1766, D398).
  It (and the exec server) refused such a working folder with "cannot enforce split writable root
  sets directly", because it compared the folders by spelling. It now
  compares their real paths, so these commands run as they would from the
  real path, with the same read-only folders. A working folder that leads
  out of the workspace is still refused. The agent's own commands and the
  default (elevated) sandbox weren't affected.
- **Non-admin sandbox: `command/exec` refuses a working folder on a network
  with a clear message** (D446). A mapped network drive or a UNC path
  (`\\server\share\...`), as the command's or the sandbox's working folder,
  is refused with "... is a network folder (a UNC path or a mapped network
  drive)", for any sandboxed permission profile, read-only included. In
  #1866's measurement the non-admin sandbox's commands couldn't write
  through a network path, so such a command ran as if the folder were
  read-only. Use a folder on a local drive. `\\localhost` paths were
  already refused (#1863). The agent's own commands and file tools, the
  exec server and the default (elevated) sandbox are unchanged.
- **Non-admin sandbox: a writable folder inside a protected `.git`, `.codex`
  or `.agents` folder is refused for older workspace-write profiles too**
  (#1810, D331). A profile in the older workspace-write format, for example
  with a writable root at the workspace's `.git\hooks`, skipped D331's
  check, so the agent's commands and `command/exec` ran and could write
  there. Such profiles are now refused, as the default sandbox already
  refused them. Naming `.git` itself writable is still allowed.
- **Non-admin sandbox: nested `.codex` and `.agents` folders are watched**
  (#1642, D283, #1541). One that appears or is replaced deeper in your
  workspace while a command runs isn't loaded until you confirm it, like one
  at the top. Commands can't rename, move or delete existing ones or the
  folders above them, so a `git checkout` or `npm ci` that does is refused
  there.
- **Non-admin sandbox: a `.git` that appears or changes is reported**
  (D332, #1682). When a `.git` at the top of a writable folder is new, or
  was replaced, when a command ends, Corbanu Terminal warns at startup and
  the TUI asks you to confirm it once you have checked its hooks and config:
  git runs them outside the sandbox. Commands can still create a `.git`
  (`git init` works). A `.git` that is a link is reported after every
  command. `.git` folders deeper inside aren't checked.
- **Non-admin sandbox: possibly planted folders are reported where you see
  them** (D424, #1775). A `.git`, `.codex` or `.agents` folder a command may
  have planted is now named in a warning right after that command ends (for
  one that leaves processes running, after a later command), not only at the
  next start. At startup, a flagged `.git` is shown (and the TUI
  asks about it) when it lies in any folder commands may write to,
  `%TEMP%` included, or below the working folder; before, only one in the
  working folder or a folder above it was.
- **Non-admin sandbox: nested and bare repositories are watched** (D420,
  D421, #1775). Repositories deeper in a writable folder (a nested `.git`,
  or a folder that is a bare repository), and a writable folder that is
  itself a bare repository, are reported like the top-level `.git` when they
  are new or replaced, or when their config, hooks or other files git runs
  programs from change (compared by content, submodules and worktrees
  included). The warning says what changed. A new repository with exactly
  the shape `git init` or `git clone` leave isn't reported, so cloning a
  project to look at it doesn't ask; later changes to it are. Nothing is
  refused: agents can still commit and run `git config` there. Going back
  to an older build drops these records the next time it saves its own.
- **Non-admin sandbox: `.codex`/`.agents` below a folder whose name isn't
  valid Unicode count as planted** (#1877). The folder holding a folder with
  such a name is flagged after every command, so what lies below it isn't
  loaded until you confirm it; before, nested folders below it weren't
  recorded at all. (When that is the workspace itself, its own
  `.codex`/`.agents` count too until the folder is renamed or removed.) Such a
  folder now gets one "isn't valid Unicode" note, not one per entry, so many
  such names no longer push earlier warnings out of the post-command notice.
- **Non-admin sandbox: the repository checks cost about half as much**
  (#1877). The checks for planted `.codex`/`.agents` folders and nested
  repositories that run at each command's start and end now use several
  threads. On a 38,000-file tree with 100 repositories (native Windows CI,
  4 CPUs) each side went from about 80-90 ms to about 40 ms. They find
  exactly the same things, in the same order.
- **Non-admin sandbox: hooks folders a git config names are watched** (D422,
  #1775). A `core.hooksPath` folder in the project (Husky, `.githooks`), an
  included config file, or a script a program key names, when it lies in a
  writable folder, is recorded by content; a new or changed file there is
  reported like a changed repository. Nothing is refused: Husky still
  installs inside the sandbox.
- **Default (elevated) sandbox: possibly planted folders are reported too**
  (D425, #1846). The `.codex`, `.agents` and `.git` folders, repositories
  and hooks folders the non-admin sandbox watches are now watched in the
  default sandbox as well: when a command ends, what it created or changed
  there gets the same warning right after the command, at startup and in the
  TUI prompt. Nothing is refused. Processes a command leaves running aren't
  waited for.
- **Default (elevated) sandbox: a change inside an existing nested `.codex`
  or `.agents` folder is reported too** (#1896). The non-admin sandbox
  keeps such a folder (below the top of a writable folder) unchanged; the
  default sandbox doesn't protect it, so it now records what is inside each
  one when a command starts and reports the folder, like a planted one,
  when its contents differ after the command. Nothing is refused. A folder
  too large to compare (over 20,000 entries or 64 MiB) is reported after
  every command.
- **Default (elevated) sandbox: a `.git` made for a read-only `.git\hooks`
  is protected from the first command** (#1758). At the top of a writable
  folder (other than `%TEMP%`) with no `.git`, a configured read-only
  `.git\hooks` makes the sandbox create `.git` before the command starts.
  The command that caused this could still write `.git\config`, `HEAD` or
  `objects`; only later commands found `.git` protected. Now that first
  command can't either, as in the non-admin sandbox. The same holds for a
  `.codex` or `.agents` folder made this way. A deeper one, such as
  `sub\.git` for `sub\.git\hooks`, still has only `hooks` protected.
- **Default (elevated) sandbox: starting a command no longer holds up
  other work** (#1896). Its preparation, which includes walking the
  writable folders for the planted-folder records (about 0.06–0.08 s on a
  38,000-file repository, 0.5–0.7 s on a `repo` client with 100 projects) and
  the credential watch's first scan, ran on the thread that drives other
  tasks; it now runs on its own. What is flagged is unchanged.
- **Non-admin sandbox: starting a command no longer holds up other work**
  (#1909). Its preparation (the command's token, the folder permissions
  and labels, and the planted-folder walk) and the start of the command
  itself ran on the thread that drives other tasks: in our measurements
  each command held it for about 2–4 s on a 38,000-file repository and
  4–8 s on a `repo` client with 100 projects. They now run on their own,
  in the same order; other tasks wait at most a few tens of milliseconds.
  What is flagged is unchanged.
- **Both sandboxes: the planted-folder watch is a little faster in a `repo`
  client** (#1909). Each git folder in `.repo\projects` is now looked
  through once per command start or end instead of twice (once as a
  repository, once through its work tree's `.git` file), with the same
  result: about 3% off each (0.51 s to 0.49–0.50 s with 100 projects).
- **Default (elevated) sandbox setup no longer fails now and then where
  Windows requires complex passwords** (#1776). Setup gives the sandbox's
  user accounts a fresh random password, and about one setup in 4,500 drew
  one without the mix of characters that Windows' "Password must meet
  complexity requirements" policy needs (the Windows Server default, and
  common under domain policy), so setup failed with
  `helper_user_create_or_update_failed`. Passwords now always hold
  upper-case and lower-case letters, digits and symbols.
- **Non-admin sandbox: links can't undo a read-only path** (#1645, D293,
  #1542; #1670, #1667). A command doesn't start when a configured read-only
  path, or a folder above it, is a link (junction, symbolic link or mount
  point) inside a writable folder, which a command could replace; the message
  says to configure the link's target instead. A read-only path spelled
  through a writable folder's link gets the same denies (#1645). Cloud
  placeholder folders (OneDrive) at or above a read-only path, or as one, now
  get the same denies as plain folders, and a writable root that is a
  placeholder gets its label back (#1671, #1668).
- **Non-admin sandbox: a configured read-only path that doesn't exist is
  created and protected** (#1687, #1686, #1705; D333, D342), as the default
  sandbox already did. Before the command starts, it's made as an empty
  folder, so a command can't create it and fill it. A path meant to be a
  file (`.env`, say) therefore becomes an empty folder, and a read-only
  `.git\hooks` in a folder with no `.git` creates a protected `.git` with
  only an empty `hooks` folder, which git doesn't treat as a repository. To
  keep a file there, create it before you start a session. See "Read-only
  paths that don't exist yet" in `docs/sandbox.md`.
- **Non-admin sandbox commands start faster in large workspaces** (#1612,
  D285). Their delete protections are now written without rewriting the
  permissions of every file below (each is still checked, and anything
  unexpected falls back to the old method). In the PR's measurement on a
  101,000-file repository, a command took about 20 s instead of 31 s.
- **Non-admin sandbox: the faster protection walks don't start at a link**
  (D352, #1754, #1722). When the folder a walk starts at is itself a
  junction or symbolic link, or is swapped for one as the walk starts, it
  now leaves that folder to Windows' own walk, so the result always matches
  what Windows does. Corbanu doesn't normally start a walk at a link, so
  nothing changes in normal use.
- **Known limitation, now documented: the non-admin sandbox breaks tools that
  use named pipes** (#1797, #1561, #1778). Its commands can't write to
  named pipes they create, so Node.js child processes with piped output
  (`EPERM`), Git Bash (`couldn't create signal pipe`), a local `git clone`
  and Python `multiprocessing` fail there. This is not new in this release.
  The default (elevated) sandbox runs all of them; see "Known limitations"
  in `docs/sandbox.md`.
- **Known limitation, now documented: non-admin sandbox commands can write
  in `C:\ProgramData\Microsoft\DeviceSync`** (#1770). Windows labels that
  folder Low and lets everyone write to it. This is not new in this
  release, and we accept it; don't keep files there that you rely on.
- **Known limitation, now documented: default (elevated) sandbox commands
  can read other projects' running commands** (#1803). A command can read
  the memory of another project's command running at the same time
  (secrets in its environment, what it prints) and, for one run in a
  terminal, its console host (what you type and what it prints). It can't
  change them. This is not new in this release. A change awaiting security
  sign-off (#1800) stops the memory read, and a follow-up stops the console
  read; until then, avoid running commands in several projects at once.
- **Default (elevated) sandbox: a command that leaves a process running
  returns when it exits** (#1830). A process started with the command's
  output handles (PowerShell's `Start-Process -RedirectStandardOutput`, for
  example) kept the command "running" until that process exited too, with no
  time limit. Now one-shot commands and unified exec sessions in the default
  sandbox finish at most about 2 seconds after the command exits, and the
  process it started keeps running; output that process writes later is not
  shown. The non-admin sandbox's one-shot commands behave this way since
  #1826, and its unified exec sessions now do too (#1861).
- **A default (elevated) sandbox command now starts while a non-admin
  sandbox command is running** (#1819). Before, when the non-admin command's
  writable folders held the Corbanu home (for example `%TEMP%`, or a project
  folder above it), the elevated command failed after 15 seconds with
  `timed out after 15000ms connecting runner pipe-out`. The folder with the
  sandbox's helper programs now keeps its normal (Medium) integrity level.
  If that folder is a link, it is left alone and the sandbox log says so. If
  copying the helper fails and the one next to `codex.exe` is used instead,
  a Low label on it is fixed the same way, or the log names it as the cause.
- **Windows PowerShell's `Start-Process` works in both sandboxes** (#1805).
  Corbanu passes commands your environment, where Windows spells the search
  path `Path`, and both sandboxes added a second copy spelled `PATH`. Windows
  PowerShell 5.1's `Start-Process` then failed with "Item has already been
  added", and the copy that counted could miss folders added by
  `shell_environment_policy.set`. Each variable now reaches a command once,
  and a value you set wins over the inherited spelling; no folder is added
  to or removed from what you configured. Where Corbanu sets a proxy
  variable in both cases (`HTTP_PROXY` and `http_proxy`), sandboxed
  commands get only the capitals, so tools that read only the lower-case
  name (MSYS2 or Git Bash `curl`) no longer see that proxy.
- **Remote and exec-server commands on Windows also get each variable once**
  (#1817). The exec-server added Corbanu's variables beside the ones it
  inherited with exact case, so an inherited `Term` next to Corbanu's `TERM`
  reached the command twice and either value could count. Corbanu's value
  now replaces every inherited spelling, as in the sandboxes since #1805.
- **The planted-folder watch reads only the file it checked** (#1909, both
  sandboxes). When the watch records a file's content (inside a nested
  `.codex` or `.agents` folder, in a repository's git folder, or a file a
  git config names), it now opens the file it checked itself, so a link
  swapped in after the check is not followed. Before, a process a command
  left running could swap the file for a symbolic link in that moment, and
  Corbanu would read through it, for example from a network share. Such a
  swapped file is now recorded as unreadable, so the change is flagged. A
  link that is already in a git folder is still read through, as git
  reads it. A file Windows can't read as itself (for example a placeholder
  whose provider isn't running) now counts as changed after every command.
