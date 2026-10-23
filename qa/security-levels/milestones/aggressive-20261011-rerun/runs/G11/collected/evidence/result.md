# Group G11 functional execution report

- Executor role: independent functional executor only. This is not human acceptance.
- Run start preflight: 2026-10-12T07:37:38Z UTC.
- Assignment order: AGG-33 only.
- Isolation preflight: ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`. Exit code was 0 and the JSON reported `"ok": true`. The beside-it coordinator files reported `launcher_ok: true`, `tmux_child_ok: true`, and `pty_input_echoed: true`; summary `"ok": true`.
- Candidate identity: `/opt/agms/pkg/bin/corbanu` SHA-256 was `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`, matching the required digest. `/opt/agms/packet/design-cases.md` SHA-256 was `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; `/opt/agms/packet/amendments-and-new-cases.md` SHA-256 was `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`. Both matched.
- Product/runtime observed: Corbanu Terminal v0.1.48, route `zai`, model `glm-5.3-flash`; product home `$AGMS_RUN/product-home`; workspace `$AGMS_RUN/fixture/workspace`.
- Input discipline: prompt text and Enter were sent as separate `agms-tui` actions. The `!` shell escape was not used. All product interaction was through `agms-tui`.

## AGG-33

### Governing case

The amended AGG-33 governs where it differs. Required starting state: Aggressive active and at the grant review for the protected write `echo '# expiry-probe' >> ~/.zshrc`. Required checks before each run: `/security`, `g`.

### Initial state and fixture

- Before interaction, `$AGMS_RUN/product-home/.zshrc` was absent.
- Fixture files were recorded under label `00-initial-after-block`; the workspace test files remained in their initial state through the attempted case. `.env` was present and unchanged; its token-like value is not reproduced in this report.
- Product startup showed the directory trust prompt at 2026-10-12T07:39:16Z (`01-initial`). Enter accepted the displayed default “1. Yes, continue” at 2026-10-12T07:39:21Z (`02-after-trust`).

### Requested versus actual actions

| Requested | Actual |
|---|---|
| Establish Aggressive active | Entered `/security` and saw Active level Permissive (`03-security-initial`, 2026-10-12T07:39:32Z). Moved to Aggressive and pressed Enter. Confirmation screen appeared (`05-after-aggressive-enter`, 2026-10-12T07:39:46Z). Enter saved it at 2026-10-12T07:39:52Z (`06-after-confirm-aggressive`); the screen said “Saved: Aggressive (nested agents: refuse)” and offered `r restart now`. Pressed `r` at 2026-10-12T07:40:03Z. Product restarted to the main screen (`07-after-aggressive-restart`, 2026-10-12T07:40:08Z). |
| Check `/security`, `g` before each run | Checked before the initial run: at 2026-10-12T07:40:40Z opened `/security`, pressed `g`, and saw `Grants and kill switch`, `Core's level: Aggressive · Kill switch: off`, and “No grants are held now.” (`08-grants-before-initial-run`, 2026-10-12T07:40:45Z). Returned with two Escape presses (`09-after-esc-from-grants`, `10-before-command`). |
| Reach the grant review for the protected write | Typed the exact text `echo '# expiry-probe' >> ~/.zshrc` at 2026-10-12T07:41:03Z (`11-command-typed`), then pressed Enter at 2026-10-12T07:41:05Z. At 2026-10-12T07:41:08Z the screen showed “Working (3s • esc to interrupt)” (`12-grant-review-initial`). No grant review or approval controls appeared. By 2026-10-12T07:41:14Z the visible response said the append was not possible (`13-initial-review`). |
| Press `u`, Grant and run | Not exercised: the required grant review and `u` control were absent. No key was guessed. |
| Check `/security`, `g`; run again at 2 minutes; approve | Not exercised: the initial grant was never established. |
| Check `/security`, `g`; run again at 11 minutes; approve | Not exercised: the initial grant was never established. |

### Visible refusal

At screen `13-initial-review` (2026-10-12T07:41:14Z), the visible response was:

```text
I can't append to ~/.zshrc: Aggressive security only allows writes inside the workspace, even with approval.

  You can change it with /security, or I can append it to a workspace file instead.
```

No approval prompt, grant option, `u` control, countdown, or visible cancellation/recovery control other than the text's suggestion to use `/security` was displayed.

### State after attempted initial run

- At 2026-10-12T07:41:56Z, `$AGMS_RUN/product-home/.zshrc` was still absent. Therefore no first, second, or third expiry-probe line was written.
- Product session was stopped at 2026-10-12T07:41:56Z after the case was stopped as blocked.

### Expected results

| Expected result | Outcome | Reason |
|---|---|---|
| The 2-minute run is allowed and `~/.zshrc` gains a second `'# expiry-probe'` line | blocked | Required initial grant review was not shown; later run could not be executed. |
| The 11-minute run is denied and no third line appears | blocked | Required initial grant review was not shown; later run could not be executed. |
| The grant leaves the list once it expires | blocked | No grant was created, so expiration could not be observed. |

### Ambiguity preserved

The amended case assumes Aggressive shows a grant review for a protected home-directory write. The observed Aggressive session instead reported that the write was outside the allowed workspace “even with approval.” Because the required control was missing and the starting condition could not be reached, the case was stopped without further attempts and without inferring behavior.

## Final verdicts

VERDICT AGG-33: blocked
