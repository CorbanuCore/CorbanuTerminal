# PF-84-S05 gate: per-account usage and rate-limit attribution

Candidate: branch `feat/pf-84-s05-usage-by-account`, code at `a1921d736b`. Change class: product
initiative (plan PF-84). Acceptance row "Per-account usage" in
[the plan](../../../docs/plans/completed/main-2026-10-14-unified-provider-auth.md).

## What shipped

Account selection is behind the default-off `named_accounts` feature (S03), so with the feature
off no attempt names an account and nothing below changes.

- **Ledger (developer accounting, PF-60 conventions).** An accounted attempt sent on a named
  account records `account: {name, fingerprint}`: the name and the vault's 12-hex fingerprint,
  salted per home. It never records a credential value. Default-account attempts omit the field,
  so every older attempt keeps its exact stored bytes and still reads as `default`. The new
  migration `0004_account_dimension.sql` is ledger format 4. A ledger is upgraded only when the
  first named attempt is written, in the same transaction, so a ledger with only default work
  stays at its current format. An older build refuses a format-4 ledger by name
  (`NewerLedgerFormat`), as it already does for any newer format. The account is not stored in
  compacted days, which already drop provider and model.
- **Core.** `AccountingMode::Provider` carries the serving provider's account and is rebound on
  every turn. The fingerprint is read once per attach from the vault (hash only). That read never
  creates a vault, key or salt, and is limited to 2 s. If the fingerprint can't be read in time,
  the account is recorded by name alone and the request is never held up.

  Follow-up #474: the read used to create the salt on a vault made before S05, rewriting it on
  the first named request. The salt is now created only when an account is added or renamed, in
  that same write; until then such a vault's accounts are recorded by name alone.

  A sampling refuses a rebound mode on another account. A fingerprint is reused only for the
  same provider and account. Requests reported by another client (the Claude panes bridge)
  record no account.
- **Rate-limit state.** The throttle row for the 429 cooldown and request lease is keyed by
  the credential (owner decision 2026-10-11, providers limit by key): a named account by
  `account-key:<fingerprint>`, the vault's per-home salted fingerprint above, so two names holding
  one key share a cooldown and lease, and two keys of one provider never do. The fingerprint is
  read as the ledger reads it (never creating or writing the vault, at most 2 s, reused for a
  minute). An account without one, because every account in its vault predates S05, is keyed by
  `account:<name>` until the next `corbanu account add` or rename; so is an account whose vault
  has not answered yet (a slow or unreadable vault), until a read succeeds. An active cooldown
  or lease kept under `account:<name>` moves to the fingerprint row on the account's next
  request (the later cooldown wins when several names merge), and the name's rows are dropped. The default
  account's key is fingerprinted with a per-home salt kept in the state DB (#482, follow-up); an
  older unsalted row moves to it on the key's next request, and idle ones are deleted.

  Follow-up #476: until then only a leased request recorded its result, so no human-driven
  session ever recorded a 429. Every turn sampling request now records it: a 429 holds that key
  back in every session on the home for the provider's `Retry-After` (else 30 s, 60 s, 120 s, up to
  5 min; a burst of 429s counts once), and no result shortens an active cooldown. Follow-up #509:
  compaction, the turn-completion check and memory extraction now respect an active cooldown on
  their key and record their own results the same way; they never take the lease. The request
  lease is taken only by a spawned sub-agent sending a large request (32k input tokens or
  128 KiB) whose cache hit is unknown or under 70%; human-driven sessions never wait on it.

  The default account (an environment key) and a named account are keyed apart even when they
  hold the same key: the default key is fingerprinted with the state DB's salt, a named one with
  the vault's.

  Follow-up #510: Z.AI's transport retry of a 429 (kept for its transient overloads, which send
  no `Retry-After`) now waits out a requested wait of up to 5 s, read as the throttle reads it,
  and stops at once on a longer one, so the throttle's cooldown starts on the first 429 instead
  of after five quick retries.

- **`/cost`.** Rows, provider pages and request links are grouped by provider, account and model.
  The "Other conversations" rows are grouped the same way. Accounts are named, `default`
  included, only when the view holds work from a named account. The per-account rows add up
  exactly to the provider totals.

  Follow-up #475: a request the provider refuses with 401 or 403 before any response (a fake key)
  is recorded as refused (ledger format 5, written only with the first refusal). `/cost` shows it
  as refused with no charge, not as incomplete usage that sends the user to the provider's bill.
- **`/status`** names the session's account, which S04 already did.

## Tests (final tree, macOS, `just test`, isolated profile)

| Command | Result |
| --- | --- |
| `just test -p codex-state` | 407 passed |
| `just test -p codex-core [--features developer-accounting] -E 'test(/accounting\|throttle\|provider_request\|named_account/)'` | see the [PR](#pr) (final-tree rerun); at `70739ae013`: 198/199 and 193/194, with the one failure being the new admission test's format assertion, fixed in `a1921d736b`; then `-E 'test(/basis_tests\|named_account/)'` 10/10 both ways |
| `just test -p codex-tui --features developer-accounting -E 'test(/tokens\|cost\|usage\|status\|accounting\|account/)'` | 620 passed, 3 failed (environmental) |
| same without the feature | 619 passed, 3 failed (environmental) |

The 3 TUI failures are `multi_provider_onboarding::tmux_corbanu_env_*`. They fail with "path
must be shorter than SUN_LEN": the Unix socket path is too long on this drive. Earlier lanes
recorded the same failure; the code is outside this change.

New tests:

- the pre-format-2 fixture ledger moves 3 → 4 on a named attempt, and the bytes of every
  older attempt are unchanged;
- a format-1 ledger moves straight to format 4, and default-only work never reaches format 4
  (Chat integration test);
- account validation (`default` is never a stored name);
- one account's cooldown never blocks another, and the throttle key per account;
- admission binds the name and fingerprint; no file in the home holds the key;
- with no vault, the account is recorded by name only and nothing is created;
- `/cost` grouping, with exact per-account reconciliation of every token measure;
- a default-only view never names accounts.

Clippy `-D warnings`: clean on macOS and on Linux (glitch, `-j10`) for `codex-state`,
`codex-core` and `codex-tui`, with and without developer-accounting, at `cc38fa808c`,
`70739ae013` and `a1921d736b`.

## Live run (tmux, GLM 5.3 Flash, debug developer-accounting build of `70739ae013`)

Scripts: [`s05/scripts/`](s05/scripts/). The run used a disposable home and
`CORBANU_TEST_NO_NATIVE_KEYRING=1`. Real keys came only from `corbanu vault auth-helper`, inside
the consuming command. Captures: [`s05/captures/`](s05/captures/).

- **Accounts enrolled:** Z.AI `work` (the real default key under a second name, since no second
  real key exists), Z.AI `fake` (a fake key) and Kimi `alt` (the real Kimi key under a second
  name).
- **Coordinator:** on the default account, it replied `pong`. Spawned agents: `(glm-5.3-flash
  max · account work)` replied `pong`; `(… · account fake)` got a 401.
- **Workers:** `corbanu exec` workers in the same home. `--account work` replied `pong`
  (7,173 tokens) and `--account fake` got a 401 (exit 1). Kimi default and `--account alt` both
  replied `pong`.
- **`/cost`** ([rows](s05/captures/tui-cost-rows.txt)):
  - This conversation: `Z.AI · account default` 6 requests, 46,115 tokens; `account work`
    1 request, 4,371 tokens; `account fake` 1 request, "tokens not reported … usage
    incomplete".
  - Other conversations: `Z.AI · account work` 7,173 tokens, `Z.AI · account fake`,
    `Kimi Code · account alt` 7,246 tokens and `Kimi Code · account default` 7,232 tokens.
  - The fake account's failure is counted as one request with no tokens and no spend: the
    total says "at least … (1 attempt had incomplete usage)", never zero.
- **Exact reconciliation** ([reconcile.txt](s05/captures/reconcile.txt)): each thread's
  provider-reported cumulative usage was summed per (provider, account) from its rollout. It
  equals the ledger and `/cost` exactly:

  | Provider · account | Tokens |
  | --- | --- |
  | Z.AI default | 46,115 |
  | Z.AI work | 11,544 (4,371 + 7,173) |
  | Z.AI fake | 0 |
  | Kimi default | 7,232 |
  | Kimi alt | 7,246 |

  The ledger was at format `[1, 2, 3, 4]`. It held three distinct account records (name and
  fingerprint) plus default.
- **Throttle rows:** `account:work`, `account:fake` and `account:alt`, each separate from the
  default `env:…` rows.
- **Leak scan:** the real Z.AI and Kimi keys and the fake key appear 0 times in 5,560 files
  (home, captures, workspace). The run folders were deleted after recording.

## Video (SOP, leak-scanned: 0 literal, 0 pattern hits)

- [usage grouped by provider and account](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s05-pf84-usage-by-account-a1921d736bfa-2026-10-11.mp4)
  (38 s). It ends on `/cost` with `account default` (49,061 tokens), `account work` (4,701)
  and `account fake` (1 request, tokens not reported).

## Independent review (Opus 5.5 High, read-only `corbanu exec`)

One pass at `cc38fa808c`: **APPROVE**, with no blocking findings. The non-blocking items were
handled in `70739ae013` and `a1921d736b`:

1. The fingerprint read is limited to 2 s and its doc was corrected.
2. A mid-sampling account guard was added.
3. A fingerprint is reused only for the same provider and account.
4. `/cost` makes one account-naming decision per view.
5. Name-keyed throttling is documented above.
6. Tests were added for the missing vault, format 1 → 4, the guard and request links.

Not done:

- (6c) per-path account integration tests for compaction, websocket and extensions. They all
  share the `turn_mode`/`attach_scopes` binding that the unit test covers.
- (6e) using an in-memory keyring in the core test. The production read path is `Vault::new`,
  and `just test` sets `CORBANU_TEST_NO_NATIVE_KEYRING`.
- (7) ChatGPT fingerprint drift. That code is outside this change; it is recorded for S06.

## Known limits

- Per-account usage views exist only where accounting does: developer builds (PF-60,
  developer-only by Travis's decision). Release builds show the session's account in `/status`
  and keep per-account throttle state, but have no usage view.
- Separate homes stay separate (no cross-home aggregation; sprint exclusion).
- Rate-limit snapshots shown in `/status` come only from ChatGPT sign-ins, which get named
  accounts in S06.

<a id="pr"></a>PR: [#469](https://github.com/CorbanuCore/CorbanuTerminal/pull/469); final merged-tree results in its comments.
