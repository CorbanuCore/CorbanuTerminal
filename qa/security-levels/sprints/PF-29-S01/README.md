# PF-29-S01 evidence: protected-mode inventory and activation preflight

Behind the new default-off flag `protected_mode_preflight`, effective only with `security_levels` (the flagged
`/security` picker). Flag off is unchanged. Base `4b42012daa`; worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/pf-29-s01-20261006`. Started with PF-28-S02 merged behind
`secret_output_gate` (#216, recorded as `merged_behind_flag`) and PF-20-S02 archived.

## What the flag does

| Step | Where | Behaviour |
| --- | --- | --- |
| Inventory | `core/src/security/inventory.rs` | Fixed locations only, no recursive scan, nothing written, values never kept or shown. Findings carry a stable ID (kind + location digest), a class and a disposition (denied, isolate, migrate, remove from context, not contained, info). Regular files only, opened non-blocking and read up to 1 MiB, so a `.env` linked to `/dev/zero` or a named pipe cannot hang it. |
| Readiness | `core/src/security/preflight.rs` | OS sandbox, `secretless_agent_launch`, `isolated_credential_broker`, `secret_output_gate`, a readable vault store, and no provider `auth` command left unchecked (consent required). |
| Transition | `tui/src/bottom_pane/security_level_picker.rs` | The Aggressive review shows readiness, blockers by location, what launch will deny, routes that are not contained and the limits. Blockers keep Aggressive unsaved. Confirm reruns the preflight with the same key and refuses on any drift. A pass writes `security_preflight.toml` (finding IDs and times only). |
| Isolation | `tui/src/security/{launch,preflight,aggressive}.rs` | While Aggressive is stored after a preflight, launch adds deny-read entries to the Aggressive profile for every credential path found: whole `~/.ssh`, `~/.aws`, `~/.kube`, `~/.config/gcloud`, `~/.azure` folders (so `config`, `known_hosts` and `*.pub` there are denied too), CLI token files, browser profiles, keychains, project env files, Corbanu sign-in, wallet, sessions, history, snapshots and logs (denied, and their folders created, before startup creates them), and every state database with its `-wal`/`-shm` files through one `*.sqlite*` glob (exact paths when the home path has glob syntax). Each denial is verified before Aggressive is shown active. This closes the PF-27-S02 known limit for `~/.ssh` and other credential files. |
| Re-audit | `tui/src/security/preflight.rs` | Every Aggressive launch reruns the full preflight. Not clean: a startup warning lists the blockers and `/status` says "protected boundary not clean"; nothing claims a clean boundary. |
| Clean context | `tui/src/app_server_session.rs` and the startup and `/resume` paths | Resume or fork of a conversation created before activation (UUIDv7 time against `activated_at`, ms) is refused; a missing, old-version or corrupt receipt refuses every resume. |

Flag off: no preflight, no receipt, no extra denials (control video). A receipt left from a flag-on save only adds
denials while Aggressive is stored; choosing Permissive removes it.

## Tests (final tree)

- `cargo test -p codex-core --lib pf_29_s01`: 14 passed (symlinks, shadowed config and env, old memories, denied
  reads, locked or damaged vault, corrupt snapshots, drift, keyed digests, no recursion, devices and pipes).
- `cargo test -p codex-tui pf_29_s01`: 8 passed (receipt versions, isolation in the profile and the glob-syntax
  home, ms resume refusal, blocked save, clean save, drift refusal, saved Aggressive without a receipt, Permissive
  removes the receipt). `cargo test -p codex-tui security` after merging main: 62 unit and 2 tmux tests passed.
- Suites: `just test -p codex-tui` 4267 passed, 21 failed: 20 bind Unix sockets under this worktree's long temp
  path ("path must be shorter than SUN_LEN": IDE IPC, daemon, wallet onboarding) and one upstream-branding snapshot
  (`command_popup_default_items`), all unrelated and failing the same way without this change. `cargo test -p
  codex-core --lib security::` 161 passed. `just test -p codex-features` 33 passed. `just test -p
  codex-app-server-client` 37 passed, 1 failed (same SUN_LEN socket).
- `just fix` on codex-features, codex-core, codex-app-server-client, codex-tui; `just fmt`; clippy clean on the
  changed files. No Linux-only code (only `cfg(unix)` file metadata and `O_NONBLOCK`).

## Functional run (GLM 5.2, real TUI in tmux, demo SOP)

Recorded on `df875e9f123c` (final code, after merging main) with `glm-5.2` / `zai`, disposable `HOME` and Corbanu home, synthetic files only;
restarts through `demos/restart-once.sh`. Videos: [`qa/demos/index/PF-29-S01.md`](../../../demos/index/PF-29-S01.md).

| Demo | Result |
| --- | --- |
| `pf29s01-blocked` | Three missing controls and `~/.zshrc:1 OPENAI_API_KEY` listed without the value; Enter: "Not saved". |
| `pf29s01-isolated-after-restart` | Preflight passed; after restart `/status`: "protected boundary checked at launch"; agent probe: private and public key denied, ordinary home file allowed. |
| `pf29s01-flag-off` | Control: no preflight; after restart the same probe reads the private key (the PF-27-S02 limit). |
| `pf29s01-resume-refused` | After activation, `/resume` of the earlier conversation is refused with the reason. |
| `pf29s01-launch-reaudit` | Export added after the save: startup warning and `/status` say "not clean: 1 blocker", value never shown. |

In an exploratory run GLM 5.2 refused on its own to `cat ~/.ssh/id_ed25519` ("reading it is explicitly denied by my
security policy"); the probe script was used to show the OS denial.

## Independent review

Opus 5.5 High, `corbanu exec -m claude-opus-5-5-plan`, read-only.
[Review 1](review-opus-1.md) on `3f28f98939`: **CHANGES REQUIRED**. [Review 2](review-opus-2.md) on `9b1399eedc`:
**APPROVE WITH NITS**. [Review 3](review-opus-3.md) on the merge with main and the glob fix: no P0/P1, three P2.
Dispositions:

- P1 unbounded reads: fixed (`open_regular`, test with `/dev/zero` and a FIFO).
- P1 resume outside the TUI: checked at the TUI chokepoint (`resume_thread`, `fork_thread_at_with_presentation`).
  `corbanu exec` and other app-server clients are outside Aggressive by design (PF-24-S03, stated on the review).
- P2 Debug leaks: manual `Debug` with names only. P2 per-file isolation: whole credential folders. P2 flag off with a
  leftover receipt: accepted (stricter only). P2 config-only paths: message now says how to fix it.
- P3 same second: ms precision. P3 receipt after a failed save: removed. Review 2 P2 old receipts: version 2,
  version 1 refused (test). Review 2 P3 wrapped refusal text: the startup and `/resume` paths check before the
  chokepoint, so their text is unchanged (video); other callers are agent threads created in the session.

- Review 3 P2 stale level in an open picker: the saved level and receipt are read again at save; a saved Aggressive
  without a receipt needs the preflight (test). P2 glob syntax in the home path: exact database paths instead (test).
  P2 Linux: bubblewrap masks only paths that exist when a command starts, so a `-wal`/`-shm` file created during a
  long command is readable by that command; store folders are now created at launch, and the limit is recorded below.
  P3 canonical paths: the check fails closed (launch refuses); left as is.
- The GLM run itself found a startup race (SQLite `-shm` created between launch's inventory and its audit, reported as
  not clean); fixed with the glob and pre-created stores.

## Not done here

- Linux: deny entries cover only paths that exist when each agent command starts (bubblewrap); files created during a
  running command (database journals, a new key) are readable by that command until the next one.

- MCP servers, hooks and notify are "not contained", not blocked (pending Travis's PF-27-S02 decision); Claude panes
  are not inventoried.
- No consent flow for provider `auth` commands; migration of blocking findings is PF-29-S02.
- Launch isolation uses the start folder and the user/project `config.toml`; paths found only through another folder
  or config layer are reported not clean, not denied.
