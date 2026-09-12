# PF-58 review scope — 2026-09-08

Original request: identify the credential rejected by codex_apps and generalize
provider-specific reauthentication, then supply a replacement human-test build.
Branch: feat/provider-reauth-health. Combined base: 472b8fed5 (includes main
3cec54d99 / 0.1.41 and the previously qualified security integration).
Review only the local PF-58 patch, not inherited security/release history.

Boundary: typed HTTP401 startup attribution, session-local credential health,
source-appropriate /providers recovery and existing auth adapters. Preserve
activation, current model and billing context. No live credentials in tests.
No automatic account/source fallback. Generic403/429/network errors are not
credential rejection. Cancellation must retain the failure. Successful replacement
must permit another request/reconnect without restart. External sources receive
instructions rather than a managed credential override.

Review budget: Astra High and Fable5.1 High via Corbanu/TMUX; five total maximum.
Four reviews completed (two Astra High and two Fable5.1 High). This is review
five, the final integration review. Focused final TMUX tests are running; do not
infer a pass. Astra's second review was clean. Fable's second found the
host-recreation ledger gap, now fixed by reusing the model policy host when the
temporary setup handle is absent. A real-TMUX account-success path also found
the old auth modal covering the updated manager. Correlated completion now
dismisses only the named shared account-auth view in management and setup.
Review these final fixes and their regression tests, plus the complete patch.
Review the complete current PF-58 diff against HEAD, including subsequent
protocol 401 classification, environment ownership, repeated-rejection guard,
narrow snapshot, and the corrected synthetic account/key fixtures. The inventory
below is the original freeze, not an instruction to omit these bounded additions.
The account fixture uses built-in `openai_base_url`/`chatgpt_base_url` controls;
reserved-provider and trusted-MCP-origin guards remain intact. The harness menu
parser must distinguish submitted prompts from the active selection.
The live user's new candidate has shown OpenAI "Credential needs attention",
Claude configured/current, and the `r` -> "Sign in to OpenAI again" menu.
No live sign-in has been attempted on the user's behalf.

Known broader failure outside this patch: the inherited native-child convergence
case fails with "source admission policy is unavailable" after combining main
and the security integration. This is a disclosed candidate limitation, not a
passing release gate. Do not expand PF-58 into a source-admission redesign.
Non-test Rust changed lines: 568. This exceeds the small-change guideline;
reviewable stages are (1) typed transport/health contract, (2) TUI adapters and
recovery, (3) regression proof. None is independently advertised as a finished
feature; qualification applies to the combined candidate.

Latest-main integration preserved both plan histories. The sprint checker also
reports upstream duplicate PF-43/44/45 IDs and absent feature/backlinks, plus an
inherited PF20/PF43 order collision. Those are not resolved by PF-58. Its own
execution order has been moved from 17 to 22 to avoid the new PF57-S02 collision.

Frozen changed-file inventory (added/deleted lines):

```text
-	-	codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst
2	1	codex-rs/app-server-protocol/src/protocol/v2/mcp.rs
8	0	codex-rs/app-server/README.md
12	3	codex-rs/codex-mcp/src/connection_manager.rs
94	0	codex-rs/core/tests/suite/mcp_auth_refresh.rs
2	0	codex-rs/protocol/src/protocol.rs
7	3	codex-rs/provider-auth/src/api_key_flow.rs
111	0	codex-rs/provider-auth/src/credential_health.rs
143	0	codex-rs/provider-auth/src/credential_health_tests.rs
2	0	codex-rs/provider-auth/src/lib.rs
1	3	codex-rs/provider-auth/src/management.rs
1	1	codex-rs/provider-auth/src/management_tests.rs
9	7	codex-rs/provider-auth/src/status.rs
4	0	codex-rs/provider-auth/src/status_contract.rs
1	1	codex-rs/provider-auth/src/status_tests.rs
4	3	codex-rs/rmcp-client/src/http_client_adapter.rs
1	1	codex-rs/rmcp-client/src/rmcp_client.rs
32	7	codex-rs/rmcp-client/src/startup_error.rs
6	2	codex-rs/rmcp-client/tests/streamable_http_recovery.rs
11	0	codex-rs/tui/src/app/event_dispatch.rs
45	5	codex-rs/tui/src/app/provider_management.rs
3	1	codex-rs/tui/src/app/tests.rs
7	0	codex-rs/tui/src/app_event.rs
2	0	codex-rs/tui/src/chatwidget.rs
3	0	codex-rs/tui/src/chatwidget/protocol.rs
91	0	codex-rs/tui/src/chatwidget/provider_health.rs
96	0	codex-rs/tui/src/chatwidget/provider_health_tests.rs
19	3	codex-rs/tui/src/chatwidget/provider_manager.rs
98	0	codex-rs/tui/src/chatwidget/provider_recovery.rs
10	0	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__provider_health__tests__openai_reauthentication.snap
3	2	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__provider_manager__tests__provider_manager_shared_status.snap
20	0	codex-rs/tui/src/provider_account_auth_host.rs
2	0	codex-rs/tui/src/provider_auth_effect_executor.rs
47	0	codex-rs/tui/src/provider_status_host.rs
3	3	codex-rs/tui/tests/suite/provider_convergence.rs
25	13	codex-rs/tui/tests/suite/provider_management.rs
196	0	codex-rs/tui/tests/suite/provider_reauthentication.rs
5	2	docs/plans/active/unified-provider-auth.md
1	1	docs/sprints/current/unified-provider-auth/index.md
2	2	docs/sprints/current/unified-provider-auth/pf-58-s01-credential-health-and-reauth.md
1	1	docs/sprints/index.md
22	0	humanTest.html
31	0	qa/provider-auth/pf-58/qualify.sh
```
