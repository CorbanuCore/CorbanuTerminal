# TUI runs driven step by step (tmuxlib.sh helpers; text and Enter sent separately; screens captured after each step)

- **D3 under Aggressive** (`captures/s03-tui-spawn-aggressive-d3.txt`): `TUI_ENV="ZAI_API_KEY=<default canary>" tui_start a1-aggr $S/homes/hS --account main --enable security_levels --enable source_envelopes -c security.version=1 -c 'security.level="aggressive"'`.
  Prompt: spawn once with account "work". The account question appeared, and Enter chose "Allow once". The wait-tool security gate then appeared, and Enter allowed it.
  Second prompt: spawn with account "work" again. The generic spawn security gate appeared (Enter), then the account question (Down, Enter = Cancel).
- **Permissive** (`s03-tui-spawn-permissive.txt`): `tui_start a2-perm $S/homes/hS --account main`, the same spawn prompt, and no question.
- **Worker preflight and inheritance** (`s03-tui-worker-preflight.txt`): `TUI_ENV='TMUX=$TMUX TMUX_PANE=$TMUX_PANE' tui_start sp2|sp3 $S/homes/hW --account work`.
  Steps: `/spawn nazgul`, Down, Enter (Bind existing pane), Enter (Main). Then `/spawn troll`, Enter (Corbanu Terminal harness), Enter (glm-5.3-flash), Up, Up, Enter (Low).
  sp2: `corbanu account remove zcmd work` from another shell, then `/spawn troll` again.
  sp3: `/agent`, Down, Enter (the troll), then the pong prompt.
- **Out-of-process workers** (`s03-out-of-process-workers.txt`): `TUI_ENV="ZPX_API_KEY=<default canary> CORBANU_LAUNCHER_WORK_DIR=$S/launcher CORBANU_LAUNCHER_WORKSPACE_DIR=$S/launcher/ws" tui_start c1-workers $S/homes/hP -s danger-full-access -a never`.
  Prompt: run `bash $S/scripts/s03/workers.sh` and echo its output. `$S/launcher` holds a copy of the checked-in `scripts/dev/corbanu-launcher.sh` (run, not read), `bin/corbanu` linked to the candidate, and an `activate.sh` stand-in with no exports.
