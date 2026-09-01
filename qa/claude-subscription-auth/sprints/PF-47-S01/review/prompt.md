# Independent final review: Anthropic account onboarding and Claude authentication

Review the complete branch diff from `origin/main` through exact candidate
`1055f423bee68d616a8271d87598ad982926b6bd` in
`/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated`.
Use read-only inspection only. Do not edit files, invoke another reviewer, or
expose any credential value.

The user-requested behavior is:

- Fresh first-run onboarding shows an Anthropic Claude account as an account
  choice, not merely an Anthropic API key.
- Choosing it opens the same explicit two-method flow used by `/providers`:
  the approximately one-year `claude setup-token` subscription token is the
  recommended default, and Claude Code login is the compatibility choice.
- Provider selection persists only after successful authentication. Cancel,
  failure, retry, recovery, and restart never silently select another source.
- Masked token paste removes CR and LF characters introduced by wrapped
  clipboard text, while preserving and rejecting other invalid whitespace.
- Existing optional Claude credential fields remain optional during legacy
  record deserialization.
- Native Claude Plan panes never inherit the real selected token. Corbanu
  reveals it only at execution and uses it inside a loopback broker. Every
  bridge turn has an unguessable capability that must authenticate before an
  upstream request can use a provider credential. The client capability is
  replaced by the real bearer token upstream and is never persisted.
- First-party Claude OAuth passthrough forwards real `/v1/messages/count_tokens`
  requests and required OAuth beta headers; compatibility passthrough bridges
  retain their existing synthetic count behavior.
- Raw credentials must not enter config, chat, rollout history, logs,
  snapshots, errors, debug output, command arguments, child-process
  environments, artifacts, or audits.

Earlier Opus and structured reviews identified and the candidate claims to fix:

1. missing `#[serde(default)]` on custom-deserialized optional Claude tokens;
2. inheritance of `CLAUDE_CODE_OAUTH_TOKEN` by model-controlled Claude tools;
3. unauthenticated localhost access to the credential bridge; and
4. synthetic token counts accidentally applied to first-party Claude Plan;
5. hidden Anthropic onboarding bypassing forced-provider policy;
6. duplicate per-request auth subprocesses instead of selection-revision caching;
7. legacy env-first migration and explicit-selection fail-closed gaps;
8. cache revisions not changing when managed tokens were removed through `/vault`;
9. hidden auth helpers invoking `codex-tui` instead of the multitool CLI; and
10. otherwise valid Claude status payloads being rejected when optional `orgId`
    or `subscriptionType` metadata was absent;
11. proactive auth refreshes contending with unrelated startup vault access,
    including an unconfigured Telegram connector; and
12. generic or bulk vault deletion bypassing the managed Claude token's auth
    revision invalidation.

The exact binary under review is `codex-rs/target/debug/corbanu`, SHA-256
`7f531be58806c1f9b129b37428ce73cfb0b996306dd1064ca42ba4415f15bb46`.
The true-Tmux first-run scenario passed 1/1, and the managed plus compatibility
scenarios passed 2/2 against that binary immediately before this review.

Inspect implementation, tests, snapshots, docs, and QA harnesses. Pay special
attention to authentication precedence, bridge request authorization and
header replacement, endpoint/path restrictions, count-token routing, secret
custody and zeroization, transaction rollback, refresh/401 races, subprocess
cancellation/timeouts, TUI state transitions, persisted-selection
compatibility, and whether the tests prove their no-disclosure/no-fallback
claims.

For every actionable finding, report:

1. severity `P0`, `P1`, `P2`, or `P3`;
2. exact file and tight line range;
3. a concrete failing scenario;
4. why existing tests do not catch it; and
5. the smallest safe fix.

Do not report style preferences, speculative hardening, or issues outside this
branch diff. End with exactly one standalone verdict line:

`CLEAN`

or

`CHANGES REQUIRED`
