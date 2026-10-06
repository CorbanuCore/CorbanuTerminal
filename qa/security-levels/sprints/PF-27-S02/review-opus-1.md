**Verdict: changes required.** The core design is sound: an env allowlist that is armed once and never disarmed, per-launch checks that require the OS sandbox, Core hardening, and the peer-pid check on the broker socket. But the protected-path list misses several credential files. In-process file tools also skip the contract entirely. Either gap lets an agent read a raw managed secret.

Tests: all `pf_27` tests in protocol, network-proxy and process-hardening pass (24/24). All `codex-core --lib pf_27_s02` tests pass (8/8). I did not build for Windows; by inspection it should compile, since the Unix-only code is behind cfg guards and Windows refuses with `UnsupportedPlatform`. I edited nothing.

## P1

**1. Credential files Corbanu itself uses are still readable by agent commands.** The list is `launch_contract.rs:38` (`PROTECTED_CODEX_HOME_ENTRIES`), plus the Keychains entry added in `capture`. It does not cover:
- `CODEX_HOME/.env`, which `arg0/src/lib.rs:304` (`load_dotenv`) loads into Core's environment and which usually holds provider keys.
- `CODEX_HOME/provider_auth.json`, the legacy plaintext provider-key store (`login/src/auth/manager.rs:1392`).
- `config.toml` and `managed_config.toml`, which can hold `experimental_bearer_token`, MCP `env` literals and HTTP headers.
- Claude's `.credentials.json` under `CLAUDE_CONFIG_DIR` or `~/.claude` (`cli/src/claude_oauth.rs:751`).

A plain `cat "$CODEX_HOME/.env"` defeats the environment allowlist.
- **Fix:** add `.env`, `provider_auth.json` and the Claude credentials path to the denials. For the config files, deny reads or refuse to arm while they contain literal secrets. Longer term, invert the rule: deny `CODEX_HOME` entirely and allow only skills, `AGENTS.md`, memories and plugins.

**2. In-process file tools bypass the contract.**
- `ApplyPatchRuntime::file_system_sandbox_context_for_attempt` (`tools/runtimes/apply_patch.rs:87-108`) uses the unprotected `exec_server_permissions`. It returns no sandbox at all when the attempt is unsandboxed.
- `structured_edit`, `structured_write` and `view_image` use `TurnContext::file_system_sandbox_context` (`session/turn_context.rs:621`). That context gets no protected denials and is never refused.

Two concrete attacks follow:
- **Read oracle:** the agent links a workspace file to `auth.json` or `secrets/*`. `structured_edit` then answers differently for "not found", "multiple matches", "write denied" and "file too large". The agent can probe guesses and recover the secret one character at a time.
- **Write path:** apply_patch on an approved unsandboxed retry, or under full access (where shell launches are refused but patches are not), can write `CODEX_HOME/config.toml` to turn the flag off, or `hooks.json`. Hooks run unsandboxed and can then read the vault.
- **Fix:** when the contract is armed, pass both contexts through `contract.protect_permissions`. Refuse when the sandbox is `None` or the file system is unrestricted.

## P2

**3. Write protection only checks `CODEX_HOME` itself and `config.toml`** (`verify_permissions`, about lines 305-310).
- A more specific writable entry inside `CODEX_HOME` still passes. It can come from the user's `writable_roots` or an approved `request_permissions` grant (for example `hooks.json`, `plugins/`, `rules/`, `skills/`). Planting a hook or MCP server there gives unsandboxed execution.
- **Fix:** refuse if any resolved writable entry is equal to or under `CODEX_HOME`.

**4. Remote exec-server launches are accepted without any enforcement** (`launch_contract.rs:225`).
- The check passes when `exec_server` is true and `sandbox_requested` is set, with nothing applied locally. The remote rebuilds the child environment from its own process (`exec-server/src/local_process.rs:641`, `child_env`). That process is never armed, so neither the allowlist nor the managed-value scrub applies. The protected paths are also local paths that mean nothing on the remote.
- The sprint's Done line "unified exec (local and exec-server)" is therefore inaccurate.
- **Fix:** while armed, refuse remote environments with a stated reason, or send `env_policy: None` with the fully scrubbed environment.

**5. Sourcing brokered values from Core's environment overrides the user's environment policy** (`credential_broker.rs:524`, `process_credential_env`).
- Previously, `exclude`, `include_only` or `inherit = none` on `GITHUB_TOKEN` meant no dummy and no authentication. Now the proxy reads Core's environment, inserts a dummy, and the agent gets authenticated access it did not have before.
- **Fix:** source only the keys that `create_env_from_vars` would have kept before the allowlist ran.

**6. Behaviour with the flag off is not unchanged for the PF-27-S04 broker.**
- `apply_secretless_launch_network_config` (`config/mod.rs:3288`) sets the runtime directory whenever `isolated_credential_broker` is on, regardless of the new flag.
- The protocol v2 bootstrap and control socket, and the broker's self-containment, are unconditional. So with only `isolated_credential_broker` on, sockets move from tmp to `CODEX_HOME/run` and the handshake changes.
- **Fix:** gate this on `secretless_agent_launch`, or record it as an explicit S04 behaviour change in the sprint and plan.
- The uncommitted `client.rs` change (fallback to the user cache directory or `XDG_RUNTIME_DIR`) has no test yet.

**7. The broker's containment can be weakened or missing without anyone being told.**
- `client.rs:247` only logs `containment`. `"none"` is accepted, as is seccomp alone or partially enforced Landlock (`broker_containment.rs:171`). This contradicts the sprint goal of refusing unsupported isolation rather than silently downgrading.
- **Fix:** when armed, fail broker start with a stated reason unless the platform's required mechanism is enforced, and show the result in `/security`.

## P3

- **Fallback runtime directory not protected:** the uncommitted fallback directories (user cache `corbanu-run`, `XDG_RUNTIME_DIR`) are not in the protected reads. Agents can list them or connect to `b.sock`, though frames are keyed, so the risk is low. Feed the broker's actual directory into `protected_read_paths`.
- **Login-shell refusal is shallow:** `is_login_shell` (line 353) only looks at the top-level command. `bash -c 'bash -l …'` and `env bash -l` get through. Profile files and common credential files (`~/.netrc`, `~/.git-credentials`, `~/.config/gh/hosts.yml`, `~/.aws/credentials`, `~/.docker/config.json`, `~/.npmrc`) stay readable, and they often hold the very tokens the broker virtualizes. Deny them, or record the limit under Remaining.
- **Managed-values list is a one-time snapshot:** it holds only secret-named environment variables present when the contract is armed. Vault and provider keys resolved later are never covered by the argv and stdin checks. Extend the list or document the gap.
- **Paths created later are not protected:** `skip_missing_path` denials leave paths created after a long-lived unified-exec session started (for example `secrets/` on first vault use) readable by that session. Add the denial even when the path is missing.
- **MCP `env_vars` forwarding is dropped silently** when armed (`rmcp-client/src/utils.rs:27`). That is inconsistent with hooks, where `handler.env` is still applied. Warn at startup, or honour the names the user listed explicitly.
- **User-initiated routes are environment-only:** app-server `command/exec` and the `!` user shell apply the allowlist but no sandbox or path checks. Record them as user-initiated exemptions in Remaining.
- **macOS close-on-exec window:** `UnixStream::connect` on Core's side can leak the control socket descriptor to a process spawned at the same moment. That process could not read secrets, but it could inject Register or Revoke requests. Low risk.
- **Sprint record doesn't match the tree:**
  - `qa/security-levels/sprints/PF-27-S02/` and `qa/demos/index/PF-27-S02.md` are linked but do not exist.
  - "removed from any launch environment" is false for remote exec-server launches.
  - "Linux tests in CI" is unverified.
  - The core tests only exercise the contract directly. None drives `env_for`, `env_for_exec_server`, `unix_escalation` or the stdin refusal in `process_manager`.