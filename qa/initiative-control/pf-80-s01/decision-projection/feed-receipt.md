# PF-80-S01 fixture decision feed — uncommitted worker receipt

Product initiative: **Internal delivery control — TO BUILD**, “Show blockers, rendered sprints, human test plans, machines, run logs and freshness.” Plan `docs/plans/active/initiative-delivery-control.md`; current sprint `docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md` remains in_progress. Read corbanu-terminal-development, root AGENTS, full active plan/current sprint, decision-feed-next packet and frozen design before coding. Parent owns shared ledgers, real state, publication and subsequent allocation.
Verified clean launch and unchanged HEAD `a0e2c327065142d7262db612aaac68c1bce8cea0`, branch `workstream/tasknode-pf80-s01-20260911`, checkout `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`. Required base `aa3f9e2c15b4cd84de28f0900f4055a6abcee80d` is an ancestor. No commits, staging or pushes. Seven of eight permitted paths changed; `test_control.py` needed no fixture compatibility edit. No other writes to checkout or real/private operational state.

## Transport and presentation

`decision_feed.py` reads only the fixed manager input `state/decisions.fixture.json`, with the frozen private-directory/file ownership checks and frozen schema validation. Input is read nonblocking/no-follow and bounded to 1 MiB plus one overflow-detection byte. Canonical snapshot envelope is `{schema:1,status:valid|missing|invalid,feed:validated-object|null}`; limit is 1 MiB + 128 bytes. Missing, invalid and legacy-unrecorded feeds remain unknown with unknown open count, never fresh empty. Rejected contents and read-error details are not exported.
Every destination export creates exactly `source/decision-feed.json` once, before the manifest completion record. `source.json.decision_feed` pins exact envelope-byte SHA-256 (`digest`), frozen canonical feed SHA-256 (`payload_digest`), feed identity/revision/assessment, status, fixed artifact name and input-observation SHA-256 (`input_digest`). The observation binds private input bytes and file metadata; directory modification time is excluded because unrelated publication bookkeeping changes it. No raw rejected content enters metadata. These separate feed pins are not added to repository file hashes, whose local paths remain resolvable. Existing source/config/inventory verification remains intact.
Export rechecks input before completing; existing `--verify` also checks the input against its original collection-time validation. Activation verifies fixed-artifact bytes/schema/pins before state or service changes. Collect verifies again before publication. Malformed metadata, noncanonical/oversized/duplicate-key payloads, symlinks, nonregular files, missing pinned artifacts and digest/identity/time mismatches fail closed and retain the last good publication. Legacy manifests without feed metadata deliberately show unknown even if a stray artifact exists.
Each source generation owns its immutable snapshot. Explicit source-generation rollback restores that generation's original assessment and history, which visibly becomes stale; it does not refresh or merge decision records. This is existing source rollback, not permission to overwrite canonical manager history; frozen `save_fixture` retains its authoring revision/history safeguards. No new transport authentication or anti-replay service is introduced.
Overview calls the frozen accordion renderer above workstreams, using existing notice styling and the existing approved/sanitized document corpus. Current sprint and historical PF-76 links resolve by exact path/identity; unavailable evidence gets an explanation without recursively publishing it. Native details and permanent anchors remain; loading a decision hash opens its containing details. Facilities, operations notices, activity labels, disabled posting and guarded serving code remain intact.
Health and publication manifest expose feed identity, both digests, assessment, input status, fresh/stale/unknown state, open/last-known count and the 1200-second threshold independently of source collection/publication times. On publication failure, health retains the displayed generation's decision assessment and ages its state. The existing page timer ages generated feed/context/evidence labels even if health is failed or unavailable, preserving original timestamps, user text and link labels; it never manufactures a new assessment. Slack remains not connected; answers belong in the manager task; no reply controls exist.

## Final implementation evidence

Pinned interpreter `/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python`, always `-B` for implementation tests. Initial nine feed tests passed in 19.017s; final broader run included ten feed tests and passed **109 tests in 30.219s**. Original 100 tests remain unchanged. The one existing `test_control.RefreshTests.test_real_http_server_is_read_only_and_rejects_untrusted_hosts_and_paths` was intentionally not run under the explicit no-actual-server instruction; 99 existing tests plus ten new tests ran. It is outstanding, not relabeled as passing. Activation's systemctl calls were stubbed; “server started” console messages are existing fixture output, not service execution. Exported publisher subprocesses used only temporary synthetic state, without serving.
Final command (repository root; `PY` below means the exact pinned interpreter):
```sh
PY -B -c 'import unittest; suite=unittest.defaultTestLoader.discover("scripts/initiative_control", pattern="test_*.py"); flatten=lambda s: [t for item in s for t in (flatten(item) if isinstance(item, unittest.TestSuite) else [item])]; tests=flatten(suite); excluded=[t for t in tests if t.id().endswith("test_real_http_server_is_read_only_and_rejects_untrusted_hosts_and_paths")]; print("Preserved but not run under no-server boundary:", [t.id() for t in excluded], flush=True); result=unittest.TextTestRunner(verbosity=1).run(unittest.TestSuite(t for t in tests if t not in excluded)); raise SystemExit(not result.wasSuccessful())'
```
Also passed: `PY -B docs/plans/check.py` (3/3 active), `PY -B docs/sprints/check.py` (115 current / 126 archived), `node --check scripts/initiative_control/status.js`, `git diff --check`. Node executes the actual shipped JavaScript in a minimal DOM harness with mocked fetch and clock; this is supporting logic evidence, not browser proof. Existing migration/source/config-drift/last-good/Facilities/notice tests passed in the broader run.

| Changed implementation path | Added / removed | SHA-256 |
| --- | ---: | --- |
| scripts/initiative_control/decision_feed.py | 116 / 0 | `1637bfd49f33df3edfd78b8f21989a90aecac1120ab5d48b10dbe443718b2230` |
| scripts/initiative_control/test_decision_feed.py | 293 / 0 | `6ef7213245b3a8476030e168645a98aebe9b1bb842b5f0313f39e49ca84761d0` |
| scripts/initiative_control/export.py | 13 / 1 | `3ac33407d18c87ce2805d2cff69e33da007757614eb5e78cbbd09ea8b72c2fb8` |
| scripts/initiative_control/activate.py | 3 / 1 | `d4b1bc58ded2589a7aeb412aee842e5d889db0b52e62e1c4658f5c791b5fdaa0` |
| scripts/initiative_control/control.py | 15 / 3 | `9c233f0d94f3cbddcfe89bec7e96498db5c3809b5d6745038f82d12c772bb4dd` |
| scripts/initiative_control/status.js | 37 / 0 | `8f27494cdba4b99f9c7e9d29717e2f3004bdd9206622b5a465c429c52eec356b` |

Implementation subtotal: 482 changed lines (477 added / 5 removed), 189 non-test. Receipt lines count toward both budgets; final total/receipt hash/patch hash are supplied in the worker return to avoid self-reference. Early count was 174 before test expansion; target 700 / hard 800 total and hard 500 non-test remain enforced. Exact patch hash recipe: concatenate `git diff --no-ext-diff --no-color --binary HEAD` followed by `git diff --no-ext-diff --no-color --binary --no-index -- /dev/null <path>` for each new file in lexicographic order, then SHA-256 the resulting bytes (no staging).
Frozen SHA-256: `decisions.py` `566c1053a94cb80ea76ce9548ced7d43da3957983b88a7b35c234341bfba8a25`; `attention.py` `1864a87bdba10e8b47792345f06c2ec5049df13bcbc3989227ac58440e11f267`; original Volta design `c4f95f0123b1a28603a3ec67214d029e597a20fb1d5d46853a77c23ecad89c6d`. Their tests and original DEC cases are unchanged. Accepted corrected offline candidate `4ff73485c` was not re-reviewed. No agent/review was launched; parent retains the allocated scoped feed-code review and future independent evidence pass without resetting earlier usage.

## Original Volta DEC-001…026 mapping

Original source remains `design-proposal.md`, Volta task `01a094c9-1364-7783-9493-9900d25f6fae`. Below F refers to `test_decision_feed.FeedTests.test_`, R to the unchanged `test_attention.DecisionRenderingTests.test_`, D to unchanged `test_decisions.DecisionTests.test_`. All are supporting automated evidence; original browser/human-observable cases are neither waived nor declared fully passed.

| Original case | Evidence and outstanding disposition |
| --- | --- |
| DEC-001 | F`chain_history_links_notices_facilities_and_repeat_assessment` proves full-page placement; R`dec001_005_full_context_exact_summary_links_and_purity` proves open-summary content. Desktop discoverability outstanding. |
| DEC-002 | Same F/R preserve full context and history across publications. Actual expand/collapse/reopen outstanding. |
| DEC-003 | Same F proves published exact current/historical destinations; same R covers two current sprints and similar IDs. Browser follow/return outstanding. |
| DEC-004 | Same R preserves stopped/continuing context; F`page_age_and_saved_link_javascript_without_browser_or_network` ages context. Away-from-computer readability outstanding. |
| DEC-005 | Same R proves question/options/recommendation text. Human interpretation outstanding. |
| DEC-006 | R`dec006_007_019_notices_separate_acknowledged_unresolved`; F chain preserves separate operations notices. Browser classification inspection outstanding. |
| DEC-007 | Same R and D`all_lifecycle_states_and_exact_answer_revision`; acknowledged remains unresolved. Interactive inspection outstanding. |
| DEC-008 | F`empty_missing_invalid_fresh_stale_and_new_assessment_chain` proves actual transported fresh-empty wording/count. Browser inspection outstanding. |
| DEC-009 | Same F plus F`immutable_generations_rollback_and_legacy_unknown`; missing/invalid/old-manifest unknown survives repeat publication. Browser refresh inspection outstanding. |
| DEC-010 | F chain repeat/publication and JS age tests expose stale feed/context. Desktop/phone interpretation outstanding. |
| DEC-011 | F empty-state chain distinguishes stale-empty from fresh-empty; JS test removes fresh-empty claim. Browser aging outstanding. |
| DEC-012 | F chain compares exact snapshot/history/input bytes and original assessment through repeated publication; F rollback restores original assessment. Browser reload evidence outstanding. |
| DEC-013 | F empty-state chain publishes genuinely newer assessment; R`dec008_014_freshness_unknown_and_mixed_age` retains older context/evidence warnings. Browser transition outstanding. |
| DEC-014 | Same R proves explicit unknown/mixed-age fields; JS test preserves unknown evidence time and user text. Browser inspection outstanding. |
| DEC-015 | F chain retains resolved stable anchor/revisions; JS hash test opens details; R`dec015_018_resolution_anchors_history_and_no_actions` covers queue transitions. Saved-link browser reopening outstanding. |
| DEC-016 | Same R preserves mixed open/resolved/superseded queue membership and history; F chain transports complete resolution history. Browser navigation outstanding. |
| DEC-017 | F chain and same R assert offline/Slack-not-connected/manager-answer language. No live connection or current-agent-running claim. Browser inspection outstanding. |
| DEC-018 | Same R proves no forms/inputs/buttons; repeated F publication compares unchanged input bytes. Actual keyboard/interaction proof outstanding. |
| DEC-019 | F chain preserves manager history notice; R`dec006_007_019_notices_separate_acknowledged_unresolved` covers notices beside empty and nonempty decisions. Browser inspection outstanding. |
| DEC-020 | F chain opens exact published historical explanation path; R`dec020_historical_identity_and_missing_context` covers absence without modern-provider substitution. Browser navigation outstanding. |
| DEC-021 | F`transport_rejects_secret_canaries_in_history_context_and_destinations`, F invalid-input and strict-snapshot tests, R`dec021_escape_and_reject_canaries_even_in_retained_history`; canaries absent from snapshot/pin/render output. Actual browser destinations/hover/accessibility inspection outstanding. |
| DEC-022 | Required desktop **1440x900** browser inspection remains outstanding, including long/multiple decisions, expansion and navigation. |
| DEC-023 | Required actual keyboard focus/order, details activation, links and no focus traps remain outstanding. Node DOM logic does not qualify this case. |
| DEC-024 | Required phone **390x844** browser inspection remains outstanding, including long text, stale input, multiple references and operability. |
| DEC-025 | Preserved advisory: multi-owner/long-evidence scan and readability remain outstanding; not deleted or waived. |
| DEC-026 | This receipt distinguishes automated fixture proof from all outstanding browser, server, deployment and live-integration evidence; independent evidence inspection remains required. |

## Parent handoff

Parent correction: scoped feed review found a destination-less export regression.
A new test reproduced publication failure with both missing and valid private
fixture input. Metadata-only export now omits a feed pin, preserving the existing
local publication path with explicit unknown decisions; it does not write a
snapshot into the repository. Destination-based generations retain full immutable
feed transport. Original worker hashes/counts above are historical after this
correction. Integrator adds one necessary correction review, retaining prior
usage and the pending independent evidence pass. Parent pre-correction full110
tests passed22.697s, including the actual HTTP-server regression; new final
tests/review are still required, not implied by that older result.

Final parent result: all111 tests pass24.609s, including real HTTP serving;
correction helper exits0/findings[], confidence0.90. Final source hashes:
export.py927a1e834697062fe5312b8101aac9d1dab705cc3ddfbbf4098f4841647da5b0,
test_decision_feed.py3e7242c70e4c4595b88ed3c7b8599f48fbfd5d7fc25b85dd614323d2508e8081.
Other source hashes unchanged. Final590changed/281non-test; browser remains pending.

Implementation fits the packet without a reslice. Parent owns scoped feed review/integration, the unchanged server regression when permitted, desktop/phone/keyboard qualification and independent evidence inspection, real state/publication, then the next bounded Slack alert/reply allocation. This does not claim full PF-80-S01 completion or unqualified human-test readiness. No actual server, SSH/sync/deploy, external network, credential access, Slack send/reply, posting flag change or real state creation occurred. Named-human acceptance remains absent. Native/TUI, TensorCash/Isometric and release/benchmarks are not qualified by this static internal document projection; plan records those repositories as inapplicable to this feature. No release is proposed.
