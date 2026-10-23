# PF-30-S03 slice 2: per-sprint gate (2026-10-06)

- **Branch:** `feat/pf-30-s03-finish`, off main at `38516a5b22` (slice 1 merged as #204).
- **Scope:** behind the default-off `source_envelopes` feature, Moderate and Aggressive only. Permissive and
  flag-off are unchanged: the check returns before classifying.

## What ships

| Piece | Behaviour |
| --- | --- |
| Approval bound to policy | A post-taint approval covers the taint generation and the effective policy snapshot it was given under: epoch, revocation generation, kill switch, level and actor chain (lineage). Any change while the prompt is open (first prompt or the escalation retry) refuses the action; so does the check no longer applying. An active kill switch refuses before asking. |
| PF-26 counts | Each decision logs `post-taint decision` with kind, taint generation, outcome (`approved`, `declined`, `refused_approvals_off`, `refused_stale`, `refused_kill_switch`) and how long the human took; classification logs its time (`classify_us`). |
| Classifier: shell forms | ANSI-C `$'..'`, line continuations, per-word bounded brace expansion (not inside quotes), `$IFS`, comments, left-to-right `$VAR` substitution of earlier assignments, `echo`/`printf` and lookup substitutions (`which`, `dirname`, `$(cd X && pwd)`, `git rev-parse --show-toplevel`) spliced into their word. Pipes survive empty commands (`\| (sh)`, `\|&`, a newline) and feed every command in a following `( )`, `{ }` or `if`/`while`/`for`/`case` group; `>(...)` reads the outer output; `< file` and `<<< word` are followed. Folders: `cd` (bare, `-`), `pushd`/`popd`, `-C`/`--directory`/`--chdir`, with `$PWD` kept in step. `~user`. |
| Classifier: interpreters | Options are read per family up to the first operand, letter by letter with attached values: shells, Python (`-c`, `-m` local modules), Node/Deno/Bun/tsx (`-e`, preloads, subcommands), Perl/Ruby (`-e`, `-pe`, `-r`), PHP, Lua, osascript, PowerShell, Tcl/expect/swift/R; versioned names (`python3.12`). Wrappers and shell keywords are skipped with their own options (`sudo`, `env`, `timeout`, `xargs`, `flock`, `taskset`, `busybox`, ...). Inline code: adjacent literals joined as a path, as words and plainly, nested literals, escapes decoded. Base64/hex words decoded and classified. |
| Classifier: files | Scripts run by a shell, an interpreter, `source`, a stdin redirect or by path are read (256 KiB, 16 files, depth 4); executables are recognised by magic number. Patches that add lines to runnable files (scripts, hooks, `.git/config`, `.gitconfig`, `.husky/`, task files, shell start-up files, shebang) are classified by those lines, using the file the patch really writes. |
| Classifier: unseen code | A fourth kind, shown as "running code the host cannot read first", gated like the others: code fed to a shell or interpreter through a pipe, group or substitution by anything but a visible `echo`/`printf` or neutral lookup; `xargs`/`parallel` code; code, script paths or here-strings named by a variable that is never assigned in the action or comes from an unseen substitution (special parameters such as `$@` never count); a required script that is missing, not a regular file, too large or past the limits; a spent lookup budget; a classification over 3 s. |
| Classifier: paths | Homes are matched segment by segment with globs. Recursive readers (tar, zip, 7z, rsync, scp, ditto, cpio, pax, `cp -r`, `grep -r`, `rg --hidden`, `find -exec` or into a pipe), with grouped short flags, pointed at a folder that holds the user or Corbanu home are credential access. Symlinks and canonical spellings are followed with cached lookups (only path-like words inside bodies) that never touch automounts, network or removable volumes, or macOS privacy folders unless under the working folder, the Corbanu home or temp; lookups run off the async workers. |

## Gate results

- **Formatting and lint:** `just fmt` and `just fix -p codex-core` clean. Bazel lock unchanged
  (`bazel mod deps --lockfile_mode=error` passes; the new dev-dependency is a workspace crate).
- **Focused tests:** `just test -p codex-core pf_30_s03` passes 27: 18 unit (classification incl. every bypass
  from review rounds 1-3, everyday commands staying quiet, limits failing closed, symlinks, script files and
  patches, PF-26 research workflow, recheck binding, live controller change, the orchestrator itself with a
  probe tool), 1 apply-patch runtime (preapproval ignored with fresh authority), 8 suite (slice 1's four plus
  hook allow ignored, escalation retry asks again, recalled memory gates a first protected action, a tricked
  child agent refused).
- **Full crate at `8d09e0345d`:** `just test -p codex-core` ran 3,813, 3,810 passed (two flaky tests passed on
  retry). The 3 failures are the known baselines (`skills_append_to_developer_message`,
  `skills_use_aliases_in_developer_message_under_budget_pressure`,
  `remote_compact_trim_estimate_uses_session_base_instructions`). `config_schema_matches_fixture` passes.
- **PF-26 numbers:** a 25-command research workflow plus 5 protected actions after untrusted content gives
  exactly 5 prompts; classification takes about 1 ms per action in a debug build (1-4 ms in the live TUI runs,
  from the `classify_us` log field).
- **GLM 5.2 tmux runs and videos** at `8d09e0345d`, in [qa/demos/index/PF-30-S03.md](../../../demos/index/PF-30-S03.md):
  - inline Python that joins `'/.ss' + 'h'`: approval prompt naming credential access;
  - `cat greet.sh | sh`: approval prompt naming code the host cannot read (GLM itself refused a base64 variant);
  - a memory written with `!`, then `/new`: the first command (reading the Corbanu config) asks;
  - a parent reads notes.txt and spawns a helper to run the vault command: the log shows
    `outcome="refused_approvals_off"` for the helper's call.
  An earlier inline-code run at the same commit was refused by GLM before it ran anything; the recorded run asks
  in the user's own words.
- **Independent review:** Opus 5.5 High, read-only through `corbanu exec`, six rounds in
  `.codex-work/workers-20261002/pf30s03b-review{1..6}/`. Rounds 1-5 CHANGES REQUESTED, all on the classifier
  (limits failing open, stdin/variable/substitution routes, filesystem lookups, glob cost, option parsing, pipes
  into groups, brace expansion, over-matching of everyday commands); round 6 APPROVE at `8d09e0345d`. Its
  remaining Lows are listed under PF-23-S01.

## Known limits (moved to PF-23-S01)

- Routes that never reach the shared approval seam: MCP tool calls, `write_stdin`, code mode.
- What a command-text net cannot see: strings built at run time (`chr()`, environment lookups, names read from
  files), build tools running arbitrary code (`make`, `npm run`, build scripts), hard links. Typed resources and
  sandbox-level denial of home reads replace the net.
- Review Lows: `cd` inside a substitution or subshell, code through positional parameters/functions/`set --`,
  command-string wrappers (`su -c`, `ssh host cmd`), `${!x}`/`declare -n`, `cd -P`/`||` approximations, loop
  stdin from a process substitution, `awk system()`/`sed e`, symlink hops into automounts during lookups, the
  quote tracker losing state in rare forms (`true;#'`), and `shell=True` literals with over 64 brace alternatives.
- Outbound disclosure requests and value transfer proposed from tainted content.

## Accepted over-matching (after untrusted content only)

- A CLI name followed by `vault` in one command, including quoted arguments (`git commit -m "codex vault"`).
- Running a script that is missing or outside the allowed lookup locations, a command named by a variable never
  assigned in the action (`$EDITOR file`), `eval "$(ssh-agent -s)"`, `cat x | sh`, and the real `gradlew`/`mvnw`
  wrappers (they `eval` a computed line or take `JAVACMD` from a compound substitution) ask the human; under
  `never` they are refused.
