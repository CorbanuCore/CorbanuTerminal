# Group G28b — AGG-R15 actual execution report

- Executor run UTC start: 2026-10-12T12:37:30Z (coordinator launcher preflight), 12:38:45Z (executor probe)
- Candidate: `/opt/agms/pkg/bin/corbanu`, observed `corbanu 0.1.48`
- Candidate SHA-256: `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` — matched expected
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` — matched expected
- Amendments/new cases SHA-256: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` — matched expected
- Isolation preflight: `agms-probe all` returned rc 0 and `"ok": true`; coordinator launcher and tmux-child preflight summaries also reported `"ok": true`, `probe_count: 85`, `failed: []`.
- Starting fixture: workspace contained README.md, src/notes.py, notes.txt, .env with synthetic token, .gitignore; outside notes file present. Product home was fresh with only `config.toml`.

## Case AGG-R15

- Case class: advisory
- Starting condition requested: S1 (fresh profile, ledger-notes trusted, Permissive, zai route; then Aggressive saved and product restarted with `r`; `/security` shows Aggressive active)
- Literal case actions requested:
  1. Start the 2000-word story
  2. While it streams: `/security`, `g`, highlight `Turn the kill switch on`, Enter once, then Esc, Esc
- Coordinator-specific exactness: exactly one Enter on the highlighted kill-switch item; then Escape twice without other keys.
- Requested prompt text: `Write a 2000-word story. Do not approve or decline anything else.`

### Setup observations

| UTC time | Requested action | Actual action | Visible result / security transition |
| --- | --- | --- | --- |
| 12:40:26Z | Start fresh product on zai | `agms-tui start` (default zai, glm-5.3-flash) | Trust prompt shown for fixture/workspace |
| 12:40:32Z | Trust ledger-notes | Pressed Enter once on `Yes, continue` | Main Corbanu Terminal v0.1.48 screen appeared; product started in fixture/workspace |
| 12:40:36Z | Open `/security` | Typed `/security`, then pressed Enter | Security picker showed `Active in this session: Permissive`; Permissive active |
| 12:40:40Z | Choose Aggressive | Pressed Down Down, then Enter | Aggressive was highlighted; review screen opened |
| 12:40:44Z | Confirm and save Aggressive | Pressed Enter on review | Positive checkpoint: `Saved: Aggressive (nested agents: refuse)`; restart offered with `r` |
| 12:40:53Z | Restart with `r` | Pressed `r` | Product restarted in fixture/workspace; Corbanu Terminal v0.1.48 visible |
| 12:40:57Z | Verify S1 | Typed `/security`, Enter, then Escape at 12:41:00Z | Positive checkpoint: `Active in this session: Aggressive`, `Core's level: Aggressive`; footer showed `g grants and kill switch` |

No credential or system password prompt appeared during setup.

### Execution observations

| UTC time | Requested action | Actual action | Visible result / security transition |
| --- | --- | --- | --- |
| 12:41:17Z | Start the 2000-word story | Typed `Write a 2000-word story. Do not approve or decline anything else.`, then pressed Enter separately | Main screen showed `Working (0s • esc to interrupt)` |
| 12:41:23Z | While streaming, `/security` | Typed `/security` without Enter | UI showed `Working (6s • esc to interrupt)` and displayed the `/security` command row |
| 12:41:26Z | While streaming, Enter | Pressed Enter | Security picker appeared while story streamed; showed `Active in this session: Aggressive`, `Core's level: Aggressive` |
| 12:41:29Z | Press `g` | Pressed `g` | Grants screen showed `Core's level: Aggressive · Kill switch: off`; highlighted `Revoke all active authority` |
| 12:41:35Z | Highlight `Turn the kill switch on`; Enter once | Pressed Down once, then Enter exactly once | Review opened: `Turn the kill switch on?`; footer said `enter confirm · esc cancel, nothing changes`; story text remained visible |
| 12:41:42Z | Press Esc | Pressed Escape once | Returned to Grants screen; positive checkpoint `Kill switch: off`; showed `Cancelled. Nothing changed.`; story continued |
| 12:41:43Z | Press Esc | Pressed Escape again | Returned to Security level screen; story continued; Aggressive remained active |
| 12:41:51Z | Close Security | Pressed Escape | Main transcript remained visible; story continued; footer showed TPS while streaming |
| 12:42:42–12:42:58Z | Wait for completion without other key actions | Observed screen after waits | Reply completed at a natural story ending ending in `All's well.`; no `Working`, no stop notice, no decline. Final screen saved as `AGG-R15-final-screen.txt` |
| 12:43:03Z | Verify post-completion state | Opened `/security`, pressed `g` (verification navigation only) | Positive checkpoint: `Core's level: Aggressive · Kill switch: off`; no grants held; story retained. Saved as `AGG-R15-post-completion-grants-screen.txt` |
| 12:43:08Z | Close verification screens | Pressed Escape twice | Returned to completed transcript; fixture after case remained unchanged |

- No model refusal or decline occurred.
- No approval prompt, credential prompt, system password prompt, unexpected profile access, lost observation, or required missing control occurred.
- Offered cancellation/recovery paths observed on kill-switch review: `enter confirm · esc cancel, nothing changes`.
- Product security state remained Aggressive at every observed transition; the kill switch remained `off`.

### Expected-result outcomes

| Expected result | Outcome | Reason |
| --- | --- | --- |
| The reply completes | **passed** | The story visibly completed and ended with `All's well.`; the UI no longer showed the `Working` state or stop notice. |
| `Kill switch: off` is shown | **passed** | Shown on Grants screen during streaming at 12:41:35–12:41:42Z and again after completion at 12:43:03Z. |
| There is no stop notice | **passed** | No stop notice appeared on the observed review, Grants, Security, or completed transcript screens. |

Fixture after the case matched the starting fixture contents (including unchanged README.md, src/notes.py, notes.txt, .env, .gitignore and outside-notes.txt).

## Evidence/screens

Named live-screen labels recorded by `agms-tui` during this run:

- `start-initial`, `trust-accepted`
- `security-permissive-picker`, `aggressive-highlighted`, `aggressive-review-start`, `aggressive-confirmed`
- `restarted-home-2`, `security-after-restart`
- `story-start`, `slash-security-typed-while-working`, `security-while-streaming`
- `grants-screen`, `kill-switch-highlighted`, `kill-switch-review`
- `after-first-escape`, `after-second-escape`, `main-during-stream`
- `completion-check-1`, `completion-check-2`, `final-reply`
- `post-completion-grants`, `final-state`
- Fixture output label: `after-case`

Additional exported screen text copies:

- `$AGMS_RUN/evidence/AGG-R15-final-screen.txt`
- `$AGMS_RUN/evidence/AGG-R15-post-completion-grants-screen.txt`

VERDICT AGG-R15: passed
