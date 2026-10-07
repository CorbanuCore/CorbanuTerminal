# Aggressive-homes registry read-only in the Aggressive sandbox

Round 4 (#219) records every Aggressive home in a per-account registry
(`~/Library/Application Support/Corbanu/aggressive-homes` on macOS,
`~/.local/state/corbanu/aggressive-homes` elsewhere), so a nested launch that
points `CODEX_HOME` and `HOME` elsewhere is still recognised. When the
workspace was the home folder, an agent command could delete those entries.

## Fix

- The Aggressive profile gets one more filesystem entry: the registry folder
  is `read`, like the Corbanu home. The sandbox design is otherwise unchanged.
- An Aggressive launch creates the folder before the first agent command, so
  a command cannot create it first.
- Verification gains a row: Aggressive is not shown as active if the
  registry would be writable by agent commands, if the workspace contains it
  but it is missing (bubblewrap would put a placeholder there), or if its
  path cannot be written into the profile (not UTF-8, or glob characters).
- The `/security` review's Sandbox row now says the Corbanu home and the
  registry stay read-only inside the current folder.
- The path comes from the account database, as in detection; debug builds
  honour `CORBANU_TEST_ACCOUNT_HOME`. With no account entry (some
  containers) there is no registry and nothing to protect.

## Limits

- **Renaming a parent folder still moves the registry away** when the
  workspace is the home folder: `mv Library moved-library` (macOS) or
  `mv .local x` works, because Seatbelt and bind mounts check the renamed
  path, not what is under it (`tmux-run/2-ancestor-rename.txt`, macOS;
  Linux is not verified). Closing
  that means making `~/Library` or `~/.local` read-only for agent commands
  when the workspace is the home folder, or keeping the registry somewhere
  the workspace cannot contain. Both change the sandbox design or the
  detection design, so they are left for Travis. Aggressive already warns
  when the workspace is the home folder.
- Plain `cargo test` without `CORBANU_TEST_ACCOUNT_HOME` (not `just test`)
  makes the launch tests create the registry folder in the real home, as an
  Aggressive launch does.
