# Configured provider credential replacement

Bounded reliability fix restoring existing credential replacement from Providers.
Product authority: **Shipping MVP — LIVE**, **Multi-provider inference**:
“OpenAI, Anthropic/Claude Plan”; **Vault and credentials**: “Encrypted `/vault`,
masked entry, metadata-only inspection, and operational credential use without
placing raw values in chat.” No authorization or credential-storage boundary changes.
No plan or sprint is required for this bounded fix.

The reported production 0.1.42 session was stuck after saving an invalid Claude
subscription token: local presence marked it Configured/Active, the Providers
menu exposed only Deactivate, and the Claude intent adapter rejected Configured
status. A user could not reach the existing Replace flow to correct the token.

Configured providers now retain activation controls and expose their existing
account/API-key replacement methods. Configured Claude explicitly starts Replace,
so it cannot short-circuit as an already-configured Add. Existing source selection,
masked entry, cancellation, persistence, identity rules, and cache invalidation
remain responsible for the replacement. Active means eligible, not remotely validated.

Worktree: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix`.
Branch: `fix/provider-credential-replacement-20260912`, base `30436889d5`.
Application sources at this base match installed release `5f3a0ad7d7`.

Validation:

- `just fmt` passed; `git diff --check` passed.
- Focused `just test -p codex-tui -p codex-provider-auth --lib` expression in
  `tests.log`: 30 passed. Six reviewed snapshots cover Claude, Anthropic, and
  OpenAI replacement while active/inactive; keys dispatch the correct provider
  and method. The Claude intent test now expects Replace for Configured.
- An exploratory wider filter also hit an unrelated pre-existing status snapshot
  version/layout mismatch. Its received output is preserved separately; it was
  not accepted or counted as passing.
- Actual built 0.1.42 TUI, actual tmux keys, isolated homes, trace logging:
  bad saved token -> HTTP 401 -> open replacement -> cancel without changing
  revision -> malformed replacement rejected without changing revision -> valid
  fixture replacement -> successful response in the same process -> successful
  response after restart. Both TensorCash and Isometric Game passed. Paths and
  repository commits are in `pty-results.json`; rendered checkpoints are included.
- The final fixture uses a custom Anthropic-wire provider invoking the actual
  Claude credential helper and a loopback server, because built-in endpoints
  cannot be overridden. Thus provider-account replacement UI and credential/cache
  behavior are real; upstream Anthropic acceptance is simulated. An initial
  fixture attempt with an ignored built-in endpoint override sent only a dummy
  token and received a real 401; no production credential was used.

Local production recovery is authorized by the user's request to fix
`goodalexander`'s `corbanu --yolo` session. This is a host-local hotfix, not a public
release. No public release or benchmark completion is claimed. Named-human
acceptance is not recorded. Existing 0.1.42 bundle is retained for rollback.
Claude still requires a fresh valid subscription token or browser reauthorization;
the API-key credit error is a separate account balance issue.
