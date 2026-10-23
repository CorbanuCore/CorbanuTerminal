# G21 functional execution result

Scope: assigned case AGG-32 only. This is independent functional execution, not human acceptance.

## Preflight and candidate identity

- Coordinator isolation preflight files observed: `$AGMS_RUN/evidence/preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`.
- Coordinator preflight top-level result: `"ok": true`; `failed: []`.
- Requested executor command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
- Actual executor preflight: rc=0 at 2026-10-13T21:19:02Z; top-level `"ok": true`.
- Candidate binary: `/opt/agms/pkg/bin/corbanu`; reported version `corbanu 0.1.48`.
- Candidate SHA-256: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; matched the expected value.
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; matched.
- Amendments/new cases SHA-256: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`; matched.

## AGG-32

### Starting conditions / state

- Fresh product profile in `$AGMS_RUN/product-home`; only `config.toml` was present before launch.
- Fresh copy of the test project in `$AGMS_RUN/fixture/workspace`.
- Required readable extra config layer was present: `/etc/codex/config.toml`, mode/owner observed as `-rw-r--r-- root wheel` at 2026-10-13T21:20:40Z.
- Fixture file hashes before action, taken 2026-10-13T21:20:40Z:
  - README.md `5eafd8d8f99086242914e721d88e03d7cef4066402b416d2d6e18cf194a9cc88`
  - notes.txt `67fbf93a8f22aae1b72212f2a170a86071ccefbdca94408af4bc99727f069dae`
  - .gitignore `69aa73cf5bc3377a7d97f2e8d2bb773195815efe225af7f1be6f5b9306bbe14f`
  - .env `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`
  - src/notes.py `8cfc17de788ec38f7c04a5916b5b1e5896691d421b32d92bb922257985b131ec`
- `git status --short` before action: empty.
- Initial UI security state: Permissive (screen label `setup-03-security-picker-permissive`, 2026-10-13T21:21:17Z).

### Setup to reach the requested starting condition

Requested: save Aggressive before the case restart. Actual native/screen actions:

- 2026-10-13T21:20:43Z: trusted the displayed fixture-folder prompt with Enter.
- 2026-10-13T21:21:17Z: typed literal `/security`; observed it in the prompt. Pressed Enter. Picker appeared showing `Active in this session: Permissive`.
- 2026-10-13T21:21:26Z: pressed Down, Down, Enter to highlight and choose Aggressive. The `Switch to Aggressive?` review appeared.
- 2026-10-13T21:21:26Z: pressed Enter to confirm and save. The screen showed `Saved: Aggressive (nested agents: refuse)`, said it starts when restart occurs, and offered `r restart now`.
- No product model declines occurred during setup.

### Requested case actions versus actual

1. **Requested: Restart.** Actual: pressed the displayed `r restart now` control at 2026-10-13T21:21:31Z. The product restarted inside the recorded TUI. The immediate post-restart screen showed the Corbanu Terminal v0.1.48 header (`setup-06-restart-immediate`). It contained no `⚠ Security level Aggressive is enforced...` warning and no `not clean` text.
2. **Requested: Open `/security`.** Actual at 2026-10-13T21:22:12Z: typed literal `/security`, observed it in the prompt, then pressed Enter. The picker showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)` (`setup-09-security-picker-after-restart`).
3. **Requested: Press i.** Actual at 2026-10-13T21:22:14Z: pressed `i`. The Security inspector opened (`setup-10-inspector-top`).
4. Additional neutral navigation within the inspector: pressed Down eight times, then eight more times, to observe the lower sections. This was observation only.

### Positive visible checkpoints

- 2026-10-13T21:21:26Z — `Saved: Aggressive (nested agents: refuse)` was visible (`setup-05-aggressive-saved`).
- 2026-10-13T21:21:31Z — `r restart now` was visibly offered before restart.
- 2026-10-13T21:21:34Z — post-restart screen showed the product header and no startup boundary warning (`setup-06-restart-immediate`).
- 2026-10-13T21:22:12Z — `/security` showed Active and Core level Aggressive (`setup-09-security-picker-after-restart`).
- 2026-10-13T21:22:14Z — `Security inspector (read only)` opened (`setup-10-inspector-top`).
- 2026-10-13T21:22:26Z — lower inspector area showed `[set] Protected boundary: preflight clean at launch; earlier conversations are not resumed` (`setup-12-inspector-scrolled-16`).

### Security level at each transition

- Before save: Permissive was visibly active.
- After save, before restart: screen reported Saved Aggressive and that Core's level was Aggressive now, while the Aggressive session rows would start after restart.
- After restart: `/security` visibly showed `Active in this session: Aggressive` and `Core's level: Aggressive`.
- Inspector: `Saved: Aggressive`; it also showed `Core config: Permissive [configured: config layers]`; `[set] Active this run: Aggressive`; `[on] Live policy: Aggressive in force; next start Aggressive`.

### Inspector observations

- Top inspector reported: `◐ Aggressive set; off or not observed: Credential broker`.
- Runtime controls included `Secretless launch` and `Output gate` observed on, Model key broker and Credential broker unobserved, and Untrusted content on.
- Protected boundary was explicitly reported as clean at launch.
- No visible inspector row named `/etc/codex/config.toml` or described the readable extra config layer as a blocker.

### Fixture after

- 2026-10-13T21:22:43Z: all five recorded workspace hashes matched the before values; `git status --short` remained empty. Outside-notes file remained present. This case requested no fixture mutation.

### Refusals / declines

- None observed from the product's model.

### Expected-result outcomes

| Expected result (amendment governs) | Outcome | Reason |
|---|---|---|
| No startup warning marks the setup `not clean` because of the readable extra config layer | **passed** | Post-restart screen had no such warning. |
| Session starts at Aggressive | **passed** | Post-restart `/security` visibly showed Active and Core level Aggressive. |
| Inspector opens | **passed** | Pressing `i` visibly opened `Security inspector (read only)`. |
| If launch checks still mark it `not clean`, record as advisory inconsistency | **not exercised / advisory observation: clean** | Launch checks reported the Protected boundary clean, so this conditional recording path did not apply. |

Frozen-text comparison: the original expected a named warning for `/etc/codex/config.toml` and a not-clean boundary. The 2026-10-13 amendment supersedes this by expecting no startup warning and treating a remaining not-clean launch check only as advisory. The observed candidate was clean, so no advisory inconsistency against the amendment was present.

### AMBIGUITY / observation note

- The inspector simultaneously showed `Saved: Aggressive`, `Active this run: Aggressive`, and `Core config: Permissive [configured: config layers]`, while the `/security` picker showed `Core's level: Aggressive`. This wording difference was observed but not interpreted beyond the visible text.

## Final verdicts

VERDICT AGG-32: passed
