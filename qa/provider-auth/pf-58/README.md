# PF-58 credential health and recovery — human-test candidate

**Current: September 10 signed picker/health replacement.** See the
[current repair record](picker-repair-20260910.md) for the installed Mac shortcut,
final RTX package, source identity and passing regression/TMUX evidence. It
supersedes both the September 8 package and the earlier September 10 Mac copies.
Two existing-profile Mac menu launches completed without password entry; human
acceptance and the explicitly yellow prerequisites remain open.

**Historical September 8 product repairs.** The incomplete replacement had been
superseded by `candidate-repaired-qualified/bin/codex`, with matching companions,
installed-plugin runtime support, native-pane policy inheritance and corrected
Claude recovery/configuration reloads. See [current repair evidence](product-repairs-20260908.md)
and [the mandatory handoff gate and remaining coverage gaps](handoff-readiness.md).
The historical scoped passes below are retained as history, not final-package acceptance.

This candidate corrects misleading configured-provider status after a typed
authentication rejection and adds `r` recovery from `/providers`.

## Identity and scope

- Feature/sprint: PF-58 / PF-58-S01, product initiative.
- Product: **Shipping MVP — LIVE**, “Encrypted `/vault`, masked entry,
  metadata-only inspection, and operational credential use without placing raw
  values in chat.”
- Branch: `feat/provider-reauth-health`; local worktree recorded in the sprint.
- Combined base: `472b8fed5`, includes main `3cec54d99` (0.1.41) and the prior
  security integration. Main is not modified by this work.
- RTX mirror: `/home/travis/worktrees/provider-reauth-health-final`.
- Evidence: `/home/travis/security-round5/evidence/provider-reauth-health-final`.
- Prior human binary `combined-b12e32d` does not contain PF-58.

## Credential diagnosis

The human session used `/home/travis/.corbanu`, not the separate `.codex` store.
The affected credential is its OpenAI account credential used by `codex_apps`.
The service rejected the access token; metadata-only logs also recorded failed
refresh attempts including `refresh_token_reused`. No vault-token expiry date was
verified. The earlier date from `.codex/auth.json` was unrelated and retracted.
Claude credentials were not changed. No raw credentials are stored in this report.

## Review ledger

Limit: five reviews. Five completed; no additional review is claimed beyond the
user's cap. Findings and their dispositions are retained in [reviews/](reviews/).

1. Astra High: accepted OpenAI environment-key ownership finding; now projects
   the effective environment source instead of offering a managed replacement.
   Accepted misleading external-retry guidance; now explicitly requires owning
   tool renewal followed by restarting Corbanu. In-process external revalidation
   is not implemented or promised. Accepted missing narrow snapshot; generated
   rendering inspected and passing.
2. Fable5.1 High through Corbanu and TMUX: accepted repeated-rejection ledger
   overwrite; `begin` now ignores already-rejected status projections and a
   regression asserts failure survives rediscovery. Also found the same missing
   narrow snapshot. No architecture expansion required.
3. Astra High: no actionable findings in the then-current patch.
4. Fable5.1 High through Corbanu and TMUX: accepted the low-severity host-recreation
   finding. Reopening setup/management now falls back to the model policy's
   existing status host, sharing rejection and recovery state after the temporary
   setup handle is cleared. A regression proves recovery clears both views.
5. Astra High: accepted configured-account reauthentication no-op. The shared
   controller now has an explicit `Reauthenticate` action used by provider
   management; normal setup remains idempotent and externally owned credentials
   remain blocked. A controller regression and a real-TMUX configured-account
   sign-in/cancel check cover this final correction. This correction is tested,
   not independently re-reviewed; a sixth review was not run under the user cap.

True-TMUX subsequently exposed a successful-account-login popup remaining above
the refreshed manager. Correlated account completion now dismisses that specific
auth view in management and setup. It never dismisses unrelated popups. This and
the final fixture corrections are included in the last review round.

Review files are retained under the external drive's
`.codex-work/provider-reauth-health/reviews/` directory. These fixes remain within
the frozen scope in [review-scope.md](review-scope.md).

## Qualification and limitations

The 0.1.41 transport/contracts passed: provider-auth 71, protocol error mapping 40,
login metadata 3, HTTP recovery 13, MCP startup 12, core MCP auth refresh 2, and
app-server protocol 287 (one ignored). Final TUI units: 27 passed. Final true-TMUX:
**6/6 passed**, run `f251f0f5-0f8f-4556-ac37-d2c4ab29ec28`, including account
rejection/cancel/relogin/new-token request and configured-account manual sign-in,
API-key replacement/new-key request, both environment ownership cases, the menu
parser regression, and Claude recovery/cancel/retry. Synthetic credential canary
checks pass; custody files alone are exempt from the transcript/log scan.

## Previous replacement identity (superseded)

- Binary: `/home/travis/security-round5/evidence/provider-reauth-health-final/candidate-replacement/codex`.
- Version: `corbanu 0.1.41` (Linux RTX build).
- SHA-256: `37511bb9348e3793255ce67806d23060f3436cee53ef0a041b921420b201da35`.
- Changed `codex-rs` file-content manifest SHA-256, identical locally and on RTX:
  `fc050adedb55af4395397b69f438b5f3be3f98b228a9c1873b7b1c5949005a2e`.
- Base: `472b8fed5`, main incorporated `3cec54d99`. Source branch remains separate
  from main. Human acceptance and release sign-off are not claimed.
- Source commit: `2545eedee`, pushed to `origin/feat/provider-reauth-health`.
  Later handoff bookkeeping does not change the binary's source files.
- Attach: `ssh -t travis@100.99.88.49 'tmux attach -t corbanu-human-pf58-final'`.
- Scope: [human checks 24–26](../../../humanTest.html#reauth-title).

Build/fix/format evidence: `fix-replacement.log`, `fmt-replacement.log`,
`provider-auth-replacement.log`, `tui-replacement.log`, `build-replacement.log`
and `tmux-replacement.log` in the RTX evidence directory above. Earlier unchanged
transport suites are recorded in `error-protocol.log`, `login.log`, `rmcp.log`,
`mcp.log`, `core.log` and `protocol.log`. Candidate viewport/scrollback evidence
is copied into [evidence/](evidence/); no real account credentials are included.

## Historical integration failures and remaining qualification limits

The broader 28-check run on the preceding build passed 25 and failed three:
the account popup above, a Claude fixture using different keyring backends in
parent and child, and native-child convergence reporting `source admission policy
is unavailable`. Matching the parent's isolated keyring setting to the harness
child makes the Claude case pass. The later native-child repair is now included
in the expanded pinned-package matrix; no case was excluded to hide the failure.
Full platform/live qualification and human sign-off remain separate release gates.

The first real-TMUX run caught an additional protocol gap: ordinary model HTTP
401 errors were mapped to `Other`. The final patch maps only definite 401 to the
existing `Unauthorized` variant and adds protocol-level coverage for 401 versus
403/429/503. Nested reauth tests also missed the provider-journey timeout rule;
they now use its existing bounded three-minute budget.

The loopback account fixture must not weaken MCP's trusted-origin guard. That
guard correctly declines to forward account credentials to an arbitrary loopback
MCP server. The account TUI flow therefore uses the explicitly configured model
endpoint, while separate MCP runtime tests prove account attachment/attribution
and refreshed-token selection. Live-service reauthentication remains a named-human
check; no live account is signed in on the user's behalf.

No human acceptance is claimed. Live repository workflows and release
benchmarks are not rerun: this feature changes home-level auth, not project
execution, and is not a release publication. Upstream planning conflicts remain:
duplicate PF-43/44/45 sprint IDs, missing PF43/44 feature/backlinks and the
PF20/PF43 execution-order collision. PF-58's own order was moved to 22.
