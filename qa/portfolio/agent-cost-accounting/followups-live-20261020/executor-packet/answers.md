# Integrator's answers to the designer's questions (facility facts only), 2026-10-20

Written after the designer's proposal was saved unchanged (`designer-output.md`). Facts about the test environment and
the product's controls; they add no expected results. Harness extensions are listed in the tester's `agms-tui help`.

1. **Moonshot spend:** pending with the owner (asked 2026-10-20 as D54: either no Moonshot turn at all, or at most one
   first turn and one second turn per Kimi model). The tester's run brief contains a file `d3.txt` stating which. The
   relay enforces it: after the allowed number of Moonshot requests it answers `test harness: Moonshot request limit
   reached`; with no grant, every Moonshot request is refused. One turn can make more than one request; never retry a
   Moonshot turn.
2. **Route and model:** chosen at start: `agms-tui start --route R --model M [PRODUCT ARGS]` (models listed in
   `agms-tui help`). To change model, stop and start again with another `--model` on the same profile (a new
   conversation unless `resume ...` is given). Whether `/model` switches models on the `moonshot` route is not known.
3. **Prices:** the packet has no rates for `gpt-6-luna` or `glm-5.3-flash`; none will be supplied.
4–5. No information beyond the packet.
6. **`/side`:** opened with `/side` (Enter). Ctrl+C once (`agms-tui key C-c`) leaves the side conversation and returns
   to the saved one; the product says so on screen. Esc in an empty composer inside `/side` does not leave it. Whether
   `/cost` works inside `/side` is not known.
7. **Editing an earlier message:** no information beyond the packet; use the keys and hints the product shows.
8–10. No information beyond the packet.
11. No information beyond the packet.
12. **Realtime conversation:** the realtime command prints the thread ID it used. The product can be started on any
    route with `resume <id>`; `-c skip_reopen_warmup=true` can be added at start. Whether that conversation can be
    opened is not known in advance. Before its first session the realtime command signs the group's profile in with
    an OpenAI API key (a synthetic placeholder; the real key is added outside the tester's reach).
13. **Realtime spend:** the relay allows 2 realtime connections in the whole run (one session plus one spare for a
    harness failure); a refused connection shows as a connection error in the command's output. Its usage is logged by
    the relay for the evidence check, not shown to the tester.
14. No information beyond the packet.
15. Correct: only the provider name `kimi` is available.
16. **`config.toml`:** `agms-tui config 'key = value'` appends one top-level line to the profile's `config.toml`
    (takes effect at the next start). There is no "remove" control.
17. No information beyond the packet.
18. **UTC day:** the run is planned between about 03:00 and 08:00 UTC on 2026-10-20, so groups stay within one UTC
    day. Record UTC times anyway.

Other facts:

- **Timing:** `/cost` can read the ledger a moment before the latest request is written. Its page has a `Refresh`
  entry; re-opening `/cost` also re-reads.
- **Exec:** `agms-tui exec [--route R --model M] [--ephemeral] 'prompt'` runs `corbanu exec` once on the same profile
  and project and prints its output (route `zai` by default).
- **Workspace:** `$AGMS_RUN/fixture/workspace` is a small git project (README.md, src/notes.py, notes.txt,
  .gitignore and a .env with a synthetic value). The tester may edit tracked files there with ordinary shell commands
  (e.g. `printf 'Anoter line for review.\n' >> README.md`).
- **Group profiles:** each group gets a fresh profile and project; nothing carries over between groups. Groups run one
  at a time, each with a fresh tester session and a wall-clock budget of about 80 minutes.
- **Screens:** `agms-tui screen` prints the visible terminal (150 × 45). The tester also has `agms-tui wait REGEX`.
