# Corbanu Terminal 0.1.49 pre-publish QA plan

Status: **candidate QA complete; final PR CI/review/merge recorded separately; nothing published**.
See [candidate results and limitations](RELEASE.md) and [raw result summaries](candidate/).
Owner authority: Travis, D701/D702, 2026-11-17. Executor: astra-rel149 (Astra High).
The coordinator dispatches publication after the READY handoff. No additional
Travis sign-off is requested. Missing evidence remains disclosed, never passed.

## Scope and provenance

Class: **Release**; no new product initiative or sprint. Product specification:
**Shipping MVP — LIVE**, Runtime: “Rust, Apache-2.0, Linux/macOS/Windows, the
`corbanu` command”; Multi-provider inference: “OpenAI, Anthropic/Claude Plan …
and custom providers.” Included initiatives retain their own plans/sprints.

Phase-1 worktree: `worktrees/astra-rel149`, branch `work/astra-rel149`,
main baseline `d3fe32513ee65deb397fa228c786212fb5afacd3`.
Previous package: `rust-v0.1.48`, commit
`e4e8d9b223215ffa9a7764ff2de32fded42b1e9b`, published 2026-09-25.
The 0.1.48 branch used a release branch plus workspace-version/lockfile and
versioned QA/notes changes. D701 explicitly changes the base for 0.1.49 to main.

## Gates before the release branch and version-bump merge

- [x] #2549 merged (quoted script command-rule bypass), 2026-11-18 04:41:38Z;
  merge `bc5b002bb5d18cdb3a9809e22a44ba140cc13f13`.
- [x] #2555 merged (trusted PowerShell parser), 2026-11-18 01:56:42Z;
  merge `5ddb26bb2651cddb8965fa0bc27a37c78c2135b1`.
- [x] #2558 merged (exact Windows allow-rule spelling), 2026-11-18 00:29:16Z;
  merge `135197a9a1f10d2cfd4af116af1c27e516e873b0`.
- [x] #2573 merged (fixing **issue** #2567, provider cooldown), 2026-11-18 03:41:30Z;
  merge `ebbe3cfac79660b4ccf8ab01974c438ec61acfcd`.
- [x] #2571 merged (astra-models62 new models), 2026-11-18 01:56:12Z;
  merge `1036647d52c95fd02ce32112c69b799d0e5f8328`.
- [x] #2580 merged (issue #2568, Windows PowerShell command parsing),
  2026-11-18 05:55:42Z; merge `ad2b22f0b287132dbc4dad48575614403c758fc6`.
- [x] #2586 merged (#2584 diagnostic), 2026-11-18 04:37:57Z;
  merge `4dec082e87070e218114f09c69e5b7caf5ca16b7`. Corrected private fixture
  permissions pass both preflight and the [final candidate Medium replay](candidate/windows-live.json).
- [x] #2591 merged (issue #2590, ChatGPT Plan compatibility version for GPT-6.2),
  2026-11-18 05:39:12Z; merge `6ee54387e872940b64d9fc3f90db138fd4d7af2f`.
  Run final-candidate plan-route smokes for `gpt-6.2-astra` and `gpt-6.2-sol`,
  plus a `gpt-6-astra` regression; preserve exact model/effort and route evidence.

The coordinator added #2568 and #2584 as release gates on 2026-11-18, and
added #2590/#2591 at 04:25Z. The initial version diagnosis was corrected by
that worker: main advertises 0.156.0; #2591 selects 0.167.0.
Preparation PR #2572 remains the preflight evidence home. After #2584's fix
merged, the measured Medium-token candidate replay passed with two tool calls.
Both the original failure and successful replay are retained.

Poll GitHub about every 20 minutes and record merge commits/times. An open or
closed-unmerged PR, or an issue closed without the accepted fix, does not pass.
After all gates, fetch main, create `release/corbanu-0.1.49` from its exact SHA,
bump `[workspace.package].version` and workspace package entries in Cargo.lock,
assemble with `python3 scripts/release/release_notes.py assemble --version 0.1.49`,
and validate the workflow's version and nonempty versioned-notes conditions.
Do not dispatch `corbanu-terminal-release.yml` during this worker's assignment.

The release branch was cut from `ad2b22f0b287132dbc4dad48575614403c758fc6`
after [postmerge CI run 44247625166](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44247625166)
completed successfully (verified 2026-11-18 06:41Z). All eight gate merge commits
are ancestors of that base, including the named-account cooldown follow-up #2589.
Candidate build source: `c46a9d10edb39e3a44bc74b0705408269233eee3`.
All tested platforms received that exact source. Later #2595 integration changes
only documentation, including the embedded help Markdown. The coordinator authorized
no binary QA replay for that delta; [provenance](candidate/documentation-delta.json)
keeps the earlier binaries and final documentation distinct. The source archive SHA-256 is
`23e437a2e58daf5f91f0dc5a5f970f3b743ebcd6bc1d6780020bc4ce7bd25728`.

## Notes and automated checks

- [x] Initial `release_notes.py check`: 123 fragments valid.
- [x] Initial `preview` captured outside the repository, without assembling.
- [x] First-parent merge inventory since `rust-v0.1.48`: 1,266 merges.
- [x] Phase-1 [coverage audit](NOTES-AUDIT.md): nine verified fragments added;
  132 preparation fragments validate and preview succeeds. After the gates,
  main has 132 fragments; adding our nine gives 141. All 141 were assembled into
  [0.1.49 release notes](RELEASE_NOTES.md); zero unreleased fragments remain.
  Six notes tests and workflow version/nonempty-notes validation pass. Bazel
  lock update succeeds without a lockfile change.
- [x] Release-note unit checks and diff checks pass. Linux installer
  release-contract checks pass 5/5. The macOS attempt failed 3/5 because the
  fixture assumes Linux assets; it is preserved, not counted as a macOS pass.
- [ ] Capture final PR CI, native Windows checks and relevant integration results.
- [ ] Independent read-only Opus 5.5 High APPROVE on exact pushed head.

Initial review budget: **2**, both used. Round 1 approved preparation head
`0c60eea1b13dcb9dd691fc50a83ec804d6fc7146`; its nonblocking findings were fixed.
Round 2 approved `a9f033833a4da28df67c8e4a3bb37a74a638c599`. The coordinator's
2026-11-18 07:07Z grant authorizes **one extra delta-only round 3** for #2595's
documentation, reassembly and provenance, with complete-PR scope confirmed at
07:16Z. Round 3 approved `ff0c7f559dd6f3ba1fb749087ebaad9e54f93955`.
The coordinator granted **one link/provenance-only round 4** at 07:52Z to use
an absolute guide URL in the published release body. Any further round requires
a new written grant via
`$W/astra-rel149.request.md`. Never post approval/request-changes under
the maintainer account. Follow CODEOWNERS and exact-SHA security approval rules
where applicable. Merge only after all checks complete successfully, using the
literal head SHA with `--merge --match-head-commit`.

## True-TUI matrix on the final candidate

Current-main [native harness preflight](PREFLIGHT-TUI.md) passed the exercised
macOS and Windows prompt/cancel/recovery flows. Its failed navigation/onboarding
attempts are preserved. [Live local-model fixture preflight](LIVE-TUI-PREFLIGHT.md)
passed on macOS, Linux and native Windows from a High-integrity launcher, with
the stated sandbox limits. The subsequent [Windows Medium-token case failed](WINDOWS-MEDIUM-FINDING.md)
at private-registry labeling. The investigation traced it to the fixture's
inherited folder permissions; our corrected user-profile-permission Medium
replay passes all four live checks. The original failure is retained, and the
release candidate still needs its own replay after #2586 merges.
[Extended preflight](EXTENDED-TUI-PREFLIGHT.md) adds passing macOS/Linux
navigation, approval typing/cancellation and security-state restart/recovery
checks. The selected Windows non-admin sandbox correctly refuses Moderate;
its security-state replay is unavailable without default/admin sandbox setup.
Canonical TensorCash/Isometric URLs were unavailable to this executor and no
local overrides were configured. The available release-branch replays are now
complete; [candidate results](RELEASE.md) separate passes from the unavailable lanes.

Every run records source SHA, package/binary SHA-256, version, platform, profile,
working directory, literal text/key events, visible checkpoints, exit status,
and raw capture/log paths. Send text and Enter separately. Use RUST_LOG=trace
with a private log directory and synthetic credentials. Do not inspect or copy
the coordinator's profile. The debug keyring flag is not isolation for a release
binary; native credential tests need a dedicated isolated account/VM.

| Platform | Execution lane | Status |
| --- | --- | --- |
| macOS arm64 | private profile, outer Seatbelt with operator-home/Keychain denial, tmux PTY | Candidate build, 12 smoke, 7 security and 4 live checks pass; three Plan routes pass |
| Linux x86_64 | glitch, non-root Docker, private profile and tmux; count-first private build -j10 | Candidate build, 12 smoke, 7 security and 4 live checks pass; upgrade/rollback and 16 cooldown checks pass |
| Native Windows | DESKTOP-UJS3HC4, native `.exe` through MSYS2/tmux; measured Medium token and protected fixture ACLs | Candidate build, 13 smoke and 4 live checks pass; full Moderate flow unavailable in non-admin lane |

For Windows, use the existing SSH identity `ambient_windows_desktop_ujs3hc4` and
host `desktop-ujs3hc4.tail36b615.ts.net`. Check the active harness and user session
before starting. Preserve UAC and host protections; do not infer console success
from an SSH-only process. Record elevated versus ordinary desktop execution and
any unavailable interactive setup separately.

Execute on each platform:

1. Fresh profile: first paint, onboarding, `/status`, `/providers`, `/model`,
   the command list opened by typing `/`, and `/docs`; choose an existing model
   and each new applicable route. The initial plan's `/help` case was corrected
   after preflight showed it is not a supported slash command.
   Verify existing choices/defaults remain available and header/footer agree.
2. Send a one-line prompt; run a harmless workspace file-read/write task;
   cancel an in-flight request with Escape; submit a recovery prompt.
3. Open an approval, type/paste ordinary text without accepting it, then
   explicitly select/confirm or cancel. Exercise `/security` on a separate
   opted-in profile, including restart and invalid-state recovery.
4. Gate regression smoke: quoted shell script rule matching; Windows trusted
   parser and exact allow spelling; cooldown/reset-hint display, explicit clear,
   and next-request recovery after the cap, without a real long wait.
5. Quit and resume the saved conversation; inspect transcript/history and send
   another turn. Reopen after a model/provider change and verify route identity.
6. Exercise a real TUI workflow in disposable TensorCash and Isometric Game
   worktrees. Record origins, base commits, tasks, verification and diffs.

Exec, mock-provider and unit results support these runs but do not replace
true-TUI or real-provider evidence. Disclose unavailable provider routes.

## Installed 0.1.48 upgrade compatibility

Phase-1 run is recorded in [upgrade compatibility](UPGRADE-COMPATIBILITY.md):
conversation/history, vault and DB migration checks passed; resume added only
`service_tier = "default"`; 0.1.48 refused the upgraded database and recovered
on the pre-upgrade backup. The [candidate replay](RELEASE.md#installed-0148-upgrade-and-rollback)
also passes upgrade/rollback and all 14 compatibility assertions, with 16 additional
cooldown checks. Its expected in-place downgrade refusal is preserved.

Use the published 0.1.48 archive verified against its release checksum manifest.
Install into a private package layout; do not change the operator installation.
Record whether the installer itself or only the installed package layout ran.

1. In isolated OS/profile storage, run 0.1.48. Create a custom-provider config,
   a conversation with a completed response, history, and a synthetic vault
   entry using 0.1.48 itself. Record hashes, thread id and database inventories.
2. Stop all fixture processes. Copy the entire fixture as a pre-upgrade backup;
   checkpoint SQLite or use its backup API so WAL data is included.
3. Seed and record synthetic `provider_request_state` cases: idle legacy row,
   active cooldown, active lease, an unrelated provider row. Include a days-long
   cooldown for #2567's final clamping check. Never use real credential hashes.
4. Start main/final candidate on the same profile. Require no config reset,
   successful database initialization and `PRAGMA integrity_check`, existing
   thread/history reopening, a new completed turn, and vault retrieval compared
   in memory without printing the value. Inspect the salt migration and retained
   cooldown/lease controls; intentional idle-row cleanup is not data loss.
5. Record every failure and preserve the pre-upgrade fixture. Phase-1 success
   cannot qualify the later version bump or changes not yet merged.

### Downgrade safety

Do not promise in-place downgrade. Main adds migration
`0053_provider_request_key_salt.sql`; older binaries may reject a database with
newer applied migrations. Cooldown keying and security state also changed.
Probe an upgraded **copy**, preserve any refusal, and test recovery on the
untouched pre-upgrade backup. Keep config, databases plus WAL, vault/key storage,
sessions and history together. Never delete a migration row or hand-edit a live
vault/database to make the older binary start. A rollback should restore the
matching backup and 0.1.48 package; turns created after it will not be in it.

## Packaging, signing and notarization

- Validate five archives (macOS arm64/x64, Linux x64 GNU/arm64 musl, Windows x64)
  and two DMGs; verify checksum manifests and embedded versions.
- Check required packaged executables/resources (wallet daemon, ACP, code-mode
  host, rg; Linux bwrap; Windows command runner, ConPTY and sandbox setup).
- Run package wallet smoke and installer contract tests. Keep native Windows
  provider initialization and installer architecture/upgrade results.
- macOS: capture `codesign --verify --deep --strict`, signature authority/team,
  `spctl --assess`, and `xcrun stapler validate` against the exact artifacts.
  Windows: record `Get-AuthenticodeSignature` for package executables.
- **Workflow observation:** current `corbanu-terminal-release.yml` has no explicit
  Developer ID signing/notarization step. Checksums and a passing archive smoke
  do not establish signing or notarization. Record unsigned/ad-hoc/missing-ticket
  outcomes honestly; do not claim a signed release from this workflow alone.

## Benchmarks and human acceptance

The canonical benchmark ledger records no complete qualifying cycle. For
0.1.49 the counter is **12 on publication; baseline pending**, never zero.
Both the Corbanu/Hermes/Kilo live-repository comparison and full coding task/model
matrix are due and currently **Incomplete**. Runtime/spend for the full cycle
are unknown, not zero. No frozen 0.1.49 model-set manifest or approved exclusions
are yet recorded. D701/D702 authorize this release preparation; disclose the
incomplete cycle in the final record without asking Travis to sign off again.

## READY handoff (fill only after preparation completes)

Record release PR, merged exact SHA, release branch SHA, all gate merge SHAs,
Opus review head/verdict, CI links, platform/upgrade evidence and remaining gaps.
Coordinator workflow: `corbanu-terminal-release.yml`; ref
`release/corbanu-0.1.49`; `release_version=0.1.49`,
`publish_release=true`, `make_latest=true`, `reuse_run_id=""`.
These are proposed publishing inputs, **not dispatched**. Coordinator confirms
that the ref resolves to the handed-off SHA before invoking the workflow.
