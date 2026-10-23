# #626, #623: Aggressive re-run defects (G11b / AGG-33, G27 / AGG-R13)

Found by the "Aggressive ships" re-run (`../../milestones/aggressive-20261011-rerun/`, runs G11b and G27). Owner
decisions (Travis, 2026-10-13) are on the issues.

| Issue | Before (candidate 6ebfa1f47c) | After |
| --- | --- | --- |
| #626 | A "don't ask again" rule approved under Permissive ran `echo '# expiry-probe' >> ~/.zshrc` unasked under Aggressive | Under Aggressive only rules a person saved while Aggressive was active apply (marked in the Corbanu home's `rules/default.rules`); every other `allow` rule is suspended, not deleted. The person is asked, and the prompt says which remembered approval was not applied and that it applies again at a less strict level. Permissive and Moderate are unchanged |
| #623 | `ln -s .agents ag` created the link (writes through it were refused) | Under Aggressive a command that creates a link to the project's `.codex`/`.agents` (or Corbanu's read-only files) is refused before it runs; nothing is created, and the warning, the model's output and the inspector say why (never granted, not even once) |

## Why #623 is a pre-run check

No OS sandbox can refuse a symbolic link by where it points: Seatbelt filters only by the link's name or type, and
Landlock and seccomp cannot read the target. The check is in core, so it is the same on macOS, Linux and Windows.

- **Parsed links.** It reads `ln`/`gln` (`-s` or hard), `cp -s`/`-l`, `mklink`, PowerShell `New-Item …
  SymbolicLink|Junction|HardLink` and `CreateSymbolicLink`. It also finds them behind shell keywords, operators and
  wrappers (`sudo`, `env`, `timeout`, `find -exec`, …), in `sh -c` scripts and heredocs, and in interpreter code
  (`os.symlink`, `symlink_to`, `fs.symlinkSync`, `FileUtils.ln_s`, `os.system('ln …')`).
- **Where it applies.** The check runs on each agent command, on each program a zsh-fork script starts, and on text
  typed into a running shell or interpreter (`write_stdin`), one write at a time.
- **When a parsed link is refused.** When its target resolves into a protected path, either as written or as the
  system reads it through links.
- **Backstop.** Any command that may create a link (an `ln`/`gln`/`mklink` word anywhere, a link call in code, a
  PowerShell link item) and names `.codex` or `.agents` anywhere in its text is refused. The paths of its own folder
  and the workspace roots are left out of that check, so a project kept below a folder named `.codex` is not caught by
  it. The backstop catches names hidden in variables, `$(…)`, backticks, `eval`, pipes into a shell, globs (also
  `[[:alpha:]]`), braces, zsh groups (`.agent(s|)`), quotes or a backslash inside the name, and other letter cases.
  It is defense in depth against ordinary link commands, not against deliberate obfuscation.
- **Bounds.** Too many paths to check, or lookups that take longer than 2 s, also refuse.

Out of reach (writes through such a link stay denied by the sandbox):
- targets or names built at run time (`ln -s "$(cat f)" x`, `$'\x2eagents'`, `$(printf …)`);
- a script file's links without zsh-fork;
- a command typed across several writes;
- links copied, moved or unpacked (`cp -a`, `mv`, `rsync -l`, `tar`, `git checkout`);
- tools not listed (`awk system()`, `Rscript`, PowerShell `-EncodedCommand`);
- remote environments.

Over-refusal is accepted: text that only looks like a link command (`grep "ln -s" .agents`, a heredoc's
documentation) is refused. So is a link call in code that also names a protected path in some other string.

## Evidence

- Regression tests, fail before and pass after (the non-test source reverted; real Seatbelt on macOS and bubblewrap on
  glitch): `core/tests/suite/issue_626.rs`, `core/tests/suite/issue_623.rs`; unit tests
  `exec_policy::aggressive::tests::*`, `security::protected_links::tests::*`.
- tmux replays (GLM 5.3 Flash, debug build of 79e5ab0c0b, disposable home and HOME): `replay-g11b.txt` (G11b steps: approve "don't ask
  again" under Permissive, switch to Aggressive, same command: asked, with the note; the rule kept) and `replay-g27.txt`
  (AGG-R13 step 6, the `python3 os.symlink` form and an absolute `$PWD` target: each refused, nothing created). Specs:
  `replay-g11b-agg33.toml`, `replay-g27-aggr13.toml`.
- Videos (79e5ab0c0b): [#626](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-626-issue626-remembered-approval-suspended-79e5ab0c0b26-2026-10-13.mp4),
  [#623](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-623-issue623-agents-link-refused-79e5ab0c0b26-2026-10-13.mp4).
