# Safe automated tests

Ordinary tests must never request access to an operator's Keychain or use their
live Corbanu profile. Do not resolve a test prompt by clicking **Allow** or
**Always Allow**, unlocking a personal Keychain, or copying authentication files.

## Ordinary debug tests

From the repository, use:

```sh
just test -p app_test_support --locked --offline
just test -p codex-app-server --locked --offline
just test -p codex-tui --locked --offline
```

`just test` calls `scripts/isolated_rust_tests.py` on macOS, Linux and Windows.
It creates a disposable, empty `CODEX_HOME`, removes the higher-priority
`CORBANU_HOME` and `PFTERMINAL_HOME` aliases, strips common inherited inference
credentials, and forces the existing debug-only native-keyring denial flag.
Build caches/toolchain paths remain available. The wrapper does not modify the
parent environment, copy credentials, or change the running application.

The app-server test helper additionally pins **all three** profile aliases to
its own fixture after caller environment overrides, and forces native-keyring
denial. A fixture cannot undo this through its environment override list.

Tests that create their own profile must set all three aliases to that fixture,
or remove the two Corbanu aliases before setting `CODEX_HOME`. Merely setting
`CODEX_HOME` is insufficient. Never use the operator's profile as a test fixture.

Use the wrapper for scheduled jobs, manager verification, and subagent test
commands too. Do not substitute raw `cargo test`/`cargo nextest`, retry an older
unfixed checkout, or turn off the guard to make a test pass. Build availability
and fixture failures are failures, not permission to use personal credentials.

## Native Keychain and packaged-build qualification

The debug flag is **not an OS sandbox** and production/release binaries ignore
it. The ordinary wrapper rejects release/custom/reused build options; it is not
the native qualification runner. It does not claim to isolate arbitrary shell
commands, all ambient credentials, malicious tests, or every system service.

Tests that actually need Keychain behavior, or execute packaged/optimized builds,
belong in a disposable VM or dedicated test account with synthetic credentials
and enforced separation from operator files, processes and credential services.
Record the exact binary, positive controls, negative access probes, cleanup and
results. Follow [code-blind functional qualification](../../qa/code-blind-functional/README.md)
when applicable. Do not use a shared personal login Keychain as the test target.

## If a test triggers a native prompt

Stop dispatching successors/retries immediately. Identify the exact test-owned
processes and stop/reap those under the campaign's ownership rules; preserve the
user's interactive session and unrelated workers. Keep logs private, mark the run
contaminated, and investigate before restarting. A process-local timeout is not
a cross-process circuit breaker: a new process can create another queued prompt.
Do not print credential values or broadly terminate SecurityAgent/securityd.

## Regression checks

```sh
python3 -m unittest discover -s scripts -p test_isolated_rust_tests.py
just test -p app_test_support -E 'test(test_environment::)' --locked --offline
just test -p codex-keyring-store -E 'test(isolated_fixture_never_accesses_native_keyring)' --locked --offline
```

These checks use synthetic profiles and verify home precedence, forced guard,
child environment, unchanged canaries, temporary-profile cleanup, exit status,
and rejection of unsafe build modes. They do not read the operator's Keychain.

Incident provenance and qualification evidence:
[September 14 test-isolation repair](../../qa/reliability/test-keychain-isolation-20260914.md).
