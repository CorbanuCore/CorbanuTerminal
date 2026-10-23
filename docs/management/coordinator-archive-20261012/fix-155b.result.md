rust-ci-full is green again: run 37215512834 on `1764c5b297` passed on the first CI round. All four test shards pass (17,457 tests, no flaky retries). On PR #155 itself, 53 checks pass, none fail and CodeQL is neutral. Five were still running when I stopped, 2.5 hours in: four v8-canary release and ptrcomp-sandbox builds, which compile V8 from source, and CodeQL Analyze (rust).

The failures came from code in main's big integration merge, plus one clash with this branch's `.git/HEAD` rule. Main's own postmerge-ci fails on the same tests.

| Failure | Cause | Fix (commit) | Verified how |
|---|---|---|---|
| 6 app-server tests that resume, read or delete paginated threads, all failing with `paginated_threads is not supported yet` | **Real bug.** `3e4dad7717` added a step to paginated cold resume that reads the thread's full history to recover its service tier. The local store refuses full-history reads for paginated threads, so any such resume failed when the thread's settings weren't in its recent history. | `9d89d44b78`: when the store reports `Unsupported`, the resume skips that step and uses the recent history, as it did before the step existed. Other store errors still fail the resume. | All 6 failed on the box before the fix and pass after |
| 3 `pf_60_s03_*` core tests: `no such table: draft_accounting_attempts` | `1aca553905` stopped recording usage for ephemeral sessions (no saved thread). These tests built their session without a saved thread, so nothing was recorded. | `d7869e7e8e`: the tests give their session a saved thread, as a real session has. Assertions unchanged. | Pass on the box |
| `home_dot_codex_is_not_a_project_layer_inside_a_repository` | This test came from main and creates an empty `.git`. This branch only counts `.git` as a repo when it contains `HEAD` (upstream #39629). | `25b7e30c3f`: the test writes `.git/HEAD`. | Passes on the box |
| `synthetic_policy::exact_wire_packets_admit_without_rewriting` | A full-workspace build turns on serde_json's `preserve_order` feature, so re-serializing a packet gave back its exact bytes and the "equivalent JSON is rejected" check failed. | `58be0b3a8f`: compare against pretty-printed JSON, which always has different bytes. Same intent. | Passes on the box |
| `accounting_responses_ws_native_auxiliary_scope_and_event_parity` timed out at 60s | One test ran compaction plus 72 fresh-session cases. | `1764c5b297`: the same 72 cases are split across 4 tests of about 8s each. No timeout changed. | Pass on the box |

**Linux verification** (RTX box, fresh clone, temp dir on ext4, the CI test profile and stack size):
- The targeted set passes 85/85, and the wider core accounting, client, config and session tests pass 513/513.
- Clippy with warnings as errors is clean on the three crates I touched.
- The full app-server suite passes except 7 tests that need helper binaries only CI's test archive provides; those passed in CI.
- The box clone and its temp files are cleaned up, and no stray `.git`/`.codex`/`.agents` directories are left in `/tmp`.

I pushed normally to `codex/fix-rust-ci-full-20261004` and the mirror `codex/full-ci-20261004`, and posted a summary comment on the PR (#issuecomment-5982215226). Not merged.

One side effect of the resume fix: a paginated thread whose recent history has no saved settings still resumes, but without its saved service tier. That tier recovery never actually worked for paginated threads. Making it work would need a new thread-store read, which I left out of scope.