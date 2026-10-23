**APPROVE WITH NITS.** Every P1 and P2 from my first review is either fixed in `c234fd3830` or recorded in the sprint as a deliberate behaviour change. I found no regressions. Tests: `codex-core --lib pf_27_s02` passed 11/11 and `codex-network-proxy pf_27` passed 17/17. I edited nothing.

## Earlier findings

- **P1-1, credential files: fixed.** Agent commands are now denied reads of `.env`, `provider_auth.json`, `config.toml`, `managed_config.toml`, Claude's sign-in file (both locations), common command-line tool credential files under `$HOME`, and the broker's fallback directory. That last item also closes my P3 about the fallback directory and the `.netrc` part of the login-shell P3.
- **P1-2, in-process file tools: fixed.**
  - All the file tools now get the protected permissions, or a deny-everything profile when protection isn't possible. That covers `apply_patch` in both the runtime and the handler's checks, `structured_edit`/`structured_write`, `view_image` and the extension tools.
  - The deny-everything fallback is safe: a restricted profile with no entries still runs through the sandboxed file helper, so it can't drop to unsandboxed access.
  - A profile that can write everywhere still gets the sandbox, because the added denials count as narrowing entries.
  - An unsandboxed `apply_patch` attempt now returns the deny-everything context instead of no sandbox, which closes the write path.
- **P2-3, writable entries inside `CODEX_HOME`: fixed.** Any writable root under `CODEX_HOME` is now refused, and there is a test for it.
- **P2-4, remote environments: fixed.** The exec-server path is only used for remote environments (`process_manager.rs:1028`), so refusing it when armed doesn't affect local unified exec. The Done line was corrected.
- **P2-5, brokered values overriding the user's policy: fixed.** Only keys the user's environment policy would pass can be read from Core's environment. The list is computed before the launch allowlist runs, so the allowlist can't empty it.
- **P2-6, flag-off behaviour: fixed or recorded.** With the flag off, socket placement and value sourcing are skipped. The handshake and broker self-containment that still apply are recorded as a PF-27-S04 behaviour note.
- **P2-7, broker containment: mostly fixed.** When armed, a broker that can't confine itself is refused (Seatbelt on macOS, at least seccomp on Linux). Showing the containment result in `/security` was not done.

**Flag off:** behaviour is unchanged in `turn_context`, `apply_patch`, `env_for`, `env_for_exec_server` and the proxy configuration. The test with no contract confirms this.

## Remaining nits (none blocking)

1. **Custom provider key could be brokered** (`core/src/security/launch_contract.rs:182`). `policy_permitted_brokered_env_keys` passes an empty list where the real shell path passes the configured provider's `env_key`. If a custom provider uses `env_key = "GITHUB_TOKEN"`, the agent now gets an authenticated GitHub dummy that it didn't get before. Pass the configured provider keys.
2. **Path comparison isn't normalized** (`launch_contract.rs:368-371`). `get_writable_roots_with_cwd` returns normalized roots (for example `/private/var/...`), but `codex_home` isn't normalized. A writable root under `CODEX_HOME` can then escape the `starts_with` check when `CODEX_HOME` sits under a symlinked path (macOS `/tmp` or `/var`). Normalize both sides.
3. **No stated reason when file tools are refused** (`launch_contract.rs:384`). The deny-everything fallback makes the tools fail with a plain permission error, while shell launches give a stated reason.
4. **Untested new code:**
   - `policy_permitted_brokered_env_keys`
   - `containment_sufficient` (`network-proxy/.../isolated/client.rs:591`)
   - the `turn_context` and `ApplyPatchRuntime` wiring
   - the fallback directory chosen in `prepare_runtime_dir` (`client.rs:608`)
5. **Missing temp-dir fallback** (`client.rs:608`). On Linux without `XDG_RUNTIME_DIR`, an armed broker still falls back to the temp directory, where an agent can delete the socket. Refuse the broker, or record this as a limit.
6. **Sprint record:**
   - The sprint links `qa/security-levels/sprints/PF-27-S02/README.md` (line 24) as the "inventory and limits", but the file doesn't exist; only an empty `logs/` folder does. `qa/demos/index/PF-27-S02.md` is also still missing.
   - None of my first-review P3 limits are recorded anywhere yet: nested login shells, the one-time snapshot of managed values, paths created after a session starts, MCP `env_vars` dropped silently, and the macOS close-on-exec window. The Done line only partly covers the login-shell one ("top-level login shells are refused").
   - `/security` doesn't show the broker's containment result.

  Write the README with these limits before the exit evidence is checked.