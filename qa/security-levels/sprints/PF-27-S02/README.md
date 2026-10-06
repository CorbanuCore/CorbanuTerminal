# PF-27-S02 secretless agent launch: gate evidence (2026-10-06)

Branch `feat/pf27-s02-secretless-20261006`, macOS arm64 debug build, Rust 1.95.0. Feature flag:
`[features] secretless_agent_launch = true` (default off). The broker cases also turn on
`isolated_credential_broker` and the managed network proxy. Synthetic credentials only.

## What ships (flag on)

- **Launch environment allowlist.** Agent command environments keep ordinary tool variables only. Provider keys,
  tokens, passwords, credential helpers, start-up hooks and URL passwords are removed. Login shells and shell
  snapshots are off. The allowlist is in `protocol/src/secretless_launch.rs` and is applied wherever an environment is
  built from Core's process: shell, unified exec, zsh-fork escalations, the `!` shell, hooks and MCP stdio.
- **Per-launch contract** (`core/src/security/launch_contract.rs`). Every agent launch must run under Seatbelt or
  bubblewrap. Its profile gets read denials for Corbanu's credential and history stores and for common CLI
  credential files, and nothing under `CODEX_HOME` may be writable. Raw managed values in argv or unified-exec stdin
  are refused. Unsandboxed, full-access, escalated and remote-environment launches, and Windows, are refused with a
  reason. In-process file tools get the same profile or no file access. Core makes itself non-dumpable (Linux) or
  refuses debugger attach (macOS).
- **Brokered credentials still work.** The proxy sources brokered tokens from Core's own environment (only those the
  user's environment policy would have passed) and gives the agent dummies.
- **Broker hardening** (also applies to `isolated_credential_broker` alone, see "S04 behaviour note"). Secrets never
  cross a pipe Core created: the broker prints only its control-socket path, accepts its parent only, and Core checks
  the socket peer is its child. The broker confines itself before starting its runtime (Seatbelt on macOS; seccomp,
  plus Landlock where the kernel has it, on Linux). It cannot exec, fork, trace or read other processes, and writes
  only under its runtime directory. It exits when its controller dies. With the flag on, sockets live in
  `CODEX_HOME/run`, or the per-user cache/runtime directory when that path is too long, and an unconfined broker is
  refused.

## Launch-boundary inventory

| Route | With the flag on |
| --- | --- |
| Shell tool, unified exec (local), zsh-fork escalations | Allowlist, sandbox required, protected paths, argv/stdin checks |
| Retries without sandbox, `danger-full-access`, approved escalations | Refused with a reason |
| Remote environments (exec-server) | Refused (the remote host builds the environment) |
| apply_patch, structured edits, image view, extension file tools | Protected profile, or no file access |
| `!` user shell, app-server `command/exec` (user-initiated) | Allowlist only; no sandbox requirement (open decision) |
| MCP stdio servers | Allowlist on inherited variables; literal `env` from the server config kept; run unsandboxed (open decision) |
| Hooks, legacy notify | Allowlist, no login shell; run unsandboxed (open decision) |
| Browser isolation | Already `env_clear` with an explicit container environment |
| Claude panes / external provider harnesses | Not covered (Remaining) |
| Windows | Every launch refused (PF-27-S06) |

## Tests

| Command | Result |
| --- | --- |
| `just test -p codex-core pf_27_s02` | 12 passed (contract, `env_for`/`env_for_exec_server` with an explicit contract, file tools) |
| `cargo nextest run -p codex-network-proxy -p codex-process-hardening -p codex-protocol` | 560 passed before the review fixes; `pf_27` 19 passed after |
| `just test -p codex-protocol -p codex-process-hardening -p codex-network-proxy -p codex-hooks -p codex-rmcp-client -p codex-features` | 922 of 923; the failure is `streamable_http_oauth_store_pinning` (native keyring unavailable in the isolated fixture; unrelated to this diff) |
| `just test -p codex-core -E '<sandboxing, unified_exec, config, network_proxy, credential, escalation, schema, pf_27>'` | 744 passed |
| Linux (Docker, `rust:1.95-bookworm`, kernel 6.12 linuxkit) `cargo test -p codex-process-hardening -p codex-network-proxy -p codex-protocol pf_27` | all passed; the broker reports `seccomp` (this kernel has no Landlock) |

The contained-broker test re-executes the test binary, confines it and checks that it cannot exec or write outside
its directory: macOS `seatbelt exec=denied inside=ok outside=denied`; Linux Docker `seccomp exec=denied`.

## GLM 5.2 runs (`-m glm-5.2`, provider `zai`)

`corbanu exec` smoke runs and the TUI recordings below use a disposable home, the synthetic GitHub Enterprise
fixture from PF-27-S04 (`127.0.0.1.nip.io:8443`, real token sha `946ae98e9fbe`) and a canary
`PF27_CANARY_API_KEY`.

| Case | Flag off | Flag on |
| --- | --- | --- |
| Credential-looking variables in the agent | `GH_ENTERPRISE_TOKEN`, `PF27_CANARY_API_KEY`, `SSH_AUTH_SOCK`, `ZAI_API_KEY` | `GH_ENTERPRISE_TOKEN` only, a dummy |
| Brokered request with the dummy | authorized | `authorized: true`, server saw `946ae98e9fbe` |
| Read `auth.json`, `secrets/`, `.env`, `config.toml`, shell snapshots | ALLOWED | denied |
| Write `config.toml` | denied | denied |
| Read Core's environment with `ps eww` | denied for the agent; the human's own shell reads it (count 1), which proves the pid | denied |
| Run under `danger-full-access` | runs | `Protected launch refused: this command would run outside the OS sandbox …` |
| Reach `auth.json` through file tools | n/a | refused (GLM reported no bytes read) |

## Videos (SOP, `qa/demos/index/PF-27-S02.md`)

1. `pf27s02-secretless-env`: the agent sees only a dummy, and the broker still authorizes.
2. `pf27s02-protected-paths`: every probe prints denied.
3. `pf27s02-unsandboxed-refused`: full access is refused with the reason.
4. `pf27s02-broker-contained`: `containment=seatbelt`, an owner-only directory, and only `b.sock`.
5. `pf27s02-baseline-flag-off`: control run with the flag off; reads are allowed, and the human shell reads Core's
   environment.

## Review

One independent Opus 5.5 High review: `review-opus-1.md` (CHANGES REQUIRED, 2 P1, 5 P2) and a re-check of the fixes
in `review-opus-2.md` (APPROVE WITH NITS). Fixed after the re-check: custom provider `env_key`s are never brokered,
writable-root comparison is symlink-normalized, and tests were added for permitted keys, the containment
requirement and the runtime-dir fallback.

## S04 behaviour note

With only `isolated_credential_broker` on, the control handshake (protocol 2, control socket instead of stdin) and
broker self-containment now apply. Socket placement under `CODEX_HOME/run`, value sourcing and the containment
requirement change only with `secretless_agent_launch`.

## Known limits (not claimed)

- Files the agent can still read can hold secrets. Shell profiles are readable, and a nested `bash -l` loads them;
  only top-level login shells are refused. Credential files outside the deny list (for example `~/.ssh`) also stay
  readable. PF-29 migration is the fix.
- The argv/stdin check knows only Core's secret-named variables at arm time, not vault values or keys resolved later.
- Read denials skip paths that did not exist when the session started (bubblewrap cannot mask missing paths).
- MCP `env_vars` pass-through names that look like credentials are dropped silently; literal `env` values stay.
- On macOS, unsandboxed same-user processes (MCP servers, hooks, the user's shell) can read Core's launch
  environment through `KERN_PROCARGS2`. PF-27-S05 moves Core's own keys out.
- On macOS a process spawned at the instant Core connects can inherit Core's end of the control socket. It cannot
  read secrets, only inject or race control messages, which is a denial of service.
- On Linux without `XDG_RUNTIME_DIR` and with a long `CODEX_HOME`, the broker falls back to the temp directory, where
  an agent can delete its socket (denial of service only).
- Linux Landlock is best effort; seccomp is required. `/security` does not show the broker's containment yet (PF-41).
- In-process file tools refused by the contract fail with a plain permission error, without the stated reason.
- Brokered OpenAI keys are not sourced under the flag (provider keys are never passed to agents).
- Windows: PF-27-S06. Linux end-to-end agent run: Remaining (Docker Desktop VM ran out of disk building Core).
