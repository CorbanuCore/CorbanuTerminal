# Screens of the candidate with the local model (2026-10-20)

Text copies of the terminal screen of the exact candidate for this re-run (Corbanu Terminal 0.1.48, the signed build
of main `8bf182de43`, the same as screens-v4), captured by the integrator on 2026-10-20 between 08:04:58Z and 08:09:05Z
in a throwaway copy of the test machine (a Tart clone, deleted afterwards) with a fresh profile and the `ledger-notes`
test project, the product launched with `agms-tui start --route qwen` (the local model), the milestone feature flags on
(protected launch included). Folder paths are shortened to `/Users/tester/run`. These are examples of real behaviour
and wording, not test cases or results; the model's own words vary between runs. No credential or protected file was
touched in this session.

| File | What it shows |
| --- | --- |
| 00-trust | First start in the project: trust prompt |
| 01-start-qwen | After Enter: the header names the local model; status line "... via qwen default" |
| 02-moderate-review | `/security`, Down, Enter: the Moderate review (bottom part) |
| 03-moderate-saved | Enter: Saved: Moderate |
| 04-after-restart | `r`: the restarted session (same model) |
| 05-command-turn | "Run exactly this command: echo shakedown-ok": the metadata warning, the command and the reply |
| 06-spawn-tool-prompt | "Start a child agent with your own agent tools (not a shell command) whose task is: run exactly this command: echo child-ok. Then wait for it and tell me what it reported.": the `spawn_agent_v1` prompt |
| 07-wait-tool-prompt | Enter (Allow once): the child was spawned on the same model; the `wait_agent_v1` prompt |
| 08-child-done | Enter, then Enter on the `close_agent_v1` prompt: the child's result and the closing reply |
| 09-inspector | `/security`, `i`: the inspector's first page (Moderate active, model keys in the credential broker) |
