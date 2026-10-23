# #660 follow-up: the "Nothing restarted" notice names Codex panes

Owner decision (Travis, 2026-10-13): after kill switch-off, a Codex pane with a waiting task shows the same "Nothing restarted…" notice Claude panes show.

## tmux check
GLM 5.3 Flash on zai, disposable home, debug build of `4c36b7f463`.

1. Switch to Aggressive. Spawn a crew: Nazgul Angmar and Codex Troll Burzum.
2. Turn the kill switch on, then send Burzum a task ("A task for Burzum [troll] waits").
3. Turn the kill switch off. The notice reads "Nothing restarted. A task waits for each of these panes … : `Burzum [troll]`."
4. Nothing starts: 10 s later there's no `codex-task.txt` and Burzum is idle in `/spawn status` ([final screen](final-screen.txt)).

Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-660-issue660-codex-switch-off-notice-4c36b7f4630e-2026-10-13.mp4

## Tests
- `app::tests::dispatch_integration::issue_660_switch_off_names_a_codex_pane_whose_task_waits` (real embedded app server). It fails without the fix and passes with it.
- `app::tests::issue_660::issue_660_running_pane_takes_its_held_task_and_is_not_listed`.
