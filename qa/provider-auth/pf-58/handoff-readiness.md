# Candidate readiness and runtime repair evidence

## September 10 Mac superseding note

The Mac launcher now targets the [Keychain repair package](macos-keychain-20260910.md).
Ten focused/native Keychain tests and staged TMUX using **Ctrl+C** pass. The
Escape-specific cancellation check failed on both the untouched September 8
binary and replacement under the same harness. Check 11 is now yellow, bringing
the live/platform/known-failure set to twelve. The historical pass counts below
are not a new all-green certification. The user subsequently reported four
more prompts on the intended ad-hoc build: that real-user test failed. The
separate Developer ID-signed copy now passes strict signatures and the same
Ctrl+C TMUX package smoke and is installed in the launcher. Its real-user
Keychain retest remains pending; credential permissions and human checkmarks
were not changed. See the linked record for the exact signed package/hash.

This is routine QA/handoff infrastructure for PF-58, not another authentication
or security-policy change. Product citation: **Shipping MVP — LIVE** —
“operational credential use without placing raw values in chat.”

The September 8 candidate passed scoped auth tests but was copied without its
Code Mode host. The expanded wallet suite also found the missing
`pfterminal-walletd` companion. The two installed plugins required Node outside the actual
launcher PATH. A chat reply did not prove tool availability. That handoff was
incomplete; historical green runs do not qualify the delivered package.

## Repaired package

- RTX executable: `/home/travis/security-round5/evidence/provider-reauth-health-final/candidate-repaired-qualified/bin/codex`.
- SHA-256: `fa2b710733a57787cb0cfbd250be903ef9d2fe1f08b981dd49196697eae14fb8`.
- Canonical package includes the matching Code Mode host and wallet daemon,
  search and sandbox resources. Use `launch-candidate.sh` for the RTX Node PATH.
- Live Fable 5.1 Max returns `FABLE_51_OK`. The actual `/mcp` inventory now lists
  `codex_apps` with bearer authentication and both installed local plugins;
  there is no missing-command or inventory reload error in that check.
- **Final pinned-package matrix: 45/45 pass.** Management 10, reauthentication 4,
  convergence 13, multi-provider/wallet 10, Claude 3, stream 1, security 2 and
  memory/restart 2. The host/shell/MCP tool smoke and real launcher preflight also
  pass. The live Fable shell call returns the actual expected tool output.
  [Final gate report](evidence/repairs-20260908/matrix/report.html) and
  [repair details](product-repairs-20260908.md) supersede the historical failures.
- All 26 human checks remain mapped. Eleven retain explicit full-flow or
  platform/live evidence requirements; those are not silently converted into
  passes by this repair. Human acceptance remains unchecked and main unchanged.

## Native Mac follow-up

**September 10 replacement:** the installed package is now
`macos-candidate-picker-final-20260910`, not either earlier signed/ad-hoc copy.
The [picker repair record](picker-repair-20260910.md) pins both platform hashes,
58 TUI and 72 provider-auth regressions, 32 passing Linux integration tests,
and final expanded Mac/Linux actual-package TMUX. Two existing-profile Mac
launches completed without password entry and with successful Keychain reads.
Desktop placement, real browser consent, live recovery and named-human approval
remain separate. Anthropic API models require its own configured API key;
Claude subscription credentials are not reused for that billing route.

The paragraph below is the historical September 8 package record.

The repaired 0.1.41 source is now also built as a complete native arm64 Mac
package, and the existing Applications launcher points to it. Strict binary
signature checks, companion/resource execution and the actual-package TMUX
smoke passed (four host/shell/MCP round trips, three concurrent processes,
restart, cancellation and narrow Claude guidance). See the
[Mac build/install record](macos-build-20260908.md) for its exact path, hashes,
rollback targets and evidence. This is not a claim that the Linux 45-test suite
ran on macOS. Actual Mac app-window placement and real-account/Keychain flows
remain unverified, so the first card stays yellow; the missing native build is
no longer its blocker. The current coverage entry reflects that narrower gap.

## Required gate

Presentation-only follow-up: `humanTest.html` now highlights the 11 incomplete
prerequisite cards in yellow, with an explanation of the remaining evidence and
any already-verified parts. The 26 checklist IDs, saved-checkmark key and candidate
are unchanged. The archived matrix report retains the HTML hash from its original
test run; it has not been rewritten to pretend the later presentation edit was
part of that run. No new acceptance or test run is claimed by the highlighting.

1. Stage the main executable **and matching companion binaries**, preserving the
   runtime package layout, including Code Mode and wallet daemons. Never qualify only `target/debug/codex` and then copy
   a different layout for the user.
2. Run `handoff_preflight.py` with the staged candidate, actual Linux launcher
   PID, and each effective local MCP configuration. This reads only dependency
   metadata, not stored credentials. It must find the host and plugin commands
   on the actual launch PATH. For remote MCPs, record a separate live handshake.
3. Run `handoff_tmux.py` on that exact package. It creates isolated homes and a
   private socket, sends real keys, and requires actual JavaScript-host, shell,
   and MCP tool outputs at a synthetic model endpoint. It also proves three
   concurrent sessions and same-home restart. It never accesses live accounts.
4. Under the RTX build lock, run `handoff_gate.py` with the same candidate and
   those two reports. It reruns eight groups of existing real-key TUI journeys,
   rejects zero-test/skipped/failing results, and writes JSON/HTML readiness.
   `CARGO_BIN_EXE_{codex,corbanu,pfterminal}` all pin the delivered binary; auth
   fixtures use synthetic credentials and cannot use the native keyring.
5. Every `data-check` in `humanTest.html` must exist exactly once in
   `checklist_coverage.json`. Each remaining clause is a blocker, not a pass.
   Complete missing automation or record exact-candidate operator evidence
   before removing a clause. The matrix intentionally records gaps rather than
   equating partial tests with every sentence in the human checklist.

Mac desktop/grid placement, real browser consent, native keychain prompts and
real-provider availability cannot be certified by Linux synthetic TMUX. These
require operator preflight on the relevant platform, not asking the human
acceptance tester to discover a broken setup. No human checkbox is auto-checked.
The native-child source-admission case remains in the provider-convergence group;
the repair restores live-parent policy inheritance instead of excluding the test.

## September 8 initial results — historical, before product repairs

All 26 human checks are mapped. Fifteen have their mechanical baseline mapped
to automated suites; eleven retain explicit missing automated or platform/live
evidence. A mapping is not a pass. The runtime preflight blocks the entire handoff.

Latest outcomes across the recorded runs (not a single all-green final run):

| Actual-key RTX group | Passed / run | Remaining failure |
| --- | --- | --- |
| Provider management | 10 / 10 | None in this group |
| Credential recovery | 4 / 4 | Live connected-app reconnection remains separate |
| Provider convergence | 11 / 12 | Native child: source admission policy unavailable |
| Multi-provider / wallet onboarding | 4 / 10 | Six cases cannot open wallet flows: missing `pfterminal-walletd` |
| Claude onboarding/recovery, updated drivers | 1 / 3 | Two missing-selection/legacy recovery cases return to the manager without opening recovery |
| Orphan stream delta | 1 / 1 | None |
| Security views | 2 / 2 | None; observation-only, not protected-mode activation |
| Memory worker/restart rehearsals | 2 / 2 | None; includes multiple scenarios per test |

Total: **35/44 Rust TMUX tests pass; nine fail.** The additional Python gate
regressions pass **10/10**. The standalone packaged-tool TMUX smoke passes real
host/shell/MCP execution four times, three concurrent processes, same-home
restart, active-stream cancellation and recovery, permission/security cancel,
and Claude guidance at 40 columns. It does not qualify the missing wallet helper
or the user's installed plugin runtime.

The CLI SHA-256 is
`37511bb9348e3793255ce67806d23060f3436cee53ef0a041b921420b201da35`;
the newly built Code Mode host is
`b91e5ab7e7af108b5651cf30a73dae98d2e98c49ecc6a87aaed8154ef92179a5`.
The staged test directory is
`/home/travis/security-round5/evidence/provider-reauth-health-final/candidate-preflight`.
The user's running `candidate-replacement` directory was **not** modified.

[Metadata-only reports](evidence/handoff-20260908/) preserve the initial broad
run, separate recovery run, updated Claude run and final tool/preflight results.
The initial report intentionally retains its failed early tool probe; the later
`tools.json` supersedes that probe only, not any failing product tests.
Full synthetic fixtures remain in the RTX evidence directory under
`handoff-matrix-1`, `handoff-reauth-1`, `handoff-claude-4` and
`handoff-tools-formatted`. No raw OAuth challenge, credential or private history
is copied into these repository report files.

The legacy Claude drivers previously used old menu labels and an unverified row
position. They now select by exact visible label, wait for actual chat readiness,
explicitly finish onboarding, use an invalid token with embedded whitespace,
and direct accidental OpenAI OAuth attempts to loopback. Fresh onboarding now
passes. The remaining recovery failure agrees with
`claude_intent_for_status`: recovery/configured status plus an unknown source
returned `None`, and the caller cancelled management authentication. That initial
finding is corrected by the later [product repairs](product-repairs-20260908.md).

No new independent review was run, no human acceptance was checked, no release
benchmark or live TensorCash/Isometric qualification is claimed, and nothing is
merged to main by this QA follow-up.

## Commands (RTX, allocated worktree)

Use new evidence directories; do not overwrite a previous run. Keep the shared
build lock during formatting, compilation and the suite run. Set
`CARGO_TARGET_DIR` to the existing shared target and use the established RTX
tool PATH. `TMPDIR` should be a short path for Unix TMUX sockets.

```sh
python3 -m unittest discover -s qa/provider-auth/pf-58 -p 'test_handoff_gate.py'
python3 qa/provider-auth/pf-58/handoff_preflight.py --help
python3 qa/provider-auth/pf-58/handoff_tmux.py --help
python3 qa/provider-auth/pf-58/handoff_gate.py --help
```

Trace logs are permitted only for synthetic fixtures. Never enter real tokens
in a tracing fixture. Tests clean up only their own TMUX server/homes; the
user's live session and credentials remain untouched.

Review budget: PF-58 has already used all five independent reviews. This QA
follow-up does not silently reset that cap or claim a sixth review.
