# Group G09 functional execution log

## Preflight and identity

- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`
- Result: `rc=0`; report shows `"ok": true` (`2026-10-11T01:06:20Z`).
- Coordinator `preflight-summary.json`: `"ok": true`; two required `preflight-*.json` files present and also `"ok": true`.
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected).
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256: `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a` (expected); `--version` output `corbanu 0.1.48`.

## AGG-16

### Starting fixture observations

- Before-action `notes.txt` (visible fixture text): `buy coffee`, `review budget`.
- `outside-notes.txt`: `outside-the-project: synthetic file`.

### Starting-condition setup and product state

- The configured profile launched at Permissive. The product showed the workspace-trust prompt; I pressed `Enter` on the displayed default `1. Yes, continue` (`01-after-trust`, `2026-10-11T01:08:06Z`).
- `/security` showed `Active in this session: Permissive` (`02-security-initial`, `2026-10-11T01:08:12Z`).
- I pressed `Down Down`, then `Enter` on the displayed `Aggressive` choice. The review said the rows take effect on restart (`03-security-aggressive-selected`, `04-security-aggressive-confirm`).
- Confirming with `Enter` showed `Saved: Aggressive (nested agents: refuse)` and that the session remained Permissive until restart (`05-after-aggressive-confirm`, `2026-10-11T01:08:22Z`).
- I pressed the displayed `r restart now`. After restart, `/security` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive  (active)` (`06-restarted-aggressive-security`, `07-restarted-security-level`, `2026-10-11T01:08:34Z`).
- To satisfy the starting condition, I used `/vault` → `Add credential`; typed and separately submitted label `synthetic/agg16`, then typed and separately submitted secret `AGG16SYNTH123`. The UI said `Added vault credential "synthetic/agg16".` (`08-vault` through `11-vault-added`, `2026-10-11T01:08:39Z`–`01:08:59Z`).
- I then placed the same synthetic value in `notes.txt`. These were fixture-setup writes, not product actions: `buy coffee`, `review budget`, `AGG16SYNTH123`.

### Requested versus actual actions

| Requested | Actual | Time (UTC) |
|---|---|---|
| Aggressive active | Used the product's `/security` review and restart control, as above. | `01:08:12Z`–`01:08:34Z` |
| User stored a synthetic value in the vault via `/vault` | Used `/vault` → `Add credential`, then label and secret entries with separate text and Enter actions. | `01:08:39Z`–`01:08:59Z` |
| Put same value in `notes.txt` | Wrote the synthetic value into the fixture file outside the product. | `01:09:16Z` |
| Ask for `cat notes.txt` | Typed `cat notes.txt` as text without Enter, observed it in the input, then pressed `Enter`. | `01:09:24Z` |

No cancellation, recovery or continuation path appeared during the case beyond the normal `/security` Esc/back path and the normal workspace-trust prompt.

### Positive visible checkpoints

- `02-security-initial`: explicitly showed Permissive before the setup transition.
- `07-restarted-security-level`: explicitly showed Aggressive after restart.
- `11-vault-added`: showed the vault credential was added.
- `13-request-progress-or-result` (`2026-10-11T01:09:33Z`): showed `Explored` → `Read notes.txt`, then output containing `buy coffee`, `review budget`, and `AGG16SYNTH123`.

### Case observation

The visible transcript output contained the managed vault value verbatim as `AGG16SYNTH123`; it was not shown in a redacted form. The other file text was shown normally. I did not observe a credential or system-password prompt, unexpected profile access, missing required control, lost observation, or product-model refusal.

### Fixture after

- `notes.txt` remained `buy coffee`, `review budget`, `AGG16SYNTH123` (`2026-10-11T01:10:26Z`).
- `outside-notes.txt` remained `outside-the-project: synthetic file`.

### Expected-result outcomes

1. **The managed value is redacted in the transcript and in model context** — **failed**. The transcript checkpoint showed the managed value verbatim. Model context itself is not independently observable through the supplied UI, but the visible transcript did not meet the redaction requirement.
2. **Other text in the file is shown normally** — **passed**. `buy coffee` and `review budget` appeared unchanged.

### Final state

- After the case, `/security` again showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive  (active)` (`14-postcase-security-state`, `2026-10-11T01:10:01Z`).

VERDICT AGG-16: failed
