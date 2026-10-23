The manager is still not on. I added the code change the manager needed to prepare worker actions, but item 2 and the real worker test (item 4) are both blocked by setup I couldn't honestly complete. So the preflight still refused, and I never started the promotion recipe. The owner is unchanged and safe: **armed, generation 1, `fixture-only`**.

## Preflight, item by item (rerun 2026-10-03 05:59Z, exit 2)

| Item | Result | Evidence |
| --- | --- | --- |
| 1. Manager prepares actions through `freeze_worker_inputs` | **Passes** | Commit `66a5b66840` adds `coordinator_cli.py prepare_worker`. It requires an explicit provider, `--yolo` and a cited authority, and runs the freeze before the allocation is registered. I used it once on the live coordinator: allocation `owner-first-check-01` (a read-only repository check in this worktree) was registered as `openai / gpt-6-astra / high / --yolo`, revision 2911 → 2912. The five owner-package files are untouched, so the package digest stays `8021fc53…2065`. Tests: 886 pass, plus 103 owner tests. A first run in the shared venv had 5 import errors because `slack_sdk` was missing there; I kept that log and reran in a fresh venv. |
| 2. A fresh prepared action to hand over | **Not met** | The only checked manager path reads its Claude login from a 0600 file that the operator must create. No such file exists, and I'm not allowed to write a login token into a file. I didn't substitute an unchecked manager or a made-up receipt. The new allocation is waiting for the manager. |
| 3. No unfinished publication write | Passes | — |
| 4. Real isolated worker test | **Not met, not run** | The test machine (`agent@192.168.64.3`) is reachable, but: (a) it has 625 MB free and the staged binary alone is 612 MB, while about 10 GB is held by another track's test runs; (b) it was changed today for the pf83 tests. Its `agent` account can now read a real subscription login and a Z.AI key, and its firewall now allows direct traffic to OpenAI and Z.AI. A run there couldn't honestly be called isolated or proxied. |
| 5. Hand dispatch and publishers quiet | **Not met** | Hand workers were quiet: the coordinator's worker tmux server wasn't running at 05:59Z. The check still fails because it depends on item 2. |

**Independent review:** none ran, because there was no test evidence to review.

**Generation and state:** still generation 1, `fixture-only`, config digest `cd07b20b…d6f5`, no recovery needed. The owner is ticking normally: 43,688 ticks, no hold, 0 errors in a row. Nothing was disarmed, reinstalled or re-armed.

**Dashboard:** republished with exit 0 at commit `66a5b66840`, verified 2026-10-03 06:00:20Z. As last round, the published copy includes the other session's four uncommitted files in `scripts/initiative_control/`. I never staged or edited them.

**Commits (fast-forward pushes to `integrate/management-workstreams-20260911`):**
- `66a5b66840`: the manager change and its tests.
- `291e858b5a`: the extended record `owner-manager-enabled-20261002.md`.

Raw evidence is in `.codex-work/owner-manager-enable-20261002/round2/`.

## Still needed before the manager can be turned on
1. **Login file for the manager:** you or Travis create the manager's Claude login file. The manager then prepares an action on `owner-first-check-01`, re-registers the allocation with `replace: true` (which cancels that first action), and prepares a fresh one. That completes item 2.
2. **Space on the test machine:** whoever owns the pf83 tests frees at least about 2 GB, or another test machine is provided.
3. **A clean test account:** either a dedicated account on the test machine, created with the `neo-vm` admin login, or the machine re-sealed so the test account can't read real logins.
4. **Then the real test:** run the full isolated-test procedure in `owner-limited-113b-return.md`, then the Opus review, the quiet check within 300 s, and the promotion. The new setting will be generation 3.
5. **Login risk:** the test's proxy refreshes the host's OpenAI login in memory only when it is within 300 s of expiring, and never writes the new one back. A refresh during the test would leave `~/.codex/auth.json` holding a used refresh token. Start the test right after the host's own client has refreshed that file.
6. **Still unsettled from round 1:** `slack-receiver-02`.