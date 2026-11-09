## Sandbox & approvals

For information about Codex sandboxing and approvals, see [this documentation](https://developers.openai.com/codex/security).

## Windows non-admin sandbox

On Windows, Corbanu Terminal can sandbox commands in two ways. The **default
sandbox** needs a one-time Administrator setup. The **non-admin sandbox**
(also called the unelevated or legacy sandbox) needs no setup, but it protects
less.

**What the non-admin sandbox stops.** Sandboxed commands run at Low
integrity. In both Read Only and workspace-write modes they can't create,
change, delete or move files and folders outside the folders they may write
to, except in places marked low-integrity: `AppData\LocalLow`, anything you
or a program labelled Low, and folders another non-admin sandbox session is
using at that moment.

**What it does not stop.**

- Reading your files (see "Files a profile protects from reading" below).
- Network access by programs that ignore proxy settings: the network block
  only sets proxy variables and offline settings for common tools.
- A few ways of starting programs that leave the command's job, so cancelling
  doesn't end them (see #1507 in `codex-rs/core/README.md`; measured on a
  server without a desktop session, not yet on a desktop).
- Two commands running at the same time in different projects (or with
  different extra folders they may write to) aren't kept apart: one can
  borrow the other's access and write in that project's folders while the
  other runs (#1560), and read the other's memory, such as secrets in its
  environment or what you type at its prompts (#1838), so avoid running
  commands in several projects at once. If your account is an
  administrator, the default sandbox closes most of these, but not reading
  another project's terminal window (#1803; see below).
- Drives without Windows permissions, such as FAT or exFAT USB drives: neither
  sandbox can protect files there.

The default sandbox runs commands as a separate account with a firewall-based
network block, and protects more. If your Windows account is an administrator,
each of its commands runs on a logon of its own, so one command can't borrow
another's processes or token or read the memory of another command's own
processes (#1560). It can still read another project's terminal window (#1803;
see below). The program that starts them runs as a third account,
`CodexSandboxRunner`, which no command uses (#1721). Setup gives that
account the Windows right "Replace a process level token"
(`SeAssignPrimaryTokenPrivilege`), which it needs to start each command on
its own logon, and denies it network, remote desktop,
batch and service sign-in. Its password is stored, encrypted for this
computer, in the sandbox's secrets folder under your Codex home. That folder
gives you, administrators and the system access; if your Codex home is
somewhere other accounts may read (for example a folder at the root of a
drive), they may be able to read it too. Anyone who can read the password can
sign in as that account and, with that right, possibly gain full control of
the computer; that includes a program running as you. To remove the account,
delete the `CodexSandboxRunner` local user; setup creates it again the next
time it runs. Windows keeps the rights it was given, which no other account
can use.

**If your account isn't an administrator**, an administrator types their
password into the setup's Windows prompt (or runs `corbanu sandbox setup` for
you), so setup runs as their account rather than yours. Setup then doesn't
create `CodexSandboxRunner`: with that right, anyone who can sign in as you
could become an administrator. Setup creates it only when it runs as the
account Corbanu Terminal runs as, which is also the account signed in to the
Windows session. Changing environment variables such as `USERNAME` doesn't
get around this. Without the account, the default sandbox works as it did
before: the program that starts each command runs as the command's sandbox
account, and the command starts from that program's logon. Corbanu Terminal
tells you so after setup. Commands in different projects can then still
interfere with each other: one can run code with the sandbox account's full
access while another command starts (#1721), borrow another's access (#1560)
and read its memory (#1803), so avoid running commands in several projects at
once. You keep the default sandbox rather than being moved to the non-admin
one, which leaves the same gaps (#1560, #1838) and protects less otherwise.
For the stronger isolation, run Corbanu Terminal from an administrator account.
If your own account becomes an administrator later, delete `sandbox_users.json`
from the `.sandbox-secrets` folder in your Codex home; setup then runs again
before the next command. If an account that used Corbanu Terminal stops being
an administrator, delete the `CodexSandboxRunner` local user (and that
`sandbox_users.json`): the account, and the password stored for it, would
otherwise keep working.

Setup also leaves the account out if it can't look up either account name
(for example a domain account while the domain controller is unreachable);
delete `sandbox_users.json` as above to retry. And if you, as an
administrator, run another account's program on your own desktop and approve
the setup prompt it raises, that account gets the runner account: approve
only prompts you started.

Two gaps remain with the runner account too:

- **Another project's terminal window** (#1803). A command that runs in a
  terminal (a pseudo-console) gets a console host process (`conhost.exe`),
  which the program that starts commands creates. Any other default-sandbox
  command, in any project, can read that process's memory: what the command
  printed and what you typed at its prompts, such as a password. It can
  probably also end that process, and with it the terminal command. This was
  measured on a real Windows desktop: a command in one project found a secret
  that a command in another project had printed, in this version and the
  previous one, and whether Corbanu Terminal ran elevated ("Run as
  administrator") or not. Until this is fixed, while commands from another
  project run, don't type secrets into a project's command and don't run
  commands that print them (tokens, `env`, credential helpers).
- Another elevated command can resume a command's suspended threads.
  Commands aren't started suspended, so this matters only for threads a
  command suspends itself.

**When you will see a warning.** If your sandboxed commands run in the
non-admin sandbox, Corbanu Terminal tells you once per session:

- in the TUI, as a warning at startup and after you choose the non-admin
  sandbox during setup; the setup screens and `/permissions` also describe the
  risk;
- in `corbanu exec`, as a `warning:` line on stderr;
- in `corbanu sandbox`, as a `warning:` line on stderr before the command runs;
- in app-server clients, as a configuration warning.

The warning never blocks a command.

**Project folders created while a command runs.** If the `.codex` or
`.agents` folder at the top of your workspace (or of another folder commands
may write to) doesn't exist when a non-admin sandbox command starts and
exists when it ends, a command may have planted it. Corbanu Terminal then
doesn't load its config, hooks or skills, warns you at startup, and asks in
the TUI whether to load it (once the command, and anything it left running,
has ended). Check its contents before you choose to load it. Such folders
deeper in your workspace are covered too. One that is new, or was replaced,
when the command ends is treated the same way. A read-only `.codex` or
`.agents` folder your permission profile names that doesn't exist yet (or
one on the way to a read-only folder inside it) is created by Corbanu
Terminal before the command starts, so that command doesn't count it as
planted; another command that was already running still does. Commands
can't change, move or delete the ones that existed when they started, or
the folders above them. A folder the sandbox can't list counts as planted. Folders inside
`.git` and the temporary folder (`%TEMP%`) aren't checked.

**A `.git` created while a command runs.** Git runs a `.git` folder's hooks
and config outside the sandbox whenever you run git there. So if the `.git`
at the top of a folder non-admin sandbox commands may write to appears, or
is replaced, while a command runs, Corbanu Terminal warns you at startup and
asks in the TUI whether you have checked it. Commands can still create a
`.git` (`git init` works). A `.git` that is a link (a junction or symbolic
link) is flagged after every command. Only that top-level `.git` is checked,
not `.git` folders deeper inside.

**Bare repositories a command plants.** Git also uses a folder that holds
`HEAD`, `objects` and `refs` as a repository (a bare repository), and reads
its config and hooks. A sandboxed command can make one in any folder it may
write to, and git run there or in a folder below it later runs whatever
that config names. Corbanu Terminal doesn't check for these (#1775), on
Windows or elsewhere. We recommend making git refuse a bare repository it
finds by searching:

```shell
git config --global safe.bareRepository explicit
```

Use git 2.45 or later: 2.38 to 2.44 also refuse git run inside `.git`,
which breaks some hooks and submodule tools, and versions before 2.38 ignore
the setting. It covers the git command line
with your global config, not tools built on libgit2 or JGit, nor a git with
another home folder (such as WSL's). Where you do work in a bare repository,
name it with `--git-dir` or `GIT_DIR`.

**Read-only paths that don't exist yet.** If your permission profile marks a
path inside a writable folder read-only and that path doesn't exist when a
sandboxed command starts, Corbanu Terminal creates it as an empty folder and
protects it, so the command can't create it and fill it. Both Windows
sandboxes do this. It has two side effects:

- A path meant to be a file, such as `.env`, becomes an empty folder wherever
  it's missing, and tools that expect a file there may fail to read it. To
  keep a file there, create the file before you start a session. If the
  folder has already been made, delete it while no session is running, then
  create the file.
- A read-only path inside `.git`, such as `.git\hooks`, in a folder that
  isn't a git repository yet creates a `.git` there that holds only an empty
  `hooks` folder. When that `.git` is at the top of a folder commands may
  write to (other than `%TEMP%`), both sandboxes protect the whole of it at
  once, starting with the command that made it (#1758); a deeper one, such
  as `sub\.git` for `sub\.git\hooks`, has only `hooks` protected. Git
  doesn't treat such a `.git` as a repository, so git commands in that
  folder report "not a git repository" or use a repository in a folder
  above it. Run `git init` there yourself, or remove the entry and then
  delete that `.git` folder while no session is running.

**Files a profile protects from reading.** The non-admin sandbox can't stop a
command from reading a file. So if your permission profile denies reading some
files (`deny` entries in your config, or the protected paths a security level
adds), Corbanu Terminal does not run the agent's commands in the non-admin
sandbox. The command fails with a message that it was not run because this
Windows sandbox mode can't block reading the protected files. The same applies
to a profile that limits which folders commands may read. Switch to the
default sandbox, as below, to run it. Profiles without such entries are not
affected.

**Known limitations: tools that use named pipes.** (#1797) A command in the
non-admin sandbox can't open a named pipe it created for writing when the pipe
has Windows' default permissions. By default Windows lets only the pipe's
owner, SYSTEM and Administrators write to a pipe, and the non-admin sandbox's
write restriction doesn't count the command as the owner. So opening such a
pipe for writing, as most programs do to connect to it, fails with "access
denied" (Windows error 5). These fail there:

- Node.js starting a child process with piped input or output (`execSync`,
  `spawnSync`): `EPERM`. A child started with `stdio: 'inherit'` or
  `'ignore'` works.
- Git Bash (`bash.exe`) crashes on start with
  `couldn't create signal pipe, Win32 error 5`.
- A local `git clone` (from a folder on your computer) fails with the same
  crash, from Git's own `sh.exe`.
- Python `multiprocessing` (for example `multiprocessing.Pool`):
  `PermissionError: [WinError 5] Access is denied`. `subprocess` with pipes
  works.
- Other programs that create and then use their own named pipe, such as .NET's
  `NamedPipeClientStream` in PowerShell (`Access to the path is denied.`).

The default sandbox runs all of these. If you need them, switch to the default
sandbox as below. This was measured on Windows Server 2022, in both an
administrator and a normal session.

**How to switch to the default sandbox.**

- In the TUI, run `/setup-default-sandbox` and approve the Administrator prompt.
  Corbanu Terminal saves the choice for later sessions.
- Or set the mode in `config.toml` in your Corbanu home folder:

  ```toml
  [windows]
  sandbox = "elevated"
  ```

  You can also pass `-c windows.sandbox=elevated` for a single run. If setup
  has not run yet, the first sandboxed command asks for Administrator approval
  once.

**Several sessions at once.** The default sandbox's read protections are
shared by every Corbanu Terminal session on the computer, whatever its Corbanu
home folder. When you remove a `deny` entry, its protection stays while any
other session that may rely on it is still running, and goes with the first
command you run after they have all ended. To track this, each session that
protects files registers itself in `C:\ProgramData\CorbanuTerminalSandbox`
while it runs. The sandbox setup creates that folder, owned by Administrators,
so updating to this version asks for Administrator approval once more. Until
the setup has run, and while that folder is missing or owned by anyone else,
no protection is removed; if that happens after the setup has run, sandboxed
commands with the secretless launch on are refused until you run the setup
again (it repairs the folder). Sessions started by an
older version are not registered, so their protections aren't kept for them.

**Over SSH.** The default sandbox also works when you start Corbanu Terminal
in a Windows OpenSSH session (or from a service). There, Corbanu Terminal
gives the command runner's logon (the SSH session's own, not the sandbox
accounts every Windows user's sandbox shares) the least access it needs in the
session's window station and desktop (no hooks, windows or clipboard). Once
the runner has started its command, that access shrinks to what the command
and its children need to start: they can't create desktops there, or open
your session's desktop unless `windows.sandbox_private_desktop = false`. It
goes when the runner exits, even if Corbanu Terminal has exited by then, so a
background process left running after its command ends can't start new
programs that need a window station. (If Corbanu Terminal is ended in the
moment between starting a runner and handing its access to the small helper
process that removes it, or that helper can't start and Corbanu Terminal
exits before the runner, that access stays until the SSH session ends.) The
non-admin sandbox with `windows.sandbox_private_desktop = false` works over
SSH too. Earlier builds couldn't run any command there: each one failed after
15 seconds with `windows sandbox: timed out after 15000ms connecting runner
pipe-in`, and the System event log showed `codex-command-runner ...
(0xc0000142)`. If you see that with an older build, update, or start Corbanu
Terminal from the console or Remote Desktop session instead.

Your organization's managed requirements may allow only the non-admin sandbox.
In that case the warning still appears so you know the limits.

This page documents the shipped capability "existing general sandboxing" from
the [product specification](corbanu-product-spec.md) heading "Shipping MVP —
LIVE": "The shipping MVP already has a wallet, vault, scoped signing, approvals,
and general workspace sandboxing."
