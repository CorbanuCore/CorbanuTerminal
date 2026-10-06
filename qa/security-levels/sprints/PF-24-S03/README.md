# PF-24-S03 evidence: flagged `/security` picker

Product heading: **P0 `/security` levels** — “Existing approval, sandbox, vault,
wallet, tool, network, and agent policies are unchanged.” Candidate commit
`72735863b6` on `codex/pf-24-s03-security-picker` (base `24242c1b0e`).

## What shipped

- `security_levels` flag (`UnderDevelopment`, off). With no state file nothing
  changes; flag off shows the old read-only `/security` view.
- Picker: Permissive, Moderate (“not available yet”), Aggressive. Enter shows
  the differences; only the accept key saves; Esc changes nothing.
- The level is stored in `$CODEX_HOME/security_level.toml`, outside every
  config layer. It takes effect at the next start: the launch path adds the
  Aggressive overrides at the highest precedence, verifies every row in the
  loaded config (and on every later config build), and refuses to start if a
  row is missing. Unknown or corrupt state enforces Aggressive with a warning.
- `/status` shows the active level and any saved change.

## Aggressive mapping as built

| Row | Existing control |
| --- | --- |
| Sandbox | Permission profile `corbanu-aggressive`: extends `:workspace`, `/tmp` and `$TMPDIR` read-only, Corbanu home read-only, network off. Its denied-read entries make unsandboxed retries impossible, so approved commands stay inside (a legacy `SandboxPolicy` plus `untrusted` retries an approved, sandbox-blocked command outside the sandbox without asking: `tmux-run/` manual1 finding, core `orchestrator.rs`). `request_permissions_tool` and `exec_permission_approvals` off. Launch flags (`--sandbox`, `--yolo`, `--add-dir`) are overridden. |
| Approvals | `approval_policy = "untrusted"`, `approvals_reviewer = "user"`; `/permissions` and auto-review changes refused while Aggressive is stored or active. |
| Network | Profile network off; `web_search = "disabled"`. |
| Vault | Exec-policy rule file `rules/corbanu-security-aggressive.rules` forbids `corbanu|codex|pfterminal[-debug] vault`; deny-read on `$CODEX_HOME/secrets` and `auth.json`; `shell_environment_policy` default excludes on plus `*VAULT*`, `*PASSWORD*`, `*PASSPHRASE*`, `*CREDENTIAL*`; `features.shell_snapshot = false` and `allow_login_shell = false` so nothing re-exports removed variables. |
| Children | Existing inheritance of the session config and turn permissions; custom roles that would change a row are refused at start. |

Remaining residuals (disclosed in the review screen or the sprint record):
MCP servers, apps and hooks run outside the sandbox; `corbanu exec` and IDE
sessions ignore the level; a malformed `.rules` file elsewhere drops the prefix
rule (the deny-read still blocks vault reads); vault stores outside the active
Corbanu home are not covered.

Follow-ups outside this sprint: verify the loaded exec policy (needs
`codex-execpolicy` as a TUI dependency); reapply env and web-search fields for
role-spawned children in core; cover `corbanu exec` and IDE sessions; state-file
tamper evidence and “restart now” (PF-24-S02); show the user's current value
beside each row (code-blind ambiguity B-06); the static `/security` command
description still says “read only” when the flag is on.

## Gate

| Item | Result | Evidence |
| --- | --- | --- |
| Focused tests | `just test -p codex-tui -- security`: 31/31 pass; `just test -p codex-features`: 33/33 pass | `tests/summary.md` |
| Full `codex-tui` | 4207 pass, 21 fail: host-only `SUN_LEN` socket paths and a command-popup snapshot on unchanged strings, none in touched code | `tests/summary.md` |
| tmux functional run (GLM 5.2) | pass: flag off, review/cancel, select, restart, probes denied after approval (outside, `/tmp`, network, vault, vault store, env, level file), `/permissions` blocked, child inherits, back to Permissive (config.toml hash unchanged), unknown value, launch flags ignored, resume both ways | `tmux-run/` |
| Independent review (Opus 5.5 High) | request changes → 12 findings dispositioned, fixes in `72735863b6` | `review/` |
| Videos (demo SOP) | 9 videos, all at `72735863b6` | `demos/index.md`, specs and wrappers in `demos/` |
| Code-blind design | 49 cases frozen before any result was shared; designer made no tool calls | `code-blind-design/` (`FROZEN.sha256`) |

Pre-existing product finding (not this sprint): with `RUST_LOG=trace` the
provider key appears in the private TUI log and log database; the demo tool
redacted it in place (`PRODUCT FINDING` in every run).

The tmux run (`tmux-run/`) used commit `b8e5f59f7a`; the videos re-prove the
core flows on `72735863b6`. Later commits only merge `origin/main` and add an
argument comment required by the Windows lint (`e82b6dd98b`).
