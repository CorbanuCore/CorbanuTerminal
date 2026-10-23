# #826: no new user namespaces for agent commands under Moderate and Aggressive (Linux)

Host: glitch (Ubuntu 24.04.5, kernel 6.8.0-146, `apparmor_restrict_unprivileged_userns=1`,
x32 ABI off), real bubblewrap (system 0.9.0 through `aa-exec -p rootlesskit`, #581).
`ulimit -n 65536`, `-j10`, TMPDIR=/tmp, disposable `CODEX_HOME`/`CORBANU_TEST_ACCOUNT_HOME`.

| Log | What |
| --- | --- |
| `glitch-before1.log` | Fail before: main's code (`core/src`, `linux-sandbox/src`, `policy_transforms.rs` from main) plus only the new protocol entry and the tests. `unshare -Ur cat <denied placeholder>` opens it (`OPENED-826`), `clone3` reaches the kernel, Moderate/Aggressive/launch-Aggressive commands list the vault store and open the Moderate marker in a namespace of their own, and `unshare -Ur corbanu exec` against another home ran at that home's level (Permissive, nested request reached the model). |
| `glitch-after1.log` | Pass after (`c8d5db38af`; `3d11753a34` only adds argument comments): linux-sandbox `user_namespaces` 4/4 and the whole crate 56/56 + 142 unit, core `issue_826` + `issue_766` 3/3, cli `nested_moderate` 11/11 (incl. `issue_826_*`). Permissive: namespace still made, nested run in one starts as before. |
| `glitch-after2.log` | Linux clippy `-D warnings` (protocol, sandboxing, linux-sandbox, core, cli; plain and developer-accounting), protocol/sandboxing/security-level unit tests, core `--lib security:: tools::orchestrator` 378/378, cli `nested_moderate` + `nested_launch`. Its core suite failures are the helper binaries `just test` builds (`test_stdio_server`, `codex-code-mode-host`), rerun in after3. |
| `glitch-after3.log` | Core security suite with the helper binaries built: pf_23/pf_25/issue_747/748/766/826/736, nested, aggressive, moderate, protected, sandbox, code_mode, rmcp_client: 168 passed. |
| `glitch-survey.log` | What breaks, through the real sandbox, without and with the rule (workspace write, network on): shell, python threads + subprocess, node child_process + worker, rustc + threads, git, `nix eval` keep working; `unshare -U`/`-Ur`, `os.unshare`, system `bwrap` fail with EPERM; a nested Corbanu sandbox says why (on main it already fails there: its registry lock is read-only). |
| `glitch-before2.log` | Review round 1 fail-before (merge-base code for core/linux-sandbox/policy_transforms plus `28c5a93c3f`'s tests): all four rule tests fail, including bubblewrap's PID 1 carrying no filter (`Seccomp_filters` unchanged). |
| `glitch-after4.log` | Pass after on `28c5a93c3f` (PID 1 filtered through `--add-seccomp-fd`, raw `clone(CLONE_NEWUSER)` refused, Permissive checked by the absent marker and filter count): linux-sandbox `user_namespaces` 5/5 and crate 57/57, core `issue_826`/`issue_766` 3/3, cli `nested_moderate` 11/11, Linux clippy plain + developer-accounting, core security suite 168/168. |
| `macos-*.log` | macOS: clippy `-D warnings` (same crates), `just test` protocol/sandboxing/security-level 456/456, core security set 96/96 (isolated HOME, 4 threads; an earlier run under load timed out/failed on skill-budget warnings from the real HOME). |
