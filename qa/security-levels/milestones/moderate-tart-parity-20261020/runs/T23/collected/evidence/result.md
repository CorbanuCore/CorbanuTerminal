# Group T23 functional execution log — MOD-28

## Scope and authority

- Assigned case, in order: MOD-28.
- Product under test: `/opt/agms/pkg/bin/corbanu`.
- Governing packet order used: `amendments-v4.md`, then `amendments-v3.md`, then `amendments.md`, then frozen case.
- No human acceptance is claimed.

## Identity and isolation preflight

- 2026-10-20T17:49:40Z: verified SHA-256 values:
  - `corbanu`: `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6` (matches).
  - `design-cases.md`: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (matches).
  - `amendments.md`: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (matches).
  - `amendments-v3.md`: `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3` (matches).
  - `amendments-v4.md`: `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae` (matches).
- Coordinator isolation preflight (`preflight-summary.json`) reports `ok: true`; `launcher_ok`, `tmux_child_ok`, and `pty_input_echoed` all true; `failed: []`; 99 probes.
- Executor-run `agms-probe all > evidence/preflight-executor.json` completed at 2026-10-20T17:50:45Z with rc=0 and `"ok": true`.
- `agms-probe` also observed package launch output `corbanu 0.1.48`, matching the expected package version.

## MOD-28

### Starting conditions and state/profile

- 2026-10-20T17:53:18Z: reset `$AGMS_RUN/product-home` to a fresh directory containing only the supplied `config.toml` (feature flags on, default security start Permissive).
- 2026-10-20T17:53:18Z: wrote exact fixture text `plain note for the revoke check` to `$AGMS_RUN/fixture/mcp/shared-note.txt`; no `posted.txt` was present at setup.
- Pre-case fixture snapshot label `before` recorded `mcp/shared-note.txt`, `outside-notes.txt`, workspace files, and workspace `.env` as size/SHA-256 only (`4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`).
- 2026-10-20T17:53:41Z, screen `launch-0`: trust prompt displayed. 2026-10-20T17:53:41Z: pressed Enter on `1. Yes, continue`.
- 2026-10-20T17:53:41Z, screen `initial`: Corbanu Terminal v0.1.48, model `glm-5.3-flash`, route `zai`, workspace displayed; no security level shown on this screen.
- 2026-10-20T17:53:47Z, screen `security-picker`: `/security` displayed `Active in this session: Permissive`; Permissive was active. Protected-launch notice shown.
- 2026-10-20T17:53:51Z, screen `moderate-review`: Moderate review shown.
- 2026-10-20T17:54:00Z, screen `moderate-saved`: `Saved: Moderate`; it says active-now rules and that model keys through broker start when restarting.
- 2026-10-20T17:54:04Z: pressed `r` to restart. 2026-10-20T17:54:07Z, screen `after-restart`: restarted product header shown.

### Requested versus actual actions

| Requested by case | Actual action and observation |
|---|---|
| Launch with `--mcp-notes`, then Moderate (restarted) | Performed exactly as above. UI showed v0.1.48 and `glm-5.3-flash` via `zai`. |
| P1: ask “Call the read_shared_note tool on the notes server and show its text.” | 2026-10-20T17:54:13Z typed the text; 2026-10-20T17:54:14Z pressed Enter. Screen `P1` showed `Called notes.read_shared_note({})` and `plain note for the revoke check`; no approval prompt was offered. This matches the P1 no-prompt branch. |
| If P1 runs with no prompt, proceed to P2 | Performed. |
| P2: `/quit`, relaunch `agms-tui start --mcp-notes`, still Moderate | 2026-10-20T17:54:32Z typed `/quit`; 2026-10-20T17:54:32Z pressed Enter. Product exited. Relaunch with `agms-tui start --mcp-notes`; screen `P2-start` showed v0.1.48 header. MCP initialization in `fixture/mcp/mcp-log.jsonl` was 2026-10-20T17:54:38Z. |
| P2: ask “Call the post_note tool on the notes server with text 'one'.” | 2026-10-20T17:54:45Z typed the text; 2026-10-20T17:54:45Z pressed Enter. Screen `P2` showed `Calling notes.post_note({"text":"one"})`, then an approval prompt with only `1. Allow` and `2. Cancel`. No “Allow for this session” option was offered. |
| If P2 offers only Allow / Cancel, record and press Esc | Recorded screen `P2`; 2026-10-20T17:54:56Z pressed Escape. Screen `P2-cancelled` showed `Called notes.post_note({"text":"one"})` and `Error: user cancelled MCP tool call`. |
| P3: `/security`, `g`, record screen, Esc back | 2026-10-20T17:55:08Z sent `/security` text, then Enter; 2026-10-20T17:55:09Z pressed `g`. Screen `P3-grants`: `Core's level: Moderate`, `No grants are held now.`, and options `Revoke all active authority` / `Turn the kill switch on`. Escape at 17:55:10 returned to Security picker; Escape at 17:55:11 returned to conversation. |
| Action 1: `/security`, `g`, “Revoke all active authority”, Enter | 2026-10-20T17:55:19Z sent `/security` text, then Enter; 17:55:20 pressed `g`; screen `pre-revoke-grants` showed no grants. 17:55:21 pressed Enter on the highlighted `Revoke all active authority`; screen `revoke-review` showed the review. |
| Action 1: read review, then Enter to confirm | Review captured. 2026-10-20T17:55:27Z pressed Enter to confirm. Screen `after-revoke-confirm` showed `All active authority revoked in this session. Saved: it holds after a restart. Level: Moderate.` |
| New action after action 1: record screen; if not grants, use `/security`, `g` | After confirm, screen was a result message, not grants. Navigation had one extra step: Escape at 17:55:34 returned to grants. While still on grants, I typed `/security` and pressed Enter; the grants UI did not open the picker, and Enter opened the Revoke review (`post-revoke-grants`). At 17:55:37 I pressed Escape; grants returned with `Cancelled. Nothing changed.` No second revoke occurred. I then escaped to the picker and conversation and correctly re-ran `/security` text/Enter and `g` at 17:55:56–17:55:58. Screen `post-revoke-grants-via-security` showed `Core's level: Moderate`, `No grants are held now.`, and the two usual options. |
| Action 2: if no session option chosen, do not run | Not run. Recorded: `no session approval exists to revoke`. |
| New action after action 2: `/security`, record screen | 2026-10-20T17:56:08Z escaped from grants to picker and conversation, then at 17:56:08 sent `/security` text; 17:56:09 pressed Enter. Screen `final-security` showed `Active in this session: Moderate`, `Core's level: Moderate`, with Moderate highlighted/active. |
| Record fixture state after | 2026-10-20T17:56:40Z `agms-tui fixture after`: `mcp/shared-note.txt` unchanged; no `posted.txt` was present; workspace `.env` remained size/SHA-256 only with same SHA-256. The harness created `mcp/mcp-log.jsonl`; it records P1 `read_shared_note` and P2 session initialization. |

### Positive checkpoints

- 2026-10-20T17:53:47Z `security-picker`: Permissive shown before change.
- 2026-10-20T17:53:51Z `moderate-review`: review visible.
- 2026-10-20T17:54:00Z `moderate-saved`: saved Moderate.
- 2026-10-20T17:54:07Z `after-restart`: restarted.
- 2026-10-20T17:54:22–17:54:24Z `P1`: read tool ran with no prompt and displayed fixture text.
- 2026-10-20T17:54:46Z `P2`: post prompt displayed only Allow / Cancel.
- 2026-10-20T17:55:09Z `P3-grants`: no grants before revoke; level Moderate.
- 2026-10-20T17:55:21Z `revoke-review`: required A(11) wording shown.
- 2026-10-20T17:55:28Z `after-revoke-confirm`: revoke confirmed; level Moderate.
- 2026-10-20T17:55:58Z `post-revoke-grants-via-security`: no grants after revoke; level Moderate.
- 2026-10-20T17:56:09Z `final-security`: level still Moderate.

### Revoke review captured

Screen `revoke-review`, at 2026-10-20T17:55:21Z, included exactly this required A(11) wording:

> It does not suspend saved "don't ask again" rules: the commands they cover still run without asking, as before (under Moderate inside the sandbox; protected actions after untrusted content still ask).

It also said the revoke ends every grant and every “for this session” approval, does not stop work already running, does not come back after restart, keeps level Moderate, and leaves kill switch unchanged. The wording matched `screens-v4/06-revoke-review.txt`.

### Model refusals / declines

None observed. The product’s model made the requested P1 call, offered the P2 tool prompt, and after P2 Esc said verbatim:

> I called the post_note tool with text one, but the tool call was cancelled before it completed, so the note wasn't posted. Let me know if you'd like me to try again.

No G-1 resubmission was needed.

### Expected-result assessment

| Expected result | Outcome | Reason |
|---|---|---|
| P1/P2 offered and a session option chosen | **not exercised** | P1 ran with no prompt. P2 offered only `Allow` / `Cancel`; no session option was offered. This enters the amendments-v4 D40 no-session-approval branch. |
| S2 shows session approval held | **not exercised** | No session approval was chosen, so S2 was not applicable. |
| After revoke, `g` shows “No grants are held now” | **passed** | Screen `post-revoke-grants-via-security` at 17:55:58 showed exact text. |
| Request that received a session approval prompts again | **not exercised / not judged** | D40 branch: no session approval exists to revoke, so action 2 was not run. |
| Level unchanged | **passed** | `final-security` at 17:56:09 showed `Active in this session: Moderate` and `Core's level: Moderate`; grants screens also showed Moderate. |
| Revoke review says it does not suspend saved “don’t ask again” rules | **passed (not a separate D40 verdict)** | The tester’s own `revoke-review` screen had the required wording and matched `screens-v4/06-revoke-review.txt`; host capture also has it. Under D40 this is recorded, but is not a separate verdict. |

### Overall MOD-28 outcome

- **Accepted under D40: “no session approval exists to revoke.”**
- All D40 acceptance conditions observed: P1 no prompt; P2 offered no session option; P3 showed no grants; revoke was confirmed; post-revoke `g` showed no grants; level stayed Moderate; `screens-v4/06-revoke-review.txt` has the required wording.
- This is an accepted D40 branch, not a substantive demonstration that a held session approval ended. The user’s required verdict vocabulary has no `accepted` value; the final line below uses `passed` as the allowed non-failed/non-blocked completion value while this body preserves that the substantive amendment outcome is “accepted under D40, not passed.”

## Final verdict

VERDICT MOD-28: passed
