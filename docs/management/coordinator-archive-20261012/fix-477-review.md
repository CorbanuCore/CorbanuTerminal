You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI).

Repository worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/fix-477-20261011 (branch fix/477-exec-plugin-clone, PR #492).
Review `git -C /Volumes/CorbanuDrive/Corbanu/worktrees/fix-477-20261011 diff origin/main...HEAD`. Issue #477: the curated plugin sync (core-plugins/src/manager.rs start_curated_repo_sync, a detached std::thread running core-plugins/src/startup_sync.rs) shells out to git, which kept running and writing `$CODEX_HOME/.tmp/plugins-clone-*/.git/shallow.lock` after `codex-exec` exited (an exec test walking the home after exit flaked).

Fix: PluginsManager stores the sync thread + cancel flag; `shutdown_curated_repo_sync(timeout)` cancels and waits (bounded); app-server MessageProcessor::drain_background_tasks calls it (3 s), covering in-process exec, TUI and stdio app-server. startup_sync: git runs in its own process group; on cancel the poll loop SIGKILLs the group and reaps; no HTTP/backup fallback after cancel. New tests: core-plugins unit test (cancel mid-fetch) and exec integration test (fake git on PATH; mock model answers only once the fetch runs; asserts git+helper dead and no clone dir after exit).

Please check with file:line evidence and rank findings (blocking / should-fix / nit):
1. Correctness of cancellation: every path where the sync thread can still spawn or leave a child, or write to the home after shutdown returns (HTTP/backup fallback paths, refresh_curated_plugin_cache after a successful git sync, lock wait, git_head_sha via .output()). Is the 3 s bound respected and is drain_background_tasks reached on all exec/TUI/app-server exits (including in-process SHUTDOWN_TIMEOUT budgets)?
2. Process-group choice (process_group(0) + kill(-pid)): safety of the unsafe block, pid reuse, Ctrl-C/SIGHUP behaviour change, macOS, non-unix builds, any risk of killing the wrong group.
3. Concurrency: CURATED_REPO_SYNC_STARTED is global but the handle is per manager; multiple managers/app-servers in one process; handle overwrite; Mutex poisoning.
4. Test determinism: can either new test flake (zombies vs kill(pid,0), PATH lookup of git on macOS via which, other POSTs to /responses, timing)? Does the exec test fail without the fix?
5. Anything else (API change of sync_openai_plugins_repo, Cargo.toml libc dep for unix, Bazel).
Report findings concisely with a verdict APPROVE / REQUEST CHANGES.
