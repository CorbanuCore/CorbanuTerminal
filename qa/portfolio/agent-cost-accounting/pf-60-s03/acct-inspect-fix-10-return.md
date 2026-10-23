# RETURN — acct-inspect-fix-10

Allocation digest `d7617a972bc5c725ee6f418964ed1bdc927e59aa0a5efa0a4feaee4c25b403f2`; claim `0f5f61fa-ff98-4b03-9852-19936cab57c0`; gpt-6-astra/high.
Brief SHA-256 verified: `4255ff9a12a31b6f33eeffb27ab57ad2451e32712aa28e88218e5bdbe4e3bba7`. Review body read.
Base `d468b305d572a7ebde99eea7490ad6bc2a13bb43`; branch `bootstrap/acct-inspect-20260915`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915`. Output is the commit containing this receipt; its SHA is in the worker return.
Classification: bounded wording revision within active PF-60 / in-progress PF-60-S03. Product heading **Product measurement**, excerpt “No commercial performance numbers have been supplied.” The frozen assignment authorizes correction of the two disclosure findings.

Coverage wording: `Effective coverage (requested ∩ aggregate retention ∩ snapshot) for {bounds}: {effective}` on the overview; `Bucket: {bounds}; effective coverage (requested ∩ aggregate retention ∩ snapshot): {effective}` on bucket pages. Naming the intersection avoids claiming a particular clipping cause. Both request-only and aggregate-floor clipping retain their exact effective bounds; detail availability remains separate.
Range-wide ancestry disclosures receive `Range:`; bucket-local disclosures receive `Bucket:`. The regression checks differing range/bucket counts together on the first bucket page: 2/1 inspectable attempts and 5/2 unavailable ancestry entries.

- `accounting_inspect_range_coverage_names_intersection_for_request_and_retention_clips`: fails before (missing intersection label); passes after, with request-only and genuine aggregate-floor clipping assertions on overview and bucket pages.
- `accounting_inspect_range_bucket_ancestry_counts_have_explicit_scopes`: fails before (four unscoped strings); passes after with differing counts and explicit scopes.
- Final inspector set: **69/69 passed**, including all 67 existing cases plus both new cases; 4,420 unrelated tests filtered out. State/TaskNode: **422/422 passed**, no skips; one leaky marker in unchanged `codex-state extract::tests::completed_user_message_items_set_title_and_first_user_message`. These runs cover 459 distinct tests (32 state inspector tests overlap).

Commands below ran from `codex-rs`; each test command used `> ../qa/portfolio/agent-cost-accounting/pf-60-s03/<log> 2>&1`.
| Exact command | Log | Exit |
| --- | --- | --- |
| `INSTA_UPDATE=no just test -p codex-tui -E 'test(accounting_inspect_range_coverage_names_intersection_for_request_and_retention_clips) \| test(accounting_inspect_range_bucket_ancestry_counts_have_explicit_scopes)' --locked --offline` | `fix-10-before.log`, then `fix-10-before-aligned.log` | 100 each; both tests failed in each run |
| `INSTA_UPDATE=no just test -p codex-state -p codex-tui -E 'test(accounting_inspect_)' --locked --offline` | `fix-10-final-inspectors.log` | 0 |
| `just test -p codex-state -p codex-tasknode-session --locked --offline` | `fix-10-final-state-tasknode.log` | 0 |
| `rustfmt --edition 2024 --config skip_children=true tui/src/chatwidget/tokens.rs tui/src/chatwidget/tokens_tests.rs` | Scoped formatting before final tests; nightly-only imports_granularity warning | 0 |

Root commands `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`, `git diff --check`, `git status --short`, and `git diff --exit-code -- codex-rs/state/src/runtime/accounting_store.rs`: all exit 0. Status after formatting showed only owned files. No accounting calculations, caps, candidate bound, collection, retention, freshness, snapshot, connection, lock, transaction, schema or index changes.
Log SHA-256, in table order: before `82d4a76fb22101bd40031704b2db83a08aa557ad555e22b79e8e8d0438c19afc`; aligned before `3d1adc2627323c4fc401cec17ced91b60161c3c403e79cb3cc1732cb1cf88c0a`; inspectors `6443a4e5970c121a4532ce7f0526c78a5f6d2bbb7af6b2d5a157dc116725e388`; state/TaskNode `db627bad53734c9e1da61528940c5fba3a05b4cc73231a20af47d4293229de1b`.

Changed lines (added/deleted): `tokens.rs` 9/4; `tokens_tests.rs` 109/12; sprint ledger 1/1; this receipt 30/0. Total **166**, including **13 production** and **45 outside tests**. All four paths are allocated.
All Rust tests use this checkout's guarded `just test` after reading `docs/development/test-isolation.md`. Logs are ignored local artifacts beside this receipt. No live profile, native credential access, push, full TUI suite, or workspace formatter is authorized or used.
Both pre-change runs failed both new tests (exit 100); the second used corrected calendar-aligned January bounds. Both logs are preserved; neither is functional qualification.

Receiving review/integration, true-TUI keys, isolated independent code-blind design/execution/evidence review, live-repository qualification and named-human acceptance remain open under S03. No human-test or release readiness is claimed; no shipped-feature documentation, release or benchmark claim is made. The Fable receiving owner retains the functional handoff gate.
