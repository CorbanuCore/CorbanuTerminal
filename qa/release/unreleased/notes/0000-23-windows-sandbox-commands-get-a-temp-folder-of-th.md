## Windows sandbox: commands get a temp folder of their own (D377, #304)

- **The default (elevated) Windows sandbox no longer lets sandboxed commands
  write anywhere in your `%TEMP%`.** Each Corbanu session makes a folder of
  its own there (`corbanu-sandbox-tmp-…`), open only to you, SYSTEM and
  Administrators, and its sandboxed commands get it as `TEMP` and `TMP` (and
  as `TMPDIR` when that named `%TEMP%`). Only that folder is granted to the
  sandbox. Before, the sandbox's accounts kept lasting write access to all of
  `%TEMP%`: other programs' temp files, and other sessions'.
- **Upgrading needs no setup rerun.** The first time a session uses
  `%TEMP%`, it removes the grant older versions left there, as you (no UAC
  prompt). This also removes the non-admin sandbox's entry for that Corbanu
  home from `%TEMP%` itself (it adds it back when it next runs). Where you
  cannot change `%TEMP%`'s permissions, the old grant stays and is logged.
  Running an older version again puts it back.
- **A sandboxed command cannot turn its temp folder into a link** to make
  a later setup grant the sandbox somewhere else: the session keeps a file in
  the folder open, so it cannot be emptied or renamed, and checks the folder
  is still the one it made before each command.
- **Cleanup.** The folder is removed when the session ends; one a crashed
  session left behind is removed by a later session.
- On native Windows CI, in an administrator's and a normal session, Python
  (temp files, venv and pip), cargo, Node and npm, Windows PowerShell and
  PowerShell 7, git and Git Bash, and MSBuild work as before.
- **What older versions' sandboxed commands left in `%TEMP%` becomes yours**
  (#1871). The sandbox's accounts owned those files and folders, and an
  owner can change their permissions: a command could give its account,
  the sandbox's group or its workspace entries there that later commands
  carry, and keep changing them after the upgrade (shown on native Windows
  CI). In the same upgrade step, before the old grant is removed, the
  session, as you (no UAC prompt), makes you their owner and removes every
  allow entry other than yours, SYSTEM's, Administrators' and OWNER
  RIGHTS' (deny entries stay; a "no DACL" becomes one inherited from the
  folder). Nothing is deleted. One whose permissions stop you from doing
  that is renamed in place (`….corbanu-sandbox-leftover-…`), so programs
  that look for it by name make a new one of yours. The counts are logged
  in the Corbanu home's `.sandbox` log. A typical session leaves several
  hundred (Node's `node-compile-cache`, `MSBuildTemp`, temp files tools
  kept); on CI all were taken in under a second.
- Not changed: the non-admin sandbox still uses `%TEMP%` itself, and
  sandboxed commands can still read (not write) other sessions' temp
  folders.
