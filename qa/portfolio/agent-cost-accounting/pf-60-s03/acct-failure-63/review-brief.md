# Independent code review: accounting contention fix (acct-failure-63)

You are a reviewer. Do NOT edit any files, commit, or push. Read-only review.

Repo worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-failure-63 (branch fix/acct-failure-63-20261003),
base 21162588cd, 4 commits. The full diff is at /Volumes/CorbanuDrive/Corbanu/.codex-work/acct63/review.diff
(read it and the surrounding code in the worktree with rg/sed).

Problem: "Native Anthropic accounting failed; request stopped without a repair send" kills sessions when
several Corbanu processes share one state DB. Root cause found: (1) the hourly full retention sweep ran under
BEGIN IMMEDIATE for 16-17 s on a 7k-attempt ledger, (2) every write re-verified every attempt of the day
(1.4 s/write), (3) every new pooled connection ran PRAGMA auto_vacuum which takes the write lock, and
(4) SQLITE_BUSY after the 5 s busy timeout was mapped straight to FAILURE and poisoned the sampling.

Review for:
1. Correctness of the ledger semantics: is `day_quotes_on_connection` exactly equal to
   `latest_quote_on_connection` for an intact ledger (rules selection, unbound attempts, missing snapshot)?
   Is the refactored `latest_quote_on_connection` still verifying every stored version exactly as before?
2. `validate_hour_on_connection` + `maintain_for_write_on_connection(.., validated_at_ms)`: can the fast path
   skip work the full sweep would have applied (expiries, compact days, gc)? Any clock/ordering hole
   (backward checkpoint, hour boundaries, AsOf::At in tests)? Is it safe that validation runs on a read
   snapshot separate from the write transaction?
3. Retry (`store_call`, `is_contention`): idempotency of open/admit/observe on retry; can a retried call
   double-record, or misclassify a non-transient error; is holding the process-wide WRITES semaphore across
   retry sleeps acceptable; does cancellation still fail closed?
4. Logging: any secret, prompt content or endpoint leaked into logs? Any FAILURE path still without a cause?
5. sqlite.rs auto_vacuum change: any case where a new DB misses incremental auto-vacuum, or an existing DB
   behaves differently?
6. Tests: do they prove the claims; any flakiness risk (the core test holds a lock ~5.6 s twice)?

Output: a verdict line (APPROVE / APPROVE WITH NITS / REQUEST CHANGES), then numbered findings with
file:line, severity (blocker/major/minor/nit), and a concrete suggested fix. Be concise.
