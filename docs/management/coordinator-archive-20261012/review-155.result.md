**Verdict: CHANGES REQUIRED.** The cleanup process can remove the sandbox's protection on `.git`, `.codex` and `.agents` while sandboxed code may still be running. The rest of the PR is fine. I posted the review as a comment only; no review approval, push or merge.

I didn't reproduce the race: I had no Linux machine with bubblewrap. The finding comes from tracing the Linux process-exit order.

**Medium**
- **M1 — `codex-rs/linux-sandbox/src/linux_run_main.rs:960-977`.** The cleanup process waits for the outer bubblewrap process to exit and then deletes the placeholder dirs. That's safe when the command ends normally: outer bubblewrap only exits after everything inside the sandbox is dead.

  After a SIGKILL the order flips. Outer bubblewrap dies first, from its parent-death SIGTERM (`:526`). The sandbox's PID 1 is only killed after that, and its other processes keep running until PID 1 exits and takes them down.

  In that gap, deleting `/tmp/.git` (or `$TMPDIR/.git`, `.codex`, `.agents`) on the host also removes the read-only cover inside the sandbox, which is still alive. Sandboxed code could then create a real `/tmp/.git/config` (for example `core.fsmonitor` or hooks, which run on a later unsandboxed git command) or a `.codex/config.toml`. The process that normally deletes such files died with the helper, and the one-shot cleanup runs before the sandbox is fully gone, so anything created afterwards stays.

  Before this PR the dirs leaked but the cover never came off early, so this is a regression. The window is short (up to 10ms of polling plus PID 1 scheduling), but sandboxed code can widen it by loading the CPU.

  Two possible fixes:
  1. Wait on the sandbox's PID 1 instead. bubblewrap reports its pid via `--info-fd` or `--json-status-fd`.
  2. Or drop the cleanup process and cherry-pick upstream `9ca99b5171` (#39629). It treats a `.git` dir as a repo only if it contains `HEAD`, which fixes the symptom without new process code.

**Low**
- **L1:** The cleanup process is forked while SIGHUP, SIGINT, SIGQUIT and SIGTERM are blocked, and it calls `setsid` (`:953`), so only SIGKILL stops it. Its lifetime is still bounded: the command's run time plus at most 30s.
- **L2:** Only the cleanup call is wrapped in `catch_unwind` (`:974`), not the forked child's whole body (`:939-945`). A future panic there would unwind into the helper's code inside the forked process and start a second bubblewrap.
- **L3:** Forking and then allocating is only safe because the helper has no other threads at `:506`. Add a comment or assertion. If a thread is added earlier, the cleanup process could hang forever in its own session.
- **L4 (already present before this PR):** The bookkeeping directory under `$TMPDIR` can be written from inside the sandbox; upstream fixed that in `fb0781b9ee`. Also, when two sandboxes overlap, one's cleanup can remove the other's covers during the same gap. Cherry-picking `9ca99b5171` and `fb0781b9ee` would address both.

**Checked and OK**
- It only deletes registered targets: placeholders are removed only if empty, and a user's pre-existing dir is never removed.
- No privilege, file-descriptor or terminal leaks.
- The sandboxed command's exit codes and signals are unchanged.
- Failure cases do nothing rather than delete: bubblewrap failing to start, the helper dying before the pid is recorded, and pid reuse.
- The new code is Linux-only, and the new test is sound.

**Rest of the PR**
- **`remote_env: false`:** acceptable as a stopgap, but the 47 remote-executor tests now run nowhere and the job is still named `tests_linux_x64_remote`. Needs a tracking issue and a rename.
- **The 8 tests that now run alone:** safe, but they confirm the placeholder dirs also exist during every sandboxed run. That's another reason to prefer the upstream `HEAD` fix.
- **opt-level change:** test builds only, no effect on releases.
- **Remote-control timeout (100ms → 500ms):** still well under the 1s retry, so the test still proves what it should.
- **Dead-code commits:** clean; the code newly marked dead should be deleted or rewired later.
- **CI workflow edits:** no new secrets or permissions.

Comment: https://github.com/CorbanuCore/CorbanuTerminal/pull/155#issuecomment-5979428989