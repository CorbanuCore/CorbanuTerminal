# Worker brief: investigate issue #158 — possible Windows sandbox escape for Administrator users

You are an Opus 5.5 High worker with a security focus. Work autonomously; do not ask questions.

## Context
- Issue https://github.com/CorbanuCore/CorbanuTerminal/issues/158: on the hosted `windows-2022` runner (built-in
  Administrator), `windows-sandbox-rs` test
  `unified_exec::tests::legacy_workspace_write_delete_is_limited_to_writable_roots` fails: the legacy (unelevated)
  sandbox with the workspace-write profile deleted `outside/outside-delete.txt`, outside every writable root.
  Test root was under the checkout (`D:\a\CorbanuTerminal\CorbanuTerminal\codex-rs\.tmpXXXX`). The nightly lane now
  skips this test via the `windows-native-tests` skip list in `.bazelrc` (PR #159).
- Unknown: runner ACL artifact (e.g. a directory writable by a SID the restricted token keeps) vs. a real escape for
  admin users. Corbanu Terminal users on Windows often run as local admins, so this matters.

## Do
1. Read the legacy sandbox implementation in `codex-rs/windows-sandbox-rs` (restricted token construction, which
   SIDs are kept/disabled/restricted, how writable roots are granted, deny ACEs) and its upstream history
   (`git log`; also compare with upstream openai/codex — `upstream` remote — for any fix in this area).
2. Reproduce on a hosted Windows runner with a temporary debug workflow on a `debug/sandbox-158-*` branch (push
   trigger limited to that branch; delete it afterwards). Capture: the effective token (groups + attributes, via
   `whoami /all` inside and outside the sandbox), the ACLs on the test root, `outside`, and the deleted file
   (`icacls`), and whether delete succeeded via DELETE on the file or FILE_DELETE_CHILD on the parent. Also run the
   same test with the root under `%TEMP%` and under a freshly created directory with explicit ACLs, to separate
   inherited-runner-ACL effects from token behaviour. If possible also as a non-admin user created in the workflow.
3. Conclude with evidence: artifact or real escape, and for whom (admin only? any user with a particular ACL?).
   If it's a real escape, implement the fix in the sandbox (e.g. disable/deny the SID that grants the access, or add
   a deny-delete ACE / don't rely on parent ACLs), add a regression test that proves the boundary, and get an
   independent review (`~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c
   model_reasoning_effort="high"`). If it's an artifact, make the test environment-independent (create its root
   with explicit ACLs) so it runs and passes in CI, and remove the skip.
4. Branch `codex/sandbox-158-20261005` off `origin/main`; PR to main referencing #158; checks green; post findings
   on the issue. Don't merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't weaken the sandbox boundary or the test's
intent.

## Final report (your last message)
Verdict (artifact vs escape, scope), evidence, fix and tests, review verdict, PR URL and check status.
