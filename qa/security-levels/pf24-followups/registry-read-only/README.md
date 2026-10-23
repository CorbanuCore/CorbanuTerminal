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

## Gate evidence

- **Tests** (after `just fmt` and `just fix`): `just test -p codex-tui -E
  'test(security::) | test(security_level_picker)'` (51), including the new
  registry tests (read-only subpath handed to the platform sandboxes, missing
  registry, glob path, nested-exec overrides).
- **GLM 5.2 tmux runs** (macOS): `tmux-run/1-delete-refused.txt` (recorded
  run: under Aggressive with the workspace as the account home, deleting the
  registry entry fails with "Operation not permitted" and a file elsewhere in
  the workspace is written); `tmux-run/2-ancestor-rename.txt` (the limit
  above).
- **Review** (Opus 5.5 High): approve with fixes; see `review/`.
- **Video:** [registry read-only](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/registry-read-only-registry-read-only-a51850d908b9-2026-10-06.mp4)
