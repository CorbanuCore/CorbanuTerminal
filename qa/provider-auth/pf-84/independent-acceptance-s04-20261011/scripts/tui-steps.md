# TUI sessions (helpers from `lib.sh`; text and Enter sent separately; screens captured after each step)

Homes live under `$S/homes`. `tui_start <session> <home> [args]` runs the binary `$B` with `env -i`, a fake `HOME`,
`CORBANU_TEST_NO_NATIVE_KEYRING=1` and a disposable `CORBANU_HOME`. Real keys go in with `paste_secret`, which
fetches them inside its own command and pastes them through a tmux buffer deleted on paste. Synthetic canaries
use `paste_value`. Both use bracketed paste (`paste-buffer -p`) once #461 was found; before that, plain paste.

| Session | Home | What was done |
| --- | --- | --- |
| z1 | hZ (fresh, flag on) | Onboarding: Z.AI with the real key, then the one Z.AI row, chooser, Esc, Add another account: Esc at name, invalid name, Esc at key (nothing saved), `obfake` saved, duplicate `obfake` refused, Done. |
| z2 | hZ | pong on default; /status; /providers rows; Add another account `fake` (Esc at key: nothing saved; then saved); duplicate `fake` refused; Esc in account actions, Rename and Remove (hashes unchanged); Use for this session `fake` → 401 naming it; Make default `fake`; Rename refused while in use. |
| z3 | hZ (restart) | /status `account fake`, 401 (durable); Use default account for this session → pong, config still `fake`; Rename `fake`→`spare` (default follows, fp kept); Remove `spare` → replacement prompt → `obfake`; Remove `obfake` → replace with `default`, config cleared. |
| p1–p5 | hP (custom `zpx` → canary proxy → real Z.AI) | Add `main`, `work` (masked); Use `main` → proxy sees `pmain`; Make default `work`, restart → `pwork`; Use default → `pdefault`; add `fake` → one `pfake` 401; Rename `main`→`spare` → still `pmain`; Remove `spare` in use → replace with `work`; Remove `fake` (Keep it first); restart → rows and `work` persist; stale row after CLI removal refused; CLI removal of the default account → TUI exits before start (0 requests); `quota` with a typed `q` value saved. |
| k1, k2 | hK (Kimi, real key) | Onboarding (pre-existing loop, #462); pong; add `fake`; Use for this session → /status `account fake`, 401 naming it; back to default → pong. |
| c1–c3 | hC, hC2 (Claude Plan) | Onboarding token (real) → one "Provider: Claude Account" row, chooser, Add another account `work` (real token) and `fake` (synthetic); c1 hit #461. /providers: rows, `work` pong, `fake` 401 naming it, Claude Code login `cfg` added (path), OpenAI API key set up: no Add another account. |
| z4 | hZ | Add `main` (real key) and `fake`; Use `main`; spawn without account → `· account main`, child-ok; spawn with `account "fake"` → `· account fake`, child 401. |
| f1–f3 | hZ, hF3 | Flag off: one Z.AI row, no account actions, no `/status` suffix, `[provider_accounts]` ignored with a warning; onboarding chooser without Add another account. |
| b1 | hB1 (baseline) | Same Claude onboarding on `2f95b4fc9a`: two configured rows. |
| m0–m3, bm | hM | Baseline-onboarded Z.AI home; candidate flag off/on, add/remove a named account; hashes. |
| q1–q7 | hQ* | Reproductions for #461. |
