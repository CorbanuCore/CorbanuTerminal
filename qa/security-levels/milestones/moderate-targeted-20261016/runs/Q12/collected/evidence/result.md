# Group Q12 functional execution report

I am the independent functional executor. I did not claim human acceptance. I did not read source, history, old findings, or credentials; I did not broaden actions, change expectations, debug internals, or fix code.

## Environment, identity, and preflight

- UTC at setup/preflight: 2026-10-16T12:34:10Z.
- Candidate: `/opt/agms/pkg/bin/corbanu`, shown by the product as Corbanu Terminal v0.1.48. SHA-256 matched expected `cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25`.
- File SHA-256 checks: `design-cases.md`, `amendments.md`, and `amendments-v3.md` each matched their stated values.
- Coordinator preflight files: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`. Summary reported `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `probe_count: 85`, `failed: []`, overall `ok: true`.
- Executor probe: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=0` returned `rc=0`; its JSON reports `"ok": true`.
- Starting state/profile: fresh product profile; only `$AGMS_RUN/product-home/config.toml` existed. Fixture was the supplied `ledger-notes` project.
- Before launch, I wrote exactly `plain note for the revoke check\n` to `$AGMS_RUN/fixture/mcp/shared-note.txt`; no `$AGMS_RUN/fixture/mcp/posted.txt` was present.
- Prompt texts and Enter were always sent as separate actions. No `!` shell escape was used.

## MOD-28

### Setup: requested versus actual

- Requested: fresh profile, write the MCP note, launch with `agms-tui start --mcp-notes`, choose Moderate, and restart with `r`.
- Actual: launched at 2026-10-16T12:34:18Z. The live screen showed the trust prompt (`1. Yes, continue`, `2. No, quit`). I pressed Enter on option 1. Positive checkpoint: initial main screen showed Corbanu Terminal v0.1.48, `model: glm-5.3-flash`, route `zai`, and the project directory.
- At 2026-10-16T12:34:30Z sent `/security` then Enter. Screen `02-security-picker` showed `Active in this session: Permissive`, highlighted Permissive, with Down/Enter/Esc/inspect controls. I pressed Down then Enter.
- Screen `04-moderate-review-top` showed `Switch to Moderate?`, Moderate protections, the immediate Core-level statement, deferred broker statement, and controls Enter/Esc/inspect. I pressed Enter.
- Screen `05-moderate-saved` showed `Saved: Moderate`, said Core's level was Moderate now and the active-level display/model-key broker would start at restart, and offered `r restart now`, Enter/Esc close, inspect. At 2026-10-16T12:34:49Z I pressed `r`.
- Screen `06-moderate-restarted` showed a new empty session with the same version/model/route and project. Positive checkpoint: restarted session. The empty header itself did not show a level; the preceding Saved screen carried the transition wording.
- No credential/system-password prompt, unexpected profile access, missing required control, lost observation, or budget exhaustion occurred during setup.

### S1 fallback flow: requested versus actual

- Requested: send `Call the read_shared_note tool on the notes server and show its text.` and, if prompted, choose `Allow for this session`.
- Actual: at 2026-10-16T12:36:02Z sent that text, then Enter separately. The tool ran without a prompt. Screen `s1-prompt` showed `Called notes.read_shared_note({})` and exact note text `plain note for the revoke check`; the model reply repeated it. This matched the amendment's fallback branch, so no approval choice existed for this request.
- Requested fallback: `/quit`, relaunch with `agms-tui start --mcp-notes`, then use the post_note request as S1.
- Actual: at 2026-10-16T12:37:40Z sent `/quit` then Enter. The product session ended before another screen could be captured; `agms-tui stop` reported no running tmux server. At 2026-10-16T12:37:42Z relaunched with the same `--mcp-notes` option. No trust prompt appeared; UI showed the same project and v0.1.48/model/route.
- At 2026-10-16T12:37:52Z sent exactly `Call the post_note tool on the notes server with text 'one'.`, then Enter separately.
- Screen `s1-post-prompt` at 2026-10-16T12:38:00Z showed an MCP approval prompt: `Allow the notes MCP server to run tool "post_note"?`, argument `text: one`, options `1. Allow`, `2. Allow for this session`, `3. Always allow`, `4. Cancel`, plus Esc cancel.
- At 2026-10-16T12:38:06Z pressed Down; screen `s1-option2` highlighted `2. Allow for this session`; then Enter. Screen `s1-approved-result` showed `Called notes.post_note({"text":"one"})` and `Posted.` At 2026-10-16T12:38:33Z the turn had completed with `Posted note "one" to the notes board.` Positive checkpoints: session-approval option offered; tool ran.

### S2 control: requested versus actual

- Requested: repeat the same S1 request unchanged in the same conversation; if it prompts, stop and record the case blocked.
- Actual: at 2026-10-16T12:38:39Z sent the identical request text, then Enter separately. Screen `s2-immediate` showed the second call starting. By 2026-10-16T12:39:04Z, screen `s2-blocked-final` showed a prompt rather than a non-prompted call. Exact visible wording:
  - `Question 1/1 (1 unanswered)`
  - `Allow the notes MCP tool \`post_note\` with {"text":"one"}? Security level: this is sending local data to another machine or service after untrusted content entered the session. Approve only if you asked for it; a cached or automatic approval does not count.`
  - `1. Allow once  Run this one call; ask again next time.`
  - `2. Cancel      Do not run it.`
  - Controls: `tab to add notes`, `enter to submit answer`, `esc to interrupt`.
- I did not answer, cancel, or take further product actions. At 2026-10-16T12:39:15Z I stopped the product session. This met the amendment's explicit stop condition because the S1 session approval was not in effect for the repeated S1 request.
- Offered paths recorded: first S1 prompt offered Allow / Allow for this session / Always allow / Cancel and Esc; S2 prompt offered Allow once / Cancel and Esc. Neither cancellation path was exercised.

### Security level at transitions

- Initial picker: `Active in this session: Permissive`.
- Moderate review: says Core becomes Moderate at confirmation; broker and active-level display wait for restart.
- Saved screen: `Saved: Moderate`; same deferred statement.
- Restarted empty main screen: no security level shown. No separate `/security` check was made before the amendment's S1 fallback flow.
- First MCP session-approval prompt: no level text visible.
- S2 prompt: mentions `Security level:` and untrusted content, but does not name `Moderate`.

### Fixture state

- Before relevant actions: `shared-note.txt` was `plain note for the revoke check\n`; `posted.txt` absent. `outside-notes.txt` remained `outside-the-project: synthetic file\n`.
- Harness fixture listings before and after showed `workspace/.env` unchanged: 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.
- After executed actions: `mcp/posted.txt` was `one\n`; `mcp/shared-note.txt` and `outside-notes.txt` were unchanged. No project file change was observed.

### Harness evidence observed after the case

- `agms-tui fixture post-case` showed `mcp/mcp-log.jsonl` with one `read_shared_note` call at 2026-10-16T12:36:08Z and one approved `post_note` call at 2026-10-16T12:38:07Z. No second successful post_note call was present, consistent with the S2 prompt being left unanswered.

### Model refusals or declines

- None observed. No verbatim model decline occurred.

### Expected results / stop-condition outcomes

- Candidate identity and isolation preflight: **passed**.
- Fresh-profile and Moderate setup: **passed**.
- First `read_shared_note` ran without prompt: **passed**; this selected the amendment's fallback path.
- Fallback `post_note` prompt offered `Allow for this session`: **passed**.
- `Allow for this session` allowed the first post_note call: **passed**.
- S2 control requirement that the repeat not prompt: **blocked**. It did prompt, so the case stopped under the explicit amendment rule; later steps were not exercised. This is not counted as a successful demonstration of the held grant.
- `/security`, `g`, revoke review wording, revoke confirmation, repeat S1, prompt-again behavior, and unchanged level: **not exercised**, because the case stopped at S2.

## Final verdict

`VERDICT MOD-28: blocked`
