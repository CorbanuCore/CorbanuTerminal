You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI).

Repository worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/pf84-fixes-474-482-20261011, checked out on tmp/pf84-fixes-combined (origin/main plus the four branches below merged).
Each fix is its own branch and PR; review each with `git -C /Volumes/CorbanuDrive/Corbanu/worktrees/pf84-fixes-474-482-20261011 diff origin/main...<branch>`:
- fix/pf84-throttle-476-482 (PR #485): #476 the provider-request throttle never recorded a 429 cooldown for any
  request without a lease (leases are taken only by spawned sub-agents sending large uncached requests), so exec/TUI
  429s never cooled down. Unleased requests now record their result (state: record_provider_request_throttle_outcome,
  UnleasedRow::apply; core: ProviderRequestThrottle / ProviderRequestLeaseGuard in core/src/session/turn.rs).
  #482: the default account's throttle key was an unsalted sha256 prefix of the API key; now salted per home with a
  salt in the state DB (migration state/migrations/0053_provider_request_key_salt.sql), legacy rows adopted on use.
- fix/pf84-vault-salt-474 (PR #486): #474 reading a named account's fingerprint created the per-home salt and
  rewrote the vault; now the read never writes and the salt is created by account add/rename in the same batch.
- fix/pf84-rejected-account-480-481 (PR #489): #480 corbanu exec names a rejected (401) named account and its
  recovery; #481 a TurnComplete carrying a terminal error maps to AgentStatus::Errored (spawn wait said Completed).
- fix/pf84-cost-refused-475 (PR #490): #475 a 401/403 before any response is recorded as a refusal observation
  (ledger format 5, state/accounting_migrations/0005_refused_attempts.sql) that replays to zero usage; /cost shows it
  as refused with no charge.

Please check, with file:line evidence, and rank findings (blocking / should-fix / nit):
1. #476: can an unleased result ever shorten or clear another request's cooldown, touch a lease, or mis-escalate?
   Memory/DB fallback consistency (cooldown_in_state_db), busy/read-only DB, interrupt (or_cancel) paths. Is skipping
   statusless failures right? Any request path (compaction, websocket, retries) where a 429 still is not recorded, or
   where a human session is now wrongly blocked? Extra DB write per request: acceptable?
2. #482: salt handling (OnceCell, missing row, read-only DB fallback key), migration DELETE correctness (time units,
   LIKE pattern vs new 'env-salted:' prefix), legacy adoption (UPDATE OR IGNORE then DELETE, once per process), and
   whether any value derivable from the key without the home's salt remains stored. Older builds sharing the DB.
3. #474: any remaining path where a read writes the vault; salt creation atomic with the account write; rename and
   write_provider_account (token refresh in login/src/auth/provider_key_vault.rs) behaviour; fingerprint stability.
4. #480/#481: exec message correctness for resume (provider_account from ThreadResumeResponse), default/Claude Plan/
   auth-command providers, JSON mode untouched; any consumer of AgentStatus::Completed that a terminal-error turn now
   changes unexpectedly (V1/V2 wait, close_agent, parent notifications, app-server CollabAgentStatus mapping).
5. #475: ledger-format discipline (Observation::ledger_format, append_on_connection, late import path in
   state/src/runtime/accounting_late_import.rs, retention/compaction), replay validation, canonical bytes of existing
   records, pricing of a refused attempt under plan/local/undeclared bases, TUI wording, and whether recording a 401/403
   as zero cost is ever wrong (e.g. a provider that bills a 403).
6. Tests: do they prove the fixes and would they fail without them? Missing regression tests.
Finish with an overall verdict per PR: approve / approve with fixes / request changes.
