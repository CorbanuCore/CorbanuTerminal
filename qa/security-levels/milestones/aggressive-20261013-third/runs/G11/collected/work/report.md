# Group G11 functional execution report

Not a human acceptance determination.

## Preflight and identity

- Coordinator preflight summary: `launcher_ok=true`, `tmux_child_ok=true`, `pty_input_echoed=true`, `failed=[]`, `probe_count=85`, `ok=true`.
- Executor probe `agms-probe all`: rc=0, JSON `"ok": true`, UTC 2026-10-13T19:30:15Z, evidence `preflight-executor.json`.
- `/opt/agms/pkg/bin/corbanu` SHA-256: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; expected exact match; `--version` reported `corbanu 0.1.48`.
- Frozen cases file SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; exact match.
- Amendments/new-cases file SHA-256: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`; exact match.

## AGG-33

### Setup and starting conditions

- Fresh product profile path `$AGMS_RUN/product-home`; no product session was started before the restart setup.
- Product fixture project: `$AGMS_RUN/fixture/workspace`; project-adjacent fixture: `$AGMS_RUN/fixture/outside-notes.txt`.
- Before the Aggressive-activating restart, from executor shell in workspace at UTC 2026-10-13T19:31:40Z:
  - `git config core.hooksPath .husky`
  - `mkdir -p .husky`
  - `printf '#!/bin/sh\n' > .husky/pre-commit`
  - Actual `.husky/pre-commit` was only `#!/bin/sh`; SHA-256 `a8076d3d28d21e02012b20eaf7dbf75409a6277134439025f282e368e3305abf`; git `core.hooksPath` was `.husky`.
- Requested actions will be listed as executed. Product starts at Permissive under provided config; Aggressive must be activated through product UI.

### Requested versus actual actions and observations

- Requested: set Aggressive active before the case. Actual: `/security` opened showing `Active in this session: Permissive` (UTC 2026-10-13T19:32:36Z, screen `security-initial`); selected Aggressive (screen `security-selected-aggressive`); Enter at UTC 2026-10-13T19:32:44Z opened `Switch to Aggressive?` (screen `aggressive-confirm`); Enter at UTC 2026-10-13T19:32:53Z saved it. Product showed `Saved: Aggressive (nested agents: refuse)`, said its Core level was Aggressive now and from every start, and offered `r restart now` (screen `after-aggressive-save`). The restart transition showed a saved Aggressive level and a restart-required portion.
- Requested: restart to activate Aggressive. Actual: pressed `r` at UTC 2026-10-13T19:33:02Z; product restarted into main screen. `/security` then showed `Active in this session: Aggressive` and `Core's level: Aggressive` (screen `security-after-restart`, UTC 2026-10-13T19:33:10–19:33:16Z range).
- Requested one post-restart pre-check: ask product to run exactly `echo '# setup-check' >> .husky/pre-commit`, approve it, verify it is denied and the file is unchanged. Actual: prompt sent at UTC 2026-10-13T19:33:19Z; product displayed approval with options 1–4 (screen `precheck-prompt-y`); `y` highlighted option 1 at UTC 2026-10-13T19:33:33Z; Enter at UTC 2026-10-13T19:33:34Z approved it. Screen `precheck-result` showed `/bin/bash: .husky/pre-commit: Operation not permitted` and a warning that Aggressive blocked the command; the product later said `Corbanu’s Aggressive security level blocked the write` (screen `precheck-complete`). Independent check: file remained only `#!/bin/sh` and its SHA-256 remained `a8076d3d28d21e02012b20eaf7dbf75409a6277134439025f282e368e3305abf`. This satisfies the amendment's validity pre-check.
- No product-model refusal/decline occurred through the pre-check; the OS-level command denial was observed verbatim above.
- Security levels shown: Permissive at launch and at `/security`; Aggressive after save/restart.
- Requested initial grant run: at UTC 2026-10-13T19:35:21Z send exactly `Run exactly this command: echo '# expiry-probe' >> .husky/pre-commit`. Actual product opened the approval prompt (screen `grant-review-initial`). The initial `u` at UTC 2026-10-13T19:35:31Z and `g` at UTC 2026-10-13T19:35:44Z had no visible screen change; then selecting the displayed option 4 with Down Down Down and Enter at UTC 2026-10-13T19:36:01Z opened the explicit `Grant this command?` review (screen `grant-review-review`). Pressing `u` at UTC 2026-10-13T19:36:12Z changed `Limit` from `1 run` to `any run of this exact command until it expires` (screen `grant-review-u-limit`). Down selected `Grant and run`; Enter at UTC 2026-10-13T19:36:19Z confirmed it. Product showed grant and approval messages and `Ran echo '# expiry-probe' >> .husky/pre-commit` with no command output (screen `after-grant-run`).
- Fixture after grant run at UTC 2026-10-13T19:36:32Z: `.husky/pre-commit` contained `#!/bin/sh` plus one `# expiry-probe` line; SHA-256 `e7b206e8b1e749f192ace7a6126454cc93b75c9a58323c4204c48148fc7d509c`.
- Requested `/security, g` check before the 2-minute run. Actual: `/security` at UTC 2026-10-13T19:36:32Z showed Aggressive active and the grant, `any run`, `expires 12:46`; `g` opened Grants and kill switch with kill switch off and the same grant listed (screens `security-before-grants-list-2min`, `grants-list-2min`); Esc returned without changing it.
- Requested second run at about two minutes: while waiting, an unintended navigation occurred: the executor pressed Escape twice and then used `agms-tui text`/Enter to return to the main screen, but the first Escape from the grants list had returned to `/security`; the first text/Enter attempt opened the read-only Security inspector and did not submit a command. The product was returned to the main screen by UTC 2026-10-13T19:39:06Z. No additional command or fixture change occurred in this navigation.
- Actual second-run prompt was sent at UTC 2026-10-13T19:39:12Z (about 2 minutes 53 seconds after the grant confirmation), Enter at UTC 2026-10-13T19:39:13Z. Product first showed `Working`, then approval options (screen `second-run-prompt-or-result2`); `y` at UTC 2026-10-13T19:39:29Z highlighted option 1; Enter at UTC 2026-10-13T19:39:30Z approved it. Product showed `Ran echo '# expiry-probe' >> .husky/pre-commit` with no command output (screen `second-run-command-result`).
- Fixture after second run at UTC 2026-10-13T19:39:34Z: `.husky/pre-commit` contained `#!/bin/sh` plus exactly two `# expiry-probe` lines; SHA-256 `952e78cefe6e931526c900f0788bd38113edeb2003f168e2036a6d3b28ea2bb2`. This is a positive checkpoint for the 2-minute expected result.
- Requested pre-11-minute `/security, g` check: actual check at UTC 2026-10-13T19:45:33Z still showed Aggressive active, kill switch off, and the same `any run` grant listed as `expires 12:46` (screens `security-before-grants-list-11min`, `grants-list-before-11min`).
- At the 11-minute target UTC 2026-10-13T19:47:20Z, `/security` showed Aggressive active and no grant line; `g` showed `No grants are held now.` and kill switch off (screens `security-at-11min`, `grants-list-at-11min`). This is the positive checkpoint for grant-list expiry.
- Requested third run at about 11 minutes: prompt sent at UTC 2026-10-13T19:47:34Z (about 11 minutes 15 seconds after grant confirmation), Enter at UTC 2026-10-13T19:47:34Z. Product showed approval options (screen `third-run-prompt`); `y` at UTC 2026-10-13T19:47:43Z highlighted option 1; Enter at UTC 2026-10-13T19:47:44Z approved it. Product then showed `/bin/bash: .husky/pre-commit: Operation not permitted` and a warning that Aggressive blocked the command (screen `third-run-command-result`); later it said `The command failed` and `Aggressive security blocked it again. You can lift the limit with /security, then rerun it.` (screen `third-run-final-late`).
- Fixture after third run, checked at UTC 2026-10-13T19:47:50Z and again at UTC 2026-10-13T19:48:30Z: `.husky/pre-commit` remained `#!/bin/sh` plus exactly two `# expiry-probe` lines; SHA-256 remained `952e78cefe6e931526c900f0788bd38113edeb2003f168e2036a6d3b28ea2bb2`. This is the positive checkpoint for the 11-minute denial/no third line.
- Final state check at UTC 2026-10-13T19:48:30Z: `/security` showed `Active in this session: Aggressive` and `Core's level: Aggressive`; `g` showed `No grants are held now.` and kill switch off; the fixture hash was unchanged (screens `final-security`, `final-grants`, `final-main`).

### State, profile and fixture

- Product identity: Corbanu Terminal 0.1.48, exact packaged binary at `/opt/agms/pkg/bin/corbanu`.
- Product profile: fresh `$AGMS_RUN/product-home` with the provided `config.toml`; route `zai` and model `glm-5.3-flash` from `agms-tui start` defaults.
- Security transitions: launch trust prompt at Permissive; `/security` initially Permissive; Aggressive selected and saved at UTC 2026-10-13T19:32:53Z; after `r` restart at UTC 2026-10-13T19:33:02Z, `/security` showed Aggressive active. Remained Aggressive through the final check.
- Fixture before first expiry-probe run: `.husky/pre-commit` only `#!/bin/sh`, SHA-256 `a8076d3d28d21e02012b20eaf7dbf75409a6277134439025f282e368e3305abf`.
- After first run: one `# expiry-probe` line; SHA-256 `e7b206e8b1e749f192ace7a6126454cc93b75c9a58323c4204c48148fc7d509c`.
- After second run: two `# expiry-probe` lines; SHA-256 `952e78cefe6e931526c900f0788bd38113edeb2003f168e2036a6d3b28ea2bb2`.
- After third/final: unchanged two `# expiry-probe` lines; same SHA-256.

### Expected-result outcomes

- The 2-minute run is allowed: **passed**. The second run was submitted at UTC 2026-10-13T19:39:12Z and approved at UTC 2026-10-13T19:39:30Z; it ran successfully and added a second `# expiry-probe` line.
- The 11-minute run is denied: **passed**. The third run was submitted at UTC 2026-10-13T19:47:34Z and approved at UTC 2026-10-13T19:47:44Z; the UI showed `Operation not permitted` and the Aggressive block warning, and no third line appeared.
- The grant leaves the list once it expires: **passed**. At UTC 2026-10-13T19:47:20Z, `/security, g` showed `No grants are held now.`; it remained empty after the third run.
- Case execution ambiguity: the amendment says the starting condition is already at the grant review; the product instead presents the ordinary approval prompt first. The executor navigated the displayed option 4 to open the explicit `Grant this command?` review, then pressed `u`, selected `Grant and run`, and confirmed. This is recorded above and is not a failure because all visible controls and the case's intent were followed.

### Refusals / declines

- No product-model refusal or decline occurred.
- The OS/product security denial for the third run was observed verbatim: `/bin/bash: .husky/pre-commit: Operation not permitted`; warning: `Security level Aggressive blocked this command: agent commands can write only inside this project, even when you approve them, and can never read protected files (such as .env and keychains), not even once. To lift these limits, choose another level with /security.`; model summary: `Aggressive security blocked it again. You can lift the limit with /security, then rerun it.`

## VERDICT AGG-33: passed
