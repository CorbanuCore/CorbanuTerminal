# Corbanu Terminal 0.1.49 candidate evidence

D701/D702 authorize this release. The coordinator dispatches publication after
the READY handoff; this worker has not dispatched a release workflow, created
a tag or published packages. Final PR CI, exact-head Opus review and merge
identity are recorded on the release PR and in the worker's READY handoff after
this evidence commit. This record does not pre-approve those steps.

## Candidate and gates

- Branch: `release/corbanu-0.1.49`.
- Main base: `ad2b22f0b287132dbc4dad48575614403c758fc6`.
- Build source: `c46a9d10edb39e3a44bc74b0705408269233eee3`.
- [Postmerge CI 44247625166](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44247625166)
  passed on that main base; verified 2026-11-18 06:41Z before cutting the branch.
- All eight required PR merge commits are ancestors of the base: #2549,
  #2555, #2558, #2573 (issue #2567), #2571, #2580 (issue #2568), #2586
  (issue #2584), and #2591 (issue #2590). The named-account cooldown follow-up
  #2589 is also included. Exact gate SHAs are in [the plan](PRE-PUBLISH-QA.md).
- Eight preparation commits were carried forward. The workspace and its lockfile
  entries move to 0.1.49; the release workflow's default input follows.
- After binary QA, the coordinator requested the complete documentation PR #2595,
  merged on main as `edfc83dcfe5c36b993d9db43a6a93235e4207d87`. It updates command-rule
  guidance and five release claims. The only compiled-input change since tested
  source `c46a9d10` is embedded help Markdown; no Rust or other executable source
  changes. The earlier binary hashes/results remain accurately attributed to
  that tested source. Per the coordinator's 07:07Z grant and 07:16Z confirmation,
  binary QA was not repeated for this documentation-only delta. See
  [delta provenance](candidate/documentation-delta.json).
- All 141 fragments were reassembled with those five corrections. The body matches
  the updated preview apart from one command-rules link made absolute for the
  published release body. No fragments remain. [Coverage audit](NOTES-AUDIT.md)
  records verified additions.

## Builds and automated checks

All native builds used Rust 1.99.1, private targets, `--locked`, and disabled dev
profile debug information. These are development binaries from the exact
candidate source, **not the workflow's release archives**. One worker build ran
at a time; glitch had one other real Cargo build before ours started.

| Platform | Build | TUI smoke | Security-state flow | Live tool task |
| --- | --- | --- | --- | --- |
| macOS arm64 | pass, 2m52s, -j8 | 12/12 | 7/7 | 4/4, 103.97s |
| Linux x86_64 GNU | pass, 1m57s, -j10 | 12/12 | 7/7 | 4/4, 9.55s |
| Native Windows x86_64 | pass, 2m27s, -j8 | 13/13 | read-only picker passed; full Moderate flow unavailable in non-admin lane | 4/4, 21.54s |

[Build hashes and checks](candidate/builds.json) identify all three CLI binaries;
[Windows manifest](candidate/windows-binaries.json) also identifies its three
matching sandbox helpers. The Mac build includes a matching code-mode host.
Release-note unit tests pass 6/6, Linux installer contract tests 5/5, workflow
version/nonempty-notes validation passes, and `just bazel-lock-update` passes
without lockfile drift. Main's full postmerge run covers the merged quoting,
Windows parser and allow-rule security regressions; no separate interactive
exploit test is claimed by this release worker.

The initial Bazel invocation used the wrong working directory; the corrected
repository-root invocation passed. Initial remote extraction commands ran before
SCP completed and failed; those incomplete trees were preserved and never built.
Both completed source archives were checksum-verified before fresh extraction.
Raw attempt logs remain in the worker evidence directory.

## True-TUI and provider evidence

Keys were sent through tmux, with text and Enter separate. The smoke flows
exercise first paint, `/status`, `/model`, `/providers`, `/docs`, `/security`, the
slash-command list, typing without accepting an approval, explicit approval
cancellation, a response, request cancellation and recovery. Each platform
persisted two completed responses and an abort. Security-state checks on Mac
and Linux cover prerequisites, saving Moderate, restart, malformed-state fallback
to Aggressive, repair and another restart.

- [Mac smoke](candidate/macos-smoke.json), [security](candidate/macos-security.json),
  [live](candidate/macos-live.json).
- [Linux smoke](candidate/linux-smoke.json), [security](candidate/linux-security.json),
  [live](candidate/linux-live.json).
- [Windows smoke](candidate/windows-smoke.json), [live](candidate/windows-live.json).

The live task reads a fresh unpredictable marker and integer array, writes the
correct marker/sum, preserves the input, and requires persisted tool calls and
completion. It uses the already-loaded local MLX Qwen3.8 Flash Next model, one
task at a time with a 240-second cutoff; no server/model settings changed.

Mac uses an outer Seatbelt boundary denying the operator home and Keychain,
with candidate native sandboxing disabled for the live fixture. Linux uses
non-root Docker with read-only root/candidate mounts, bounded tmpfs, dropped
capabilities and no host credential mounts; its live case uses a private SSH
loopback forward and candidate native sandboxing disabled. These passes do not
qualify native Mac sandbox or Linux bubblewrap enforcement. The Linux mock/security
and upgrade containers have no network.

Windows is a native `.exe` on DESKTOP-UJS3HC4 through the existing MSYS2/tmux
harness, not WSL or a desktop GUI claim. Both final runs used a measured Medium
token (S-1-16-8192; administrator groups deny-only), native non-admin sandbox,
private known-folder resolution, and individually protected AppData/Local and
workspace ACLs: owner=user, Full control for user/SYSTEM/Administrators.
The original #2584 failure and its fixture-permission root cause remain in
[the finding](WINDOWS-MEDIUM-FINDING.md). The corrected candidate live task
passes with two `shell_command` calls. Default/admin sandbox setup and its full
Moderate security-state flow were not attempted. All live SSH forwards closed
with their owning commands.

### Required ChatGPT Plan models

All three low-effort, real-provider TUI cases pass on the candidate:
`gpt-6.2-astra` (14.96s), `gpt-6.2-sol` (4.87s), `gpt-6-astra` (4.86s).
The picker exposes all three. Each request uses the requested model/effort,
compatibility version 0.167.0 and HTTP 200; each completes and persists the
assistant's unpredictable nonce. Discovery also uses 0.167.0 and returns 200.

[Results](candidate/plan-results.json), [request metadata](candidate/plan-requests.json)
and [manifest](candidate/plan-manifest.json) preserve the evidence. An in-memory,
fixed-upstream broker uses the existing sign-in; the product receives synthetic
auth in an isolated home with operator-home/Keychain denial. No real credential
was copied into the fixture or evidence, and scans pass. Each SSE request records
a client connection reset after the completed event as the TUI closes its stream;
all three persisted completions and nonce checks pass. No 429, retry, fallback,
WebSocket or optional recovery qualification is claimed. This is three required
route smokes, not a full model/effort benchmark matrix.

## Installed 0.1.48 upgrade and rollback

The published GNU 0.1.48 archive was checksum-verified and used in a private
installed package layout. The installer itself was not exercised by this upgrade.
A fresh no-network Docker profile created a conversation, history and synthetic
vault entry using the old binary. A stopped full-profile backup preceded the
candidate upgrade; active provider fixtures were seeded immediately before it.

[Baseline](candidate/upgrade-baseline.json), [upgrade](candidate/upgrade-result.json)
and [14 compatibility assertions](candidate/upgrade-assertions.json) pass:
original config entries, thread identity, rollout/history prefixes and vault
value survive; a second response extends the same conversation. Resume adds only
`service_tier = "default"`. All four database integrity checks return `ok`.
State migrations add 53 (32-byte salt) and 70 (provider reset hint). Idle legacy
rows are intentionally removed; active cooldown, lease and unrelated rows remain.

[Cooldown replay](candidate/cooldown-result.json) passes 16 checks: actual old-key
adoption, one-hour cap, preserved longer reset hint and active lease/count, no
request during cooldown, TUI and CLI clearing, lease retention, unrelated-row
preservation and successful probes after clearing or expiry. Fixture SQL advances
only the isolated lease/cooldown timestamps, avoiding a real one-hour wait.

**In-place downgrade is incompatible.** The old binary refuses migration 53 and
misleadingly calls the database damaged; the [expected refusal](candidate/downgrade-refusal.txt)
and [failed old-version startup result](candidate/downgrade-result.json) are kept.
Restoring the untouched full backup at the same absolute profile path passes
[rollback](candidate/rollback-result.json), including session/config/vault checks.
Do not erase migration rows. Restoring a backup loses turns created after it.
The synthetic vault canary was absent from all 21 upgrade evidence files scanned.
This upgrade lane is Linux; native macOS/Windows upgrade and native credential-store
migration are not claimed.

## Remaining evidence limits and coordinator handoff

- The competitive benchmark cycle remains **Incomplete**, counter **12 on
  publication**, no reset; see [disclosure](benchmarks/README.md). Canonical
  TensorCash and Isometric repositories were unavailable and no overrides were
  configured. Disposable tool fixtures do not replace those lanes.
- Mac dev binary: ad-hoc signature, strict verification passes, no Developer ID
  team. Windows dev binaries: NotSigned. No final archive, DMG, release checksum,
  notarization ticket, Gatekeeper or package installer/architecture evidence is
  available before the coordinator runs the packaging workflow. The workflow
  has no explicit Developer ID signing/notarization step. Those artifact checks
  remain coordinator post-build work; they are not reported as passes here.
- Final exact-head Opus review and release-PR CI must finish before merge/READY.
  Initial Opus rounds 1 and 2 approved preparation and candidate head `a9f03383`.
  The coordinator granted one extra delta-only round 3 for #2595's documentation,
  reassembly and provenance, then one link/provenance-only round 4 at 07:52Z.
  Final exact-head review/approval is recorded on the
  PR; do not alter that head without following the review-budget rules.

Once the PR is reviewed, green and merged, the coordinator uses workflow
`corbanu-terminal-release.yml`, ref `release/corbanu-0.1.49`,
`release_version=0.1.49`, `publish_release=true`, `make_latest=true`,
`reuse_run_id=""`. The READY handoff supplies the exact final branch SHA.
The coordinator verifies that ref before dispatch; this worker does not dispatch.

Raw evidence roots: local `.codex-work/astra-rel149-final-*`; glitch
`~/corbanu-rtx/astra-rel149/final-*`; Windows
`C:\CorbanuQA\astra-rel149\acl-parent-candidate-{smoke,live}-01`.
Harnesses, full build logs and attempt records are under
`.codex-work/workers-20261002/astra-rel149-*`. Small unchanged result files are
committed in [candidate/](candidate/).
