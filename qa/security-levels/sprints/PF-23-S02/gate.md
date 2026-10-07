# PF-23-S02: per-sprint gate (2026-10-07)

- **Branch:** `feat/pf-23-s02`, off main at `64137b7189`. Behind `source_envelopes`, Moderate/Aggressive only.
  Flag off and Permissive: `post_taint_state()` is `None`, so no check runs. One accepted change outside the flag:
  the macOS rename guard applies to every Seatbelt policy that already denies reads (a user's own deny-read config,
  the Aggressive picker profile): folders above a denied path can no longer be renamed or removed in the sandbox,
  and a missing folder on the way to a denied path can no longer be created there.

## Denied-surface matrix (with `source_envelopes`)

| Surface | Moderate, before untrusted content | Moderate, after | Aggressive (from session start) |
| --- | --- | --- | --- |
| Reads: Corbanu home, other homes, `$HOME` credentials | unchanged | denied by the sandbox; a fresh approval of the exact protected command lifts it for that run | denied; only a matching grant lifts it for that exact command |
| Writes: shell start-up, `~/.config/{fish,git,autostart,systemd,environment.d}`, `~/.local/bin`, `~/.claude`, `~/Library/LaunchAgents`, `.gitconfig`; the enclosing repository's hooks, config, config.worktree, commondir (Linux: when it exists) and `.git` file (worktree git folders too); project `.codex`/`.agents`; Corbanu `tmp`, `shell_snapshots`, skills, plugins, packages, `AGENTS.md` | unchanged | read-only where the profile allowed writes; same lift | read-only; same grant |
| Renaming or removing a folder above a protected path or a protected link; creating a missing parent (`mv x ~/.config`) | unchanged | denied (macOS; Linux mounts a missing workspace part read-only and leaves missing home paths to the command-text net) | same |
| Command text naming such a write, move or removal (`>`, `>>`, `>&file`, writer commands, `git config` run keys or `--global`, cron, launchd, `systemctl --user`, patches) | unchanged | fresh human approval (Persistence) | fresh approval; rules stay unless granted |
| Codex Apps upload read | unchanged | through the protected sandbox; none available: refused | same |
| Typing into a process started without the rules | unchanged | asks once per process start (policy epoch bound); refused with approvals off | refused unless a grant names that start |
| Unknown surface, other agent, child, other session, expired, used up, other epoch, kill switch | — | — | no grant applies; another level or the kill switch drops all grants |

## Gate results

- **Focused:** `just test -p codex-core pf_23_s02 pf_23_s01 pf_30_s03 mcp_openai_file aggressive confined`: 75/75.
  `just test -p codex-sandboxing renaming_a_folder`: 1/1 (real `sandbox-exec`). Two seatbelt arg tests
  (`create_seatbelt_args_for_cwd_as_git_repo`, `..._with_read_only_git_and_codex_subpaths`) fail the same way on
  main in this checkout (temp folder under `/Volumes`), not from this change.
- **Review (Opus 5.5 High):** `.codex-work/workers-20261002/pf23s02-review{1..5}/`; round 5 APPROVE. Round 1 CHANGES REQUESTED: parent
  rename and symlink bypasses on macOS, Linux bwrap failing on symlinked or worktree paths, worktree hooks, call-id
  confinement keys, grant resurrection, upload fallback, classifier gaps; fixed in `bd9d3aa48b`. Round 2 CHANGES
  REQUESTED: rename onto a missing parent, Linux placeholders in the real home, the enclosing repository from a
  subfolder, dangling links, a single confinement marker; fixed in `b25eb6460b`. Round 3 CHANGES REQUESTED: git
  could be pointed elsewhere through `commondir` or a rewritten `.git` file, Linux placeholders through dangling
  links; fixed in `23a1818ec8`. Round 4 CHANGES REQUESTED on Linux records only: anything below `.git` (but
  objects, refs, index) is now Persistence for the command-text net, and the Linux limits are recorded below.
- **Full crate** at `93e149e066` (merged with main): `just test -p codex-core` 3,897 of 3,902. Four failures
  fail the same way on main (`config_schema_matches_fixture` and the 3 known baselines);
  `shell_command_snapshot_still_intercepts_apply_patch` failed once under load and passes 3/3 alone (it passed in
  the earlier full run). `just test -p codex-sandboxing`: only the two baseline seatbelt arg tests fail.
- **Linux clippy** (`cargo clippy --locked -p codex-core -p codex-memories-write --all-targets -- -D warnings`) on
  the RTX box: clean at `93e149e066` (round 1 caught two too-many-arguments and one type-complexity error).
- **GLM 5.2 videos** at `93e149e066`, in [qa/demos/index/PF-23-S02.md](../../../demos/index/PF-23-S02.md):
  Aggressive denies a hidden home read before any untrusted content; with full access after untrusted content a
  run-time-built hook write gets "Operation not permitted" while an ordinary write works; a literal `~/.zshrc`
  append asks first (declined, file unchanged); typing into a shell started before untrusted content asks once.
  Earlier takes were recorded verbatim: GLM refused two hook/alias requests it read as injected.
  No video for grants (no issuing UI until PF-25-S01) or uploads (need ChatGPT auth): unit tests only.

## Known limits

- Grants have no issuing UI yet (PF-25-S01 calls `aggressive::issue`); until then nothing lifts the Aggressive rules.
  Broker, retrieval, browser-login and derived-data adapters do not exist yet; they must call `aggressive::admit`.
- Linux: a symlinked dotfile is protected at its target only (bwrap binds targets), and missing home files
  (`~/.bash_profile` and the like) are left to the command-text net: a sandbox placeholder would change the user's
  own login shells.
- A writable root configured inside a protected folder keeps that folder's protection off (as PF-23-S01 reads).
- Follow-up (2026-10-07): the hooks, config, config.worktree and commondir of every folder below `.git/modules`
  (walked without following links, git's own folders skipped) and the folder `core.hooksPath` names (repository,
  `~/.gitconfig`, `~/.config/git/config`; `~/` expanded) are read-only too (Linux: when they exist). Past 512
  folders, 8 levels, or a folder that cannot be read, all of `.git/modules` is read-only (submodule updates then
  fail under the rules); an unreadable folder on the way to `.git` makes that `.git` and those above it read-only,
  and the paths (and any config's `hooksPath`) this process found before still hold; after a restart with the
  folder still unreadable they are not known (on Linux, a `.git` placeholder bwrap cannot create then fails
  every command until the permissions are fixed). A module folder holding very
  many plain files slows every command. Not covered: config includes, `~user/`, `GIT_CONFIG_GLOBAL`; a module folder a command
  creates is protected from the next command on; a new nested `.git` stays with the command-text net (the profile
  cannot deny creating one name anywhere without denying the repository's own).
- Linux, full-write profiles only: a missing `commondir` is left to the command-text net (an empty placeholder
  breaks git), and renaming or removing an existing folder above a protected path is not blocked (bind mounts
  move with the folder), so `mv .git .g` and rebuilding `.git` is caught only when the path is written out.
  macOS blocks both. Follow-up (2026-10-07): `linux-sandbox` test
  `pf_23_s02_full_write_profile_keeps_protected_files_read_only` pins this on Linux (a protected file stays
  read-only through a symlinked dotfile and after its folder is renamed; the rename, rebuild and link removal go
  through), and `pf_23_s02_persistence_files_become_read_only` pins that missing home files leave no placeholder.
  It found that every full-access command with these rules failed on Linux (`bwrap: Can't mkdir /.git`): the
  filesystem root of `:root = write` no longer gets repository-metadata masks.
- External sandboxes and remote environments take no extra rules; processes there count as unconfined.
- Sandboxed upload reads load the whole file (within the 512 MiB limit).
