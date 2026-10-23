**Verdict: APPROVE.** Nothing blocks merge. There are two should-fix items, both about the PR description or process, plus one edge case and some nits.

I reviewed the code read-only. `gh` couldn't reach GitHub, so I couldn't open #498, #508 or #496. For the PR description I used the local draft at `/tmp/pr498.md`.

## Answers to your four questions

**1. Is the argv correct?** Yes.
- **Order:** bubblewrap runs mount operations in the order given, so `--dev /dev` must come after `--bind / /` (`bwrap.rs:272-278`), and it does. The `--unshare-*` flags apply to the whole sandbox, so where they sit doesn't matter. `--proc /proc` still comes last (`:287-290`), and `--argv0` is still added just before `--`.
- **Who reaches this path:** only full write combined with the proxy flag. Full write without the proxy skips bubblewrap entirely (`linux_run_main.rs:207-218`), so Isolated mode can't get here. The PR's "in practice: managed proxy" claim is accurate.
- **Proc check and fallback:** the check that `/proc` can be mounted (`linux_run_main.rs:449-475`) builds its own minimal sandbox, which already includes `--dev /dev` (`linux_run_main_tests.rs:82`). If that check fails and `/proc` isn't mounted, nothing in the new argv breaks.
- **Other dependents:** the only exact-argv test is the one this PR updates (`bwrap.rs:1545-1579`). `linux_run_main_tests.rs:280-292` only checks that `--bind / /` is absent from the proc check. Nothing else depends on the old argv.

**2. Effect on proxy routing.** No new access to the host, and nothing fails open.
- The new `/dev` is a private tmpfs. The device nodes on it are bubblewrap's standard set, bound from the host: null, zero, full, random, urandom and tty.
- Host device nodes were already unusable before, because the bind was `nodev`.
- `/dev/log` and the other host sockets look like a regression but aren't: proxy mode's seccomp already blocks `socket(AF_UNIX)`.
- POSIX message queues work through the IPC namespace, which isn't unshared, so they're unaffected.

**3. Does the test prove the fix?** Yes.
- `managed_proxy.rs:347-385` goes through exactly this path: the Disabled profile has unrestricted filesystem access and no unreadable patterns, plus the proxy flag.
- Without the fix it fails with `null=1`. The `other=0` check confirms `/` is still writable.
- The skip logic is sound. The skip probe runs `bash -c true`, which never touches `/dev/null`, so it can't hide this bug. Unknown failures don't cause a skip; the test runs and fails.
- Proxy routing itself is still covered by the existing Disabled-plus-proxy tests (`:173`, `:289`), which now use the new argv.

**4. Is the PR description accurate?** Mostly. The behaviour-change bullets are correct and honestly disclosed. It needs the two fixes below.

## Should-fix

1. **The PR body still contains the literal `GATE_PLACEHOLDER`.** This change affects what shell commands see in the TUI under a managed proxy, so the repo policy requires true-TUI proof and a code-blind test decision. Record those, or give a reasoned N/A, before merge.
2. **The product-spec citation is missing.** Policy requires the exact product-spec heading plus a short excerpt for a bounded fix. The draft has neither. It may be on issue #498, but I couldn't check without network access.

## Follow-up (low likelihood, not blocking)

3. **The proxy socket folder can end up hidden inside the sandbox.** The folder is created under `$CODEX_HOME/tmp`, falling back to `TMPDIR` and then `/tmp` (`proxy_routing.rs:330-344`).
   - On the restricted path it's re-added as a readable folder (`linux_run_main.rs:224-231`), but the full-filesystem path ignores that.
   - So if `CODEX_HOME` isn't set (or its path is too long for a socket) and `TMPDIR` is under `/dev/shm`, the folder is now covered by the fresh `/dev`. Proxy connections would then fail. That's fail-closed, and it worked before this change.
   - Fix options: re-bind the socket folder after `--dev` when it's under `/dev`, or just document the limitation.

## Nits

4. `/dev/pts` is now a fresh, private instance (`--dev` creates it). In environments that refuse to mount that, this path would now fail where it used to work. Every other sandboxed profile already uses `--dev`, so the added risk is small.
5. The doc comment on `create_bwrap_command_args` (`bwrap.rs:227-232`) still says this path is "preserving full filesystem access". It should mention that `/dev` is now replaced.
6. `tty` inside the sandbox may print a `/dev/pts/N` path that doesn't exist there, because `/dev/pts` is now private. The PR already discloses the private `/dev/pts`.