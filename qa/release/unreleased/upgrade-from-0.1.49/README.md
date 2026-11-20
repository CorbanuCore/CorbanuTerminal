# Upgrade compatibility: released 0.1.49 to the sync candidate

Date: 2026-11-20 UTC. Landing-plan gate 6 (compatibility) for the upstream sync
(`sync/upstream-rust-v0.167.0`), #2812, #1632. **Upgrade passed every check.
Running 0.1.49 on the upgraded home refuses the state DB, as documented.
Backup-and-restore rollback passed.** The rollback policy itself is owner
decision D745; this record only tests the path the "Upgrading from 0.1.49"
release note describes.

## Inputs

- Old: published `rust-v0.1.49` (`776b4364e7`), Linux x86_64 GNU package,
  verified against the published `corbanu-terminal-package_SHA256SUMS`.
  `bin/corbanu` SHA-256
  `b8614f0be29edef2b73255939eae77652ef7d138f4c6939b3f5c7ea09e0eee10`.
- New: sync head `31f2b95bd145e1a14b7a0c45c9965cb775cefc44`, the
  `corbanu-terminal-package-linux-x64-smoke` artifact of CI run 44664456447
  (`dev-small` profile; package SHA-256
  `53bb58f64ecd526d7a9f13d6966effd9f34fa7103dc67209d9457dc172586253`,
  `bin/corbanu`
  `76dcdba0e9d2116bc18d422160f82ef45145c7a9ec7e3bdd6b488453c08fb1fb`). It
  reports 0.1.48, the sync workspace version.
- Host glitch, in Docker (`run.sh`, image from `Dockerfile`; this run used the
  identical existing `corbanu-astra-rel149-qa` image): `--network none --cap-drop ALL`,
  no-new-privileges, 4 GiB and 256-PID caps, host UID, only the work
  directory mounted. The harness asserts that no host home, user socket or
  Docker socket is visible. No keyring service: the vault key is the
  file-backed fallback in the home. A local mock Responses server answers
  every model request. All credential values are random fixture strings,
  compared in memory and never printed or written to results; `summarize`
  refuses to write a result file containing one. The vault value is typed into
  the TUI, so the raw `evidence/` screen captures stay out of the repository.

## What the run does (`upgrade_from_release.py`)

1. **baseline (0.1.49):** two `exec` sessions; a TUI session that adds the vault
   credential `upgrade-canary` with `/vault credential add`; named accounts
   `zai/personal` and `anthropic/work` (API keys) and `claude-plan/team`
   (Claude token); an API-key login; `corbanu archive` of the first session;
   two memory files. Every DB is checkpointed, then the whole stopped home is
   copied as the backup (the release note's step).
2. **upgrade (candidate, same home):** `exec resume` of both live sessions, TUI
   `resume --last`, credential checks, a second start, and a fresh candidate home
   for schema comparison.
3. **downgrade (0.1.49 on a copy of the upgraded home):** TUI start and `exec`.
4. **rollback:** move the upgraded home aside, restore the backup, then
   0.1.49 `exec resume` of both sessions, TUI `resume --last` and credential
   checks.
5. **summarize:** writes `results/*.json` from the raw phase results.

## Results

| Check | Result |
| --- | --- |
| State migrations | 1–53 and 70 stay byte-identical (version, name, checksum); exactly 54–69 are added, all successful, applied after 70 |
| Other DBs | goals 1–2, logs 1–2 unchanged; memories gains 2, queue DB created; `integrity_check = ok` on these five |
| Schema | The upgraded home has the same six DBs as a fresh candidate home (adding thread history), with equal schemas |
| Second start | No migration added |
| Sessions | Same three threads with the same archived flags; both live sessions resume, each rollout an unchanged prefix of the new one; the resumed TUI shows the old prompt, then answers |
| Archive | Archived rollout kept and still archived |
| Vault | `vault auth-helper upgrade-canary` returns the stored value; `secrets/` byte-identical; no vault value in clear text in the home |
| Named accounts | All three listed with kinds, values never shown |
| Login | `login status` still reports the API-key login |
| Memories | `memories/` files byte-identical |
| Config | Every original line kept; the candidate added `service_tier = "default"` (as in 0.1.49, disclosed) and `[tui] screen_reader_detection_done = true` |
| 0.1.49 on the upgraded home | TUI stops: "local database appears to be damaged"; its technical detail names exactly foreign migrations 54–69. `exec` logs the same refusal as a warning and runs without the state DB (the turn completes; no thread row is added). State DB untouched |
| Backup restore | Restored files equal the baseline; 0.1.49 resumes both sessions and the TUI session; migrations back to 1–53 and 70; vault, accounts and login intact |

Results per phase: `results/*.json` (written by `summarize`; the bulk file list
is reduced to the checked paths' digests, checksums to 16-hex-digit prefixes).

## Automated coverage

`codex-rs/state/tests/state_db_upgrade.rs`
(`release_0_1_49_state_db_upgrades_without_changing_applied_migrations`) runs in
CI with a state DB and memories DB written by the 0.1.49 release code. See
`codex-rs/state/MIGRATIONS.md` for how the fixture was made. The vault file
format has its own never-regenerate fixtures (D319,
`codex-rs/secrets/tests/d319_vault_compat.rs`); 0.1.49 did not change it.

## Limits

Linux only, file-backed vault key, a deterministic provider and a CI
`dev-small` build of the PR's base (this PR changes only tests and docs) rather
than a release package. It does not cover the macOS
Keychain, Windows Credential Manager, installer-in-place upgrade or real
providers. Repeat it on the final candidate: put the old and new packages in
`old/` and `new/` of a fresh directory with this script and `run.sh`, then
run the five phases in order. The expected migration numbers (0054–0069 added
after 0070) are specific to this upgrade.
