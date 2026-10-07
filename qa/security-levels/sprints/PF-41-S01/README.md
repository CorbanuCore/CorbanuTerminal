# PF-41-S01: per-sprint gate (2026-10-07)

- **Branch:** `pf-41-s01-inspector` off main at `e4d17dbdc6` (after PF-23-S03, PF-29-S02 #235, PF-24-S02 #253).
  Behind `security_levels`: the inspector is offered only with the `/security` picker, so the flag-off view is
  unchanged. Candidate code commit: `99b5308309`.
- **Model:** GLM 5.2 (`-m glm-5.2 -c model_provider="zai"`) for every TUI run and video. No refusals.

## What the inspector shows

`i` in `/security` opens it; Esc returns to `/security`, `r` reads every fact again, arrows scroll under a pinned
header. No key changes the level, a grant, a revocation or a stop.

| Kind | Rows | Source |
| --- | --- | --- |
| Configured | saved `/security` level, Core `[security] level` from config layers (and a stricter non-user layer) | files |
| Resolved | active level (verified at launch), the session's sandbox, approvals, network, vault and child values, the platform sandbox backend, egress destination policy, launch preflight boundary | config, platform |
| Observed | Core's live policy (level, next start, kill switch, generation), each agent's level (stricter, stopped), held grants (surface, expiry, uses), taint, secretless launch and hardening, output gate, model key broker, recent denials | this process |
| Not contained | MCP servers, hooks, `!` commands, app-server `command/exec` (with or without the environment allowlist) | config, fixed |
| Not available | PF-32 screened search, PF-37 brokered browser login, PF-34 quarantine, PF-40 Agent Sweep | fixed |

Badge: **Protected** only when nothing is degraded, every required control was observed or checked at launch, and the
observation is under 30 s old (the view redraws every second). **Partial** ("set; off or not observed") for off or
unobserved required controls, including a level that comes only from config. **Degraded** for stale facts, a degraded
control, no sandbox backend, an unclean boundary, Core behind the launch level, or Aggressive saved but not yet active.
**Blocked** for the kill switch, unreadable state or a stopped session.

Nothing secret is returned: levels, counters, thread ids, timestamps and fixed text. Grant ids, operations, nonces,
denial paths and commands are not exposed; the boundary row shows only its summary.

## Results

- **Focused (final tree `99b5308309`):** `just test -p codex-core pf_41_s01` 6/6; `just test -p codex-tui pf_41_s01`
  14/14 (three reviewed snapshots).
- **Security filters:** core `security tainted orchestrator taint aggressive launch protected pf_23 pf_30` 324/324;
  TUI `security slash_command` 286/287; `codex-app-server-client` 37/38. Both failures are path-length problems in this
  machine's long `TMPDIR` (a picker message wraps mid-phrase; a Unix socket path exceeds `SUN_LEN`); both pass with
  `TMPDIR=/tmp` and touch no code in this sprint.
- **Clippy:** `just fix` on macOS; Linux `cargo clippy --tests -p codex-core -p codex-tui -p codex-app-server-client
  -- -D warnings` on the RTX box: clean at `3c352e9c75`, `3f9cc8f7e3` and `99b5308309`.
- **tmux run (GLM 5.2, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`):** Permissive inspector; confirm
  Aggressive (Core Aggressive now, session controls after restart: Degraded); restart; Aggressive inspector with live
  policy; model reads a file (taint) and is asked to write `.git/hooks/pre-commit`; Esc declines; the inspector shows
  the session tainted and the declined action. Failure injection: `security_state.json` overwritten, restart →
  Blocked (session stopped); confirm Permissive, restart → Permissive with a live policy.
  - Found and fixed during the run: an Esc on the approval interrupts the turn before the decline was logged, so it was
    missing from denials (now a drop guard records it); a tall inspector lost its header (now pinned and bounded); a
    Core level raised before restart read "enforced" (now Degraded).
- **Review (Opus 5.5 High, installed `corbanu exec`, read-only):** round 1 REQUEST CHANGES (2 blocking, 8 other);
  round 2 APPROVE WITH FIXES (5 low/nit, all applied in `99b5308309`). See `review/round1.md`, `review/round2.md`.
- **Videos (GLM 5.2, commit `99b530830976`):** three, listed in `qa/demos/index/PF-41-S01.md`:
  - live Aggressive inspector after confirm and restart;
  - taint and a declined protected action under Core Moderate from config;
  - injected failure → Blocked → confirm → restart → recovered.

## Known limits

- The network credential broker's and the model key broker's health are not probed, so they are never green.
- Launch-contract denials have no thread and show as "this process".
- A permission-hook refusal on the post-taint path reads "declined (by you or a permission hook)".
- The session's Vault row (PF-24) can say the vault store is readable under Aggressive in a fresh home; that row is
  computed by the existing PF-24 code and was not changed here.
