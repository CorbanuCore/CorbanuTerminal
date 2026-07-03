# Phase D Status — PFTerminal 0.1.2 Local Install Reset + Fresh-Install QA

## 2026-07-03 09:25 UTC — backup gate

Status: PASS — backup completed before any local install reset.

Directive:

- `/home/pfrpc/repos/orc_directives/tasknodeorc_directive_phase_d_0925.md`

Backup artifacts:

- Source backed up: `/home/pfrpc/.pfterminal`
- Source size before backup: `1.6G`
- Tarball: `/home/pfrpc/pfterminal_backup_20260703/dot-pfterminal-20260703T092534Z.tar.gz`
- Tarball size: `531M`
- Tarball SHA256 file: `/home/pfrpc/pfterminal_backup_20260703/dot-pfterminal-20260703T092534Z.tar.gz.sha256`
- Tarball SHA256: `7752ac0e3308fc16efa4d8b6033a81b020d23f4336922210bdb8130b7a8e7035`
- Contents listing: `/home/pfrpc/pfterminal_backup_20260703/dot-pfterminal-20260703T092534Z.contents.txt`
- Contents listing line count: `8758`
- Installed binary inventory: `/home/pfrpc/pfterminal_backup_20260703/installed-binaries-20260703T092534Z.txt`

Installed binary state before reset:

- `which -a pfterminal` found `/home/pfrpc/.local/bin/pfterminal` and `/home/pfrpc/bin/pfterminal`.
- `/home/pfrpc/.local/bin/pfterminal --version`: `codex-cli 0.0.0`
- `/home/pfrpc/.local/bin/pfterminal0630 --version`: `codex-cli 0.0.0`
- `/home/pfrpc/bin/pfterminal --version`: `codex-cli 0.1.1`
- `/home/pfrpc/.npm-global/bin/codex --version`: `codex-cli 0.142.1`
- `/usr/bin/codex --version`: `codex-cli 0.104.0`

Gate decision:

- Backup gate passed. Proceeding to reset + fresh install through the documented release path.

## 2026-07-03 09:28 UTC — reset + fresh install gate

Status: PASS — old state moved aside and 0.1.2 installed through the release installer.

Move-aside artifacts:

- Move log: `/home/pfrpc/pfterminal_backup_20260703/moved-aside-20260703T092534Z/reset_move_log.txt`
- Moved old `/home/pfrpc/.local/bin/pfterminal` to `/home/pfrpc/pfterminal_backup_20260703/moved-aside-20260703T092534Z/home-local-bin/pfterminal`
- Moved old `/home/pfrpc/.local/bin/pfterminal0630` to `/home/pfrpc/pfterminal_backup_20260703/moved-aside-20260703T092534Z/home-local-bin/pfterminal0630`
- Moved old `/home/pfrpc/bin/pfterminal` symlink to `/home/pfrpc/pfterminal_backup_20260703/moved-aside-20260703T092534Z/home-bin/pfterminal`
- Moved old `/home/pfrpc/.pfterminal` to `/home/pfrpc/.pfterminal.pre012-20260703T092534Z`

Release assets used:

- Release URL: `https://github.com/agtico/PfTerminal/releases/tag/rust-v0.1.2`
- Installer: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/install.sh`
- Linux package: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/pfterminal-package-x86_64-unknown-linux-gnu.tar.gz`
- Checksum manifest: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/pfterminal-package_SHA256SUMS`
- Download checksum verification: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/download_checksum_verify.txt`
- Verification result: `pfterminal-package-x86_64-unknown-linux-gnu.tar.gz: OK`

Install command:

```bash
PFTERMINAL_RELEASE=0.1.2 \
PFTERMINAL_NON_INTERACTIVE=1 \
PFTERMINAL_INSTALL_DIR=/home/pfrpc/.local/bin \
PFTERMINAL_HOME=/home/pfrpc/.pfterminal \
PFTERMINAL_PACKAGE_ARCHIVE=/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/pfterminal-package-x86_64-unknown-linux-gnu.tar.gz \
PFTERMINAL_CHECKSUM_MANIFEST=/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/pfterminal-package_SHA256SUMS \
sh /home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/install.sh --release 0.1.2
```

Install result:

- Install output: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/install_output.txt`
- `which pfterminal`: `/home/pfrpc/.local/bin/pfterminal`
- `pfterminal --version`: `codex-cli 0.1.2`
- Installed launcher: `/home/pfrpc/.local/bin/pfterminal`
- Installed binary: `/home/pfrpc/.pfterminal/packages/standalone/current/bin/pfterminal`

Gate decision:

- Reset + fresh install gate passed. Proceeding to fresh-load TUI QA using the installed 0.1.2 command.

## 2026-07-03 09:35 UTC — fresh-load QA gate

Status: PASS — installed 0.1.2 loads, runs with env-key auth, dispatches, and relaunches cleanly from fresh state.

Evidence:

- Clean rerun evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_d_clean`
- Clean rerun scratch: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_d_fresh_clean`
- Initial harness-contaminated attempt retained for transparency: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_d`
- Harness-contaminated fresh home moved aside: `/home/pfrpc/.pfterminal.phaseD-contaminated-20260703T093246Z`
- Clean reinstall log before rerun: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_install_0.1.2/clean_reinstall_after_harness_contamination.txt`

Clean rerun results:

- PASS: no-auth first launch showed the PFTerminal welcome/provider picker without panic, including OpenAI, Anthropic, Ambient, Z.AI, OpenRouter, Baseten, and Vercel provider choices.
- PASS: env-key launch showed installed `PFTerminal (v0.1.2)` with `zai/glm-5.2-fast` and YOLO mode.
- PASS: basic Vercel env-key turn returned `QA_PHASE_D_BASIC`.
- PASS: spawned Burzum Troll and Snaga Orc on `zai/glm-5.2-fast`.
- PASS: `/spawn status` showed `Nazgul: Codex - Main (current)`, Burzum, and Snaga addressable.
- PASS: targeted Burzum task returned visible child report: `QA_PHASE_D_TROLL_REPORT`.
- PASS: Main remained responsive after dispatch and returned `QA_PHASE_D_MAIN_OK`.
- PASS: TUI exited cleanly after the env-key run.
- PASS: relaunch loaded `PFTerminal (v0.1.2)` without panic and `/spawn status` opened sanely.
- PASS: relaunch exited cleanly.
- PASS: key scan over clean fresh QA evidence, `hit_count=0`.

Notes:

- The first attempted no-auth probe accidentally selected Ambient from the provider picker and treated the typed `/providers` probe as an API-key entry. That state was discarded before the clean rerun and is not used for the fresh-load verdict.
- The clean installed build displays `zai/glm-5.2-fast` without an `xhigh` effort suffix in the status line. The directive did not set a reasoning-effort acceptance criterion, so this is recorded as an observation rather than a Phase D blocker.

Gate decision:

- Fresh-load QA gate passed. Proceeding to restore the backed-up auth/vault state and verify a plan-auth provider.

## 2026-07-03 09:38 UTC — restore usability gate / final Phase D verdict

Status: PASS — backed-up auth/vault state restored and installed 0.1.2 remains active.

Restore method:

- Restore log: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_restore/restore_overlay_log.txt`
- Restored from: `/home/pfrpc/.pfterminal.pre012-20260703T092534Z`
- Restored into: `/home/pfrpc/.pfterminal`
- Command shape: `rsync -a --exclude=/packages/standalone /home/pfrpc/.pfterminal.pre012-20260703T092534Z/ /home/pfrpc/.pfterminal/`
- Rationale: restore backed-up user config/auth/vault/session/plugin state while preserving the freshly installed 0.1.2 standalone package under `/home/pfrpc/.pfterminal/packages/standalone`.

Restored auth/vault paths:

- `/home/pfrpc/.pfterminal/config.toml`
- `/home/pfrpc/.pfterminal/secrets/local.age`
- `/home/pfrpc/.pfterminal/secrets/keyring-fallback/`
- Listing evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_restore/restored_auth_paths_listing.txt`

Plan-auth verification:

- Command route: installed `pfterminal exec`, no provider env keys, `model_provider=ambient`, `model=zai-org/GLM-5.2-FP8`.
- Prompt: `Reply exactly QA_PHASE_D_RESTORE_OK.`
- Exit status: `0`
- Output: `QA_PHASE_D_RESTORE_OK`
- Evidence:
  - `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_restore/ambient_restore_exec.status`
  - `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_restore/ambient_restore_exec.stdout`
  - `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_restore/ambient_restore_exec.stderr`

Final machine state:

- `which pfterminal`: `/home/pfrpc/.local/bin/pfterminal`
- `pfterminal --version`: `codex-cli 0.1.2`
- `/home/pfrpc/.pfterminal/packages/standalone/current/bin/pfterminal --version`: `codex-cli 0.1.2`
- `~/.pfterminal` restored size after overlay: `1.1G`
- Original pre-reset state remains available at `/home/pfrpc/.pfterminal.pre012-20260703T092534Z`
- Backup tar remains available at `/home/pfrpc/pfterminal_backup_20260703/dot-pfterminal-20260703T092534Z.tar.gz`
- No `phase_d_tui` tmux session left running.
- No lingering `pfterminal` or `sleep 120` process found after cleanup.

Key scan:

- Final evidence scan: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_d_key_scan.txt`
- Roots scanned: fresh TUI evidence, clean fresh TUI evidence, install evidence, restore evidence.
- Key values scanned: `42`
- Result: `hit_count=0`

Defects / observations:

- Blocking install or 0.1.2 code defects found: none.
- Observation: clean fresh installed TUI displayed `zai/glm-5.2-fast` without an `xhigh` effort suffix when launched with `-m zai/glm-5.2-fast` and no explicit effort override. This was recorded but not treated as a Phase D blocker because the directive did not define effort display as an acceptance gate.
- Observation: the first no-auth probe accidentally saved a test Ambient credential due to verifier input sequencing. That fresh home was moved aside at `/home/pfrpc/.pfterminal.phaseD-contaminated-20260703T093246Z`, and the clean rerun passed from a newly reinstalled fresh state.

Gate decision:

- Phase D local install reset + fresh-install QA: PASS.
- Machine ended in usable state with restored auth/vault and installed PFTerminal 0.1.2 active.
- Standing by per directive.
