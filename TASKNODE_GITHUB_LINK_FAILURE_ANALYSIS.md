# Task Node GitHub Link Failure Analysis

**Date:** 2026-08-06  
**Scope:** PFTerminal 0.1.27 Task Node terminal authentication  
**Severity:** High — an attempted relink can destroy a working local terminal session before replacement authentication succeeds

## Executive summary

The Task Node GitHub link is fundamentally unsafe because PFTerminal stores two different states in one mutable vault record:

1. the **active terminal session**, containing the bearer token used by Task Node requests; and
2. an **incomplete GitHub link attempt**, containing a request ID, poll token, and verification URL but no terminal token.

Starting `/tasknode link` immediately writes the incomplete link attempt over the active `tasknode/session` credential. The old terminal token is discarded before GitHub authorization succeeds. From that moment, fresh TUI requests and the JSON helper can only see a pending session and refuse normal Task Node operations.

This is not merely a confusing message or an expired-token edge case. It is a broken authentication-state transition: **beginning a replacement destroys the currently usable credential instead of atomically swapping credentials after the replacement is proven valid.**

## Observed incident

The same user/account produced contradictory Task Node results during one terminal workflow:

- A Task Node status view reported a linked GitHub account, wallet, task counts, and `terminal direct-write enabled`.
- The JSON helper first received `terminal_login_required` from the backend for the stored terminal token.
- Starting a new `/tasknode link` then changed the helper result to `Task Node link is pending`.
- The pending link record prevented the helper from reading the active task and submitting evidence, even though an already-rendered Task Node view still showed the linked account and valid task state.

No token or authorization URL is included in this report.

## Concrete implementation defect

### 1. Active and pending authentication share one vault label

Both states are serialized as `TaskNodeLocalSession` and stored under:

```text
tasknode/session
```

Relevant code:

- `codex-rs/tui/src/chatwidget/tasknode_menu.rs`
  - `open_tasknode_link`
  - `TaskNodeLocalSession::save`
  - `ensure_tasknode_session`
- `codex-rs/cli/src/tasknode_cmd.rs`
  - `load_tasknode_session`

The record can contain either:

```text
terminal_token = Some(...)
```

or:

```text
terminal_token = None
pending_request_id = Some(...)
pending_poll_token = Some(...)
pending_verification_url = Some(...)
```

Those are different lifecycle objects, but the implementation treats them as two shapes of the same credential.

### 2. Link startup destructively overwrites the active credential

`open_tasknode_link` calls `start_github_link`, constructs a session with `terminal_token: None`, and immediately calls `session.save(&codex_home)`.

`TaskNodeLocalSession::save` uses `vault.update(...)` when `tasknode/session` already exists. It does not:

- retain the existing terminal token;
- write pending state to a separate label;
- compare a session generation or revision;
- wait for GitHub authorization;
- validate the replacement token;
- roll back when authorization is abandoned or fails.

Therefore the transition is:

```text
working active session
        |
        | /tasknode link begins
        v
pending session with no usable token
```

The safe transition should be:

```text
working active session + separate pending attempt
        |
        | GitHub authorization completes and replacement token is validated
        v
atomically replace active session; then delete pending attempt
```

### 3. The helper correctly exposes the corrupted local state, but cannot recover it

`load_tasknode_session` in `tasknode_cmd.rs` requires `terminal_token`. If the shared vault record contains only pending fields, it exits with `Task Node link is pending`.

That behavior is locally consistent, but the helper has no access to the previous working token because link startup already erased it. It cannot continue normal work, choose the still-valid active session, or roll back the pending attempt.

### 4. Concurrent PFTerminal processes make the failure nondeterministic

Multiple PFTerminal processes can share the same `CODEX_HOME` and vault. Each process can start or poll a link attempt and update the same label. There is no visible compare-and-swap revision or session-generation check around `TaskNodeLocalSession::save`.

Consequences include:

- one process replacing another process's active session with pending state;
- a late poll completion overwriting a newer session;
- one surface showing previously fetched account data while another reads the newly overwritten vault record;
- failures that appear to contradict the user's visible linked state.

A vault file lock protects storage integrity, but it does not provide authentication lifecycle correctness. Last-writer-wins serialization can be race-free at the file level and still be wrong at the product level.

## Diagnostic and UX failures

### Ambiguous backend error

The backend returns `terminal_login_required` with guidance to link Task Node when a terminal bearer token is absent, revoked, expired, or unknown. That message does not distinguish:

- GitHub is not linked;
- GitHub is linked but this terminal session is invalid;
- a relink is pending;
- the local vault points at the wrong origin or home;
- another process replaced the local session.

The user can therefore be truthfully linked at the account level while being told to link again at the terminal-session level.

### Status is not a diagnostic surface

`/tasknode status` itself requires a valid terminal token. When authentication state is broken, the command cannot report the local facts needed to explain the breakage.

A safe status view should show, without exposing secrets:

- active session present: yes/no;
- active session origin;
- active session expiry;
- pending link present: yes/no;
- pending link age;
- resolved `CODEX_HOME` or a non-sensitive fingerprint;
- runtime/binary identity;
- last successful terminal authentication time;
- server result category such as valid, expired, revoked, or unknown.

### Relinking is presented as harmless

The UI says it is starting a GitHub link and later says the link is ready. It does not warn that starting the flow replaces the stored terminal credential immediately. The operation behaves like destructive logout-before-login without confirmation or rollback.

## Why this is a general boundary failure

The user-visible example is evidence of a broader authentication and persistence defect, not a sentence or command that needs special handling.

The failed boundary is the **active-session replacement protocol**:

- pending intent is persisted as if it were active authority;
- an unverified credential replaces a verified credential;
- concurrent writers have no generation discipline;
- account linkage and terminal-session validity are conflated;
- status does not expose enough non-secret state to reconcile surfaces.

Any OAuth/provider relink workflow implemented with this same one-record pattern can suffer the same class of failure.

## Required repair

### Separate active and pending state

Use distinct vault labels or an explicitly versioned envelope, for example:

```text
tasknode/session/active
tasknode/session/pending
```

Starting a link must only create or update `pending`. It must never delete or modify `active`.

### Atomically promote only a verified replacement

After GitHub authorization completes:

1. poll the pending request;
2. obtain the replacement terminal token;
3. validate it against the status endpoint;
4. acquire the vault mutation lock;
5. compare the pending attempt ID/generation;
6. atomically replace `active`;
7. delete only the matching `pending` attempt.

If any step fails, preserve `active` and report that the replacement failed.

### Define concurrency semantics

Every pending attempt and active session should carry a non-secret generation ID. A process may promote only the pending generation it started or explicitly adopted. Late completion of an older attempt must not overwrite a newer active session.

### Improve error taxonomy

Return and render distinct errors, such as:

- `github_provider_not_linked`;
- `terminal_session_missing`;
- `terminal_session_expired`;
- `terminal_session_revoked`;
- `terminal_session_unknown`;
- `terminal_relink_pending`;
- `terminal_session_origin_mismatch`.

Recovery guidance should match the actual state. “Link GitHub” is incorrect when GitHub is already linked and only the terminal credential is invalid.

### Make status authoritative across surfaces

The TUI and JSON helper must use the same session resolver and report the same active/pending generations. Previously fetched task/account data should be labeled cached or stale rather than presented as proof that the currently persisted credential is usable.

## Regression tests

The repair is incomplete without tests for the behavior class:

1. **Active session survives link start**  
   Given a valid active token, starting a new GitHub link leaves normal status/task/helper requests functional.

2. **Abandoned link is non-destructive**  
   Closing PFTerminal or never completing GitHub authorization preserves the active session across restart.

3. **Failed replacement rolls back naturally**  
   A denied, expired, or failed pending attempt is removable without changing the active session.

4. **Successful replacement is atomic**  
   The active generation changes only after the replacement token passes validation.

5. **Concurrent link attempts cannot clobber**  
   Two processes sharing one `CODEX_HOME` cannot promote an older pending attempt over a newer active generation.

6. **Helper/TUI parity**  
   The TUI and `pfterminal tasknode status --json` resolve the same home, origin, active generation, and pending state.

7. **Revoked token has accurate guidance**  
   A linked GitHub account with a revoked terminal token reports terminal-session invalidity, not GitHub-not-linked.

8. **Status remains diagnostically useful**  
   Local non-secret active/pending metadata is visible even when the backend rejects authentication.

## Acceptance criteria

The GitHub Task Node link can be considered repaired only when:

- starting or abandoning a relink cannot interrupt a valid active session;
- promotion of a replacement token is validated and atomic;
- concurrent PFTerminal processes cannot overwrite newer authentication state;
- TUI and JSON helper agree on active and pending state;
- error messages distinguish account linkage from terminal-session validity;
- users can diagnose the state without revealing tokens or deleting credentials;
- the regression matrix above passes.

## Conclusion

The core problem is not that the user failed to link GitHub. The core problem is that PFTerminal models **a pending attempt as the active session record** and overwrites proven authority with unproven intent. Until active and pending state are separated and promotion becomes atomic, `/tasknode link` remains capable of breaking a working Task Node installation merely by being started.
