# PF-60-S02 — original-contract native golden increment

Test-only product initiative; Product measurement: “No commercial performance
numbers have been supplied.” Active plan portfolio-agent-cost-accounting, S02
in_progress; S03 dependent. Manager accepted internal-only N/A: no user control,
collection, live provider, TUI, code-blind execution or human acceptance is added.
The Corbanu skill/root1.7 and the two-path original-contract allocation govern.

## Candidate and immutable inputs

- Worker: /Volumes/CorbanuDrive/Corbanu/worktrees/accounting-contract-goldens-20260913
- Branch: workstream/accounting-contract-goldens-20260913
- Clean launch:34f0786e6c4f2ff94a3c260c95b3fc9c96cacc1a.
- Source base:81d0f90e77c1e9217a16e70fef1019ff9aa13753.
- Only new state/tests/accounting_contract_golden.rs and this receipt are written.
  All accepted runtime/import/support, manifests/locks and shared ledgers unchanged.
- S01 fixtures.json SHA256:
  96ba9416a8d0e69436fdd07c4ca6ddbb20cc30ece757b983eb1de2d708ca95dd.
- New Rust SHA256:3c2ab99d95bfa956e946c6e7f0787c490a529d92b31779081eedd079ded71483.
- Raw evidence E: /Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-contract-golden.KONSIg
  Final frozen-manifest.json and candidate.diff there contain exact file/log/JUnit
  digests, complete literal patch and command metadata. No overwritten attempts.

## Transcription and authority limits

All numeric inputs and rates are hand-invented S01 fixtures, not provider evidence.
UUID integers below mean Uuid::from_u128(n); no historical IDs are discovered.

| Original names (in order) | Native fixture mapping |
| --- | --- |
| root, child-a, child-b, history | Thread IDs11,12,13,14; actual public spawn edges11->12,11->13 |
| root-1, child-a-1, child-a-2, child-b-1, historical-priced, historical-unknown | Attempt IDs1..6, unchanged original dispatch dates/times and turn labels |
| request-root, request-child-a, request-child-b, historical-priced, historical-unknown | Request IDs101..105; attempt3 retains request102/retry_of2 |
| synthetic-openai-old, synthetic-openai-new, synthetic-anthropic | Snapshot IDs201..203; source_reference401..403 |
| synthetic-account + openai/synthetic-responses, anthropic/synthetic-messages, pfterminal-plan/synthetic-corbanu-api | Opaque scopes301,302,303; no actual account authorization |
| Nine original observation indices0..8 | Synthetic source901/sequence0..8; original revisions retained independently of arrival |
| Responses input_tokens/details cached_tokens/cache_write_tokens/output_tokens/details reasoning_tokens/total_tokens | Patch input/read/write/output/reasoning/total, Inclusive |
| Anthropic input_tokens/cache_read_input_tokens/cache_creation_input_tokens/output_tokens | Patch input/read/write/output, NativeAnthropic; omitted fields remain Missing |
| Chat prompt_tokens/details cached_tokens/completion_tokens/details reasoning_tokens/total_tokens | Patch input/read/output/reasoning/total, Inclusive; cache-write remains Missing |

Snapshot rates, USD/per-million unit and captured/effective intervals are exact
S01 transcriptions. Synthetic approved_at equals captured_at; SourceKind::NativeCatalog
simulates an existing typed descriptor, NOT authenticity or real approval.
Child-b explicitly Unpriced. Replay never applies today's catalog or fills unknowns.
Checkpoint indices[0,1,3,4]; reordered indices[8,5,1,0,7,3,2,6,4,5,0,2,8].
Provider response IDs, free-text status/provenance, invoice/allowance/balance are
not stored by these DTOs. Billed0.000300 is never added to estimated0.001084.
This tests arithmetic/persistence mapping, not every descriptive S01 field.

## Executed proof boundaries

Three normal-library external tests, no private implementation inclusion:
raw_replay_and_two_reopens persists a partial checkpoint, closes/reopens, replays
twice and again after two complete-state reopens. Exact complete DB unchanged on
duplicate delivery; all attempts/observations/source positions/bindings/snapshots
and each owner/day's full RetainedDay/DayTotals match explicit expectations.
original_price_and_native_family uses actual native descendants, four owned
attempts/three request IDs, exact retry edge and exclusion of independent history.
Individual known USD:0.000220,0.000228,0.000636,0,0.000110,0.000040; incomplete
attempts2,4,6 stay unknown. Old historical amount is not repriced to0.000220.
Family metrics (known/unknown), in native order:420/0,210/1,90/0,60/1,60/1,8/2,400/1;
known USD0.001084, unknown estimates2, attempts4. Noncached is the derived seventh
metric, not an invented explicit field. Exact fixture-only micro-USD summation
uses decimal text, no float and no new production aggregation API.
compact_original_bundle_parity compares young raw->maintenance and direct original
owner bundles at latest original dispatch+100days, independently against literals
then against all ten accounting tables. Snapshot/reference/fence/key sets exact;
all five raw tables empty. Two reopens/repeats leave complete DB and checkpoint
unchanged; a later read still reports NeedsMaintenance after all-suppressed replay.
No future-fixture public-clock deletion or time clamp. Accepted now-relative
deletion/OFF/GC/delayed-native-observer and365-day expiry tests are reused, not rewritten.

## Commands and retained results

All Rust commands run in worker/codex-rs with prefix P:
env RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true UV_OFFLINE=true CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/codex-rs/target
No competing build at launch; serial lease, no cache/dependency/denial repair.
Each E/*.time is literally start UTC / end UTC / actual exit, captured separately.
JUnit copied from worker codex-rs/target/nextest/local/junit.xml before next selector.

| Gate suffix after P | Result / evidence basename |
| --- | --- |
| sandbox-exec -p recorded one-file write/network profile just fix -p codex-state --locked |101; fix.log/time: TCP lock-listener bind denied OS error1; no successful fix claim |
| rustfmt --edition 2024 --config skip_children=true state/tests/accounting_contract_golden.rs |0 twice; format-01/02.log/time, stable imports_granularity warning |
| just test -p codex-state --test accounting_contract_golden --test-threads 1 --locked |First101 compile:5 diagnostics ThreadId not Ord/Metric not Copy; golden-01 log/time, compile-01-source.rs retained. Corrected only new test collection/borrowing. Final0:3/3,0skipped2.585s; golden-02 |
| just test -p codex-state -E 'test(accounting_late_import)' --test-threads 1 --locked |0:20/20,290filtered6.921s; late |
| just test -p codex-state -p codex-api -p codex-tasknode-session --test-threads 1 --locked |0:598/598,0skipped45.914s; shared |
| just test -p codex-core -E 'test(accounting) \| test(agent::role::tests) \| test(memory_stage_one::tests) \| test(exec_env::tests) \| test(tools::handlers::shell::tests)' --test-threads 1 --locked |0:100/100,3466filtered26.609s; core |
| just clippy -p codex-core -p codex-api -p codex-login -p codex-http-client -p codex-state -p codex-tasknode-session --locked --no-deps |0,1m50s; clippy; existing levels, not global -D warnings |
| cargo check --offline --locked -p codex-core -p codex-api -p codex-login -p codex-http-client -p codex-state -p codex-tasknode-session --lib |0,51.19s; check |

Governance python3 docs/plans/check.py and python3 docs/sprints/check.py (root)
passed3/115/126 before edits and on final Rust; final diff/patch checks exit0.
New target emits8 dead-code warnings from reused frozen support. Full warnings,
run IDs retained per raw log/JUnit/manifest. Final execution skips/LEAKs:0; Mac only.
Selector exclusions are not execution skips; overlapping gates are not unique tests.

## History, size and handoff

Original compact1970/463, corrected2099/515, review01 P2 then review02 clean.93,
all26 evidence hashes and separate metadata addendum remain immutable history.
Parent received81d0f90e7 with20/6/595/100 plus format/Clippy/check exit0; not rerun
as a new baseline here. All earlier failed full-Core gates/LEAKs remain unwaived.
Final Rust757 + receipt114 =871total/114non-test, all additions; target800/150,
hard900/200. Above-target formatting expansion was reported; no proof compression.
Manager owns one new material review plus necessary correction and receiving.
Target idle, lease released to manager at freeze. No worker review/children/commit/push.
Direct Anthropic provenance only; other actual routes/legacy acquisition remain
S02 gaps. Anonymous365-day fences are not permanent owner revocation or physical/
cross-DB erasure. Next manager unit must select a bounded actual remaining caller
from real interfaces, not infer provider coverage from these synthetic goldens.
Internal-only N/A is increment-specific. Later collection/replay/S03 handoff needs
independent isolated execution/actual probes and applicable TUI/live/human gates;
no full S02/S03/benchmark/release readiness is claimed.
