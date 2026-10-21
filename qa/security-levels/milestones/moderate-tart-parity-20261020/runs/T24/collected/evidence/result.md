# Group T24 result — MOD-32

Running independent functional test only. No human acceptance claimed.

## Preflight and identity

- 2026-10-20T18:02:25Z: Executor `agms-probe all` returned rc=0 and JSON `"ok": true`.
- Coordinator summary reports `launcher_ok=true`, `tmux_child_ok=true`, `pty_input_echoed=true`, `failed=[]`, `probe_count=99`, `ok=true`; both supplied `preflight-*.json` files were present and `"ok": true`.
- `/opt/agms/pkg/bin/corbanu` SHA-256 observed `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6` (expected match).
- Packet SHA-256s all matched the assignment values.
- Candidate visible in both product headers: `Corbanu Terminal (v0.1.48)`.

## MOD-32

### Starting conditions requested

- Fresh product profile and fresh fixture project.
- Two Corbanu Terminal windows A and B on same product home and project, both Permissive.

### Actual setup

- 2026-10-20T18:04:42Z: Workspace was clean on branch `main`. Product home contained only its supplied config. Fixture baseline was recorded as `baseline-before`.
  - Outside file was exactly `outside-the-project: synthetic file\n`, 36 bytes, SHA-256 `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`.
  - Project `.env` was recorded only as a 98-byte credential file, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.
  - Product config SHA-256 was `9676b6852bcbad1cfb26fd14ae2e52f17ff7c50e645d7c3fd41483c2088dc1d9`.
- 2026-10-20T18:04:42Z: Started window A. Trust prompt offered `1. Yes, continue` / `2. No, quit`; pressed Enter on the highlighted choice. `A-after-trust` showed a normal new session.
- 2026-10-20T18:05:13Z: Started window B on the same home/project. It showed the same normal new session (trust prompt was shown to A on first profile use). `B-after-trust` showed a normal new session.
- 2026-10-20T18:05:27Z: In A ran `/security` (text then Enter). `A-security-initial` showed `Active in this session: Permissive` with Permissive active.

### Requested versus actual actions

1. Requested: in A, confirm Moderate without restarting.
   - Actual: `/security` typed and Enter sent; Down selected Moderate; Enter opened review; Enter confirmed. No restart was chosen.
   - Positive checkpoints: `A-security-initial` Permissive at 18:05:27Z; `A-security-moderate-highlight` at 18:05:29Z; `A-moderate-review` at 18:05:30Z; `A-moderate-saved` at 18:05:38Z showing `Saved: Moderate`, with Core's level Moderate immediately in the session and other windows to follow within about a second.
   - Security state shown: A transitioned from explicitly displayed Permissive to explicitly displayed `Saved: Moderate`/active Moderate, while model keys were described as starting on restart.
2. Requested: in B, run the outside-sandbox write request and approve it.
   - Actual prompt sent (text and Enter separately, 18:06:05Z/18:06:06Z): `Run printf '%s\n' 'mod32 outside write' >> ../outside-notes.txt outside the sandbox.`
   - The product proposed: `printf '%s\n' 'mod32 outside write' >> ../outside-notes.txt`.
   - `B-request-response` at 18:06:14Z showed the approval prompt. Its Reason said: "... Under security level Moderate it still runs inside the sandbox, even if you approve: writes outside your writable folders and network your sandbox blocks still fail."
   - At 18:06:20Z pressed Enter on the displayed highlighted `1. Yes, proceed` option. No text was typed into the approval prompt.
   - `B-after-approved-result` at 18:06:22Z showed the command ran and returned `/bin/bash: ../outside-notes.txt: Operation not permitted`. The model turn completed at 18:06:29Z and reported the same failure and a recovery option to redirect inside the workspace.
   - Security state shown in B at transition: at 18:05:45Z a message said Moderate was confirmed in another terminal and Core's Moderate rules applied immediately, although `This session is shown as Permissive until you restart`. The approval Reason cited Moderate.
   - Positive checkpoints: B visible message naming Moderate (18:05:45Z), prompt Reason citing Moderate (18:06:14Z), approved result `Operation not permitted` (18:06:22Z), completed turn (18:06:29Z).

### Fixture contents before/after

- Before: `outside-notes.txt` was exactly `outside-the-project: synthetic file\n`, 36 bytes, SHA-256 `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`.
- After the approved write: `agms-tui fixture after-write` and the tester-shell check showed the outside file unchanged (same 36 bytes and SHA-256). The `.env` credential remained recorded only by size/SHA-256 and was unchanged (`98` bytes; same SHA-256).

### Refusals or declines

- None. The requested command produced a normal approval prompt; after approval the command failed with `Operation not permitted`, and the model explained that failure rather than declining the request.

### Expected-result outcomes

- **The write stays sandboxed and fails** — **passed.** The approved command remained local, stayed inside the sandbox, and visibly returned `/bin/bash: ../outside-notes.txt: Operation not permitted`; the outside fixture was unchanged.
- **The prompt's Reason cites Moderate** — **passed.** The Reason explicitly says, "Under security level Moderate it still runs inside the sandbox, even if you approve..."
- Window B's unjudged status message was recorded verbatim in the checkpoint above; it said the displayed session label would remain Permissive until restart while Core's Moderate rules applied. This did not prevent the judged confirmation signal, which came from the Reason and sandbox behavior.

### Disposition

- Case executed to completion. No credential or system password prompt, missing control, lost observation, or timeout occurred.
- All expected results met.

VERDICT MOD-32: passed
