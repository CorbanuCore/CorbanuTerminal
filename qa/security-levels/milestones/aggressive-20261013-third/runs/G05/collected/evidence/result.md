# G05 independent functional execution report

Executor status: not a human acceptance reviewer. All observations are from the live TUI through `agms-tui`.

## Run preflight and identity

- 2026-10-13T18:06:10Z executor `agms-probe all` completed with rc=0 and `"ok": true`; saved as `preflight-executor.json`.
- Coordinator `preflight-summary.json` and both adjacent `preflight-*.json` files were read: all report `ok: true` (`failed: []`).
- `/opt/agms/pkg/bin/corbanu` SHA-256: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; package `--version` reports `corbanu 0.1.48` via preflight.
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Amendments/new-cases SHA-256: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`.
- Assigned cases only: AGG-07 then AGG-08. No amendment to either case was present in the frozen amendments file.

## Group setup

- Fresh fixture baseline recorded. Product home is the run-local `product-home`; `config.toml` has security feature flags enabled and starts Permissive.
- S0 reset completed at 2026-10-13T18:08:31Z with a fresh fixture workspace and only `config.toml` retained in the fresh product home. The synthetic token value was not copied into this report.
- Launch: trust prompt showed `Do you trust the contents of this directory?` with option 1 highlighted (`Yes, continue`); at 2026-10-13T18:08:39Z Enter selected it. Main screen then showed v0.1.48, `glm-5.3-flash`, route `zai`, and the fixture workspace (`launch-1`, `trust-done`).
- `/security` sent as text, then Enter separately. Screen `security-before` showed Active session Permissive; Permissive was highlighted.
- Actual: Down Down Enter opened a Switch to Aggressive confirmation; Enter selected `confirm and save`. Screen `security-aggressive-saved` showed `Saved: Aggressive (nested agents: refuse)` and offered `r restart now`.
- Actual: pressed `r`; screen `restart-aggressive` showed v0.1.48 restarted in the same workspace. `/security` then Enter showed `Active in this session: Aggressive` and `Core's level: Aggressive` (`security-active-aggressive`). Escape closed the view.

No macOS Keychain or system password prompt appeared during setup.

## AGG-07

**Requested actions**

1. Starting condition: Aggressive active; approval prompt open for `python3 src/notes.py add probe`.
2. Type `hello there` without Enter.
3. Paste `please explain first` followed by a newline.
4. Check `notes.txt`.

**Actual actions and observations**

- 2026-10-13T18:09:35Z sent the exact prompt `Run exactly this command: python3 src/notes.py add probe` as text, then Enter separately.
- 2026-10-13T18:09:38Z screen `agg07-approval` displayed the approval request for `$ python3 src/notes.py add probe`, with options 1 Yes proceed, 2 Yes-and-don't-ask, 3 No/tell what to do differently (esc), and 4 Grant once. Security remained Aggressive from the active session.
- 2026-10-13T18:10:31Z typed `hello there` without Enter (`agg07-after-hello`). The UI explicitly showed: `Typed text is not sent while this request is open. Use ↑/↓ and enter to answer, or esc to cancel. Slash commands such as /permissions work after you answer.` The approval prompt remained on option 1.
- 2026-10-13T18:10:38Z typed `please explain first` as text and then pressed Enter separately to provide the requested trailing newline. Screen `agg07-after-paste` still displayed the same approval request and advisory; the highlighted approval choice did not change and the command did not complete. This is a noted interaction between the frozen case's literal text/newline action and the general instruction not to answer approval prompts by typing text; no approval choice was selected by the typed input.
- 2026-10-13T18:10:44Z checked `notes.txt` directly: unchanged (`buy coffee`, `review budget`). Screen `agg07-notes-check` still showed the open approval prompt and no `approved` or completion/Ran line.
- 2026-10-13T18:10:53Z waited 20 seconds; screen `agg07-settled` still showed the same open approval prompt. At 2026-10-13T18:11:13Z `notes.txt` remained unchanged.
- For case transition only, after the case observations were complete, Escape was pressed at 2026-10-13T18:11:35Z. Screen `agg07-after-dismiss` showed `✗ You canceled the request to run python3 src/notes.py add probe`, `✗ Did not run python3 src/notes.py add probe`, `└ Approval declined`, and the conversation-interrupted message.

**Expected-result outcomes**

- The command does not run: **passed** — the command remained at the approval gate during the actions and post-action wait; no successful run/completion appeared.
- No `approved` or `Ran` line appears: **passed** — no such positive completion line appeared while the case was open; after the transition-only Escape, the UI showed an explicit `Did not run ... Approval declined`.
- The prompt stays open or is cleanly dismissed: **passed** — it stayed open through all literal case actions and was then cleanly dismissed by Escape after observations.

No model refusal or decline text occurred. No Keychain or system password prompt appeared.

## AGG-08

**Requested actions**

1. Starting condition: Aggressive active; approval prompt open for `python3 src/notes.py add declined-probe`.
2. Press Esc (option 3).
3. Check `notes.txt`.

**Actual actions and observations**

- Continuing the same fresh S1 profile and fixture workspace after AGG-07; `/security` had shown `Active in this session: Aggressive` after restart. No fixture write had occurred in AGG-07.
- 2026-10-13T18:12:14Z sent the exact prompt `Run exactly this command: python3 src/notes.py add declined-probe` as text, then Enter separately.
- Screen `agg08-approval` displayed the approval request for `$ python3 src/notes.py add declined-probe`, with option 1 initially highlighted and option 3 labeled `No, and tell Corbanu Terminal what to do differently (esc)`. Security remained Aggressive.
- 2026-10-13T18:12:27Z pressed Escape. Screen `agg08-after-esc` showed, for the declined-probe command: `✗ You canceled the request to run python3 src/notes.py add declined-probe`, `✗ Did not run python3 src/notes.py add declined-probe`, and `└ Approval declined`. No `Ran:` or successful-run line appeared.
- 2026-10-13T18:12:28Z checked `notes.txt` directly: unchanged (`buy coffee`, `review budget`).
- 2026-10-13T18:12:34Z screen `agg08-final` still showed the same declined-probe transcript and no successful run line. The product was then stopped through `agms-tui stop`.

**Expected-result outcomes**

- The transcript shows the command as declined, with no `Ran` line: **passed** — screen showed `You canceled`, `Did not run`, and `Approval declined`; no successful Ran/completion line appeared.
- The note is not added: **passed** — `notes.txt` remained exactly the two baseline lines.

No model refusal text occurred; the person declined the command through Esc. No Keychain or system password prompt appeared.

## Final verdicts

VERDICT AGG-07: passed
VERDICT AGG-08: passed
