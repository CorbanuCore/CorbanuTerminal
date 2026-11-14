# Frozen original test design — #88 split recovery

**Scope:** Independent, code-blind, one-pass functional design. No repository, source, tests, results, or prior findings inspected. Synthetic credentials and isolated fixtures only; no live or personal keys. Coordinator records authority.

## Test observables

Observe request attempts, selected account/source, refresh-process activity, user-visible output, tool dispatches, terminal outcomes, persisted credential records, and resume behavior.

Use distinguishable synthetic accounts **A** and **B**, credentials, signed-thinking histories, and unknown credential metadata. Secret sentinels must never appear in user output, diagnostics, errors, or captured logs.

## P0 — Recovery classification and output safety

| ID | Starting state | Action | Observable outcome |
|---|---|---|---|
| R1 | A explicitly selected; valid Claude Plan credentials; accepted conversation history. | Complete an ordinary inference request. | One answer using A; no refresh, history repair, or source switching. |
| R2 | A valid; history contains recoverably rejected signed-thinking material. No substantive output. | Return the recoverable rejection as an HTTP error; allow the repaired attempt to succeed. | History recovery completes the request once. No credential refresh; A and its source remain selected. |
| R3 | Same as R2. | Return HTTP success, then an explicit recoverable signed-thinking rejection inside the event stream. | Same successful recovery as R2; HTTP success does not conceal the failure or prevent repair. |
| R4 | Same as R3. | Emit only stream metadata—start events, IDs, usage, keepalives, and empty deltas—before the rejection. | Metadata alone does not prevent otherwise eligible recovery. Exactly one substantive answer is delivered. |
| R5 | Recoverable signed-thinking rejection before output. | Make the recovery attempt reject again. | Recovery terminates within a bounded policy with an intelligible failure; no endless replay, credential refresh, or account switching. |
| R6 | Original attempt has delivered visible answer text. | Emit a signed-thinking rejection afterward, including a very short visible prefix variant. | No automatic replay of the request. Existing output is not duplicated or presented as a complete successful answer; failure is observable. |
| R7 | Original attempt has dispatched a tool action, but no answer text. | Emit a signed-thinking rejection. | No replay or duplicate tool action. The run reports the failure without silently restarting the workflow. |
| R8 | Valid credentials, but the selected account lacks the required Plan entitlement. | Return a true entitlement error, both as an HTTP failure and inside an HTTP-success stream. | Entitlement failure is surfaced; no pointless credential refresh, history-repair loop, or fallback to B. |
| R9 | A’s credentials are genuinely expired or rejected; no substantive output. | Trigger authentication failure; provide a successful selected-account refresh and successful continuation. | Existing credential recovery remains effective for A. B is neither refreshed nor used; no history repair is substituted for authentication recovery. |
| R10 | Genuine credential rejection for A. | Make credential recovery fail. | Clear terminal authentication failure; no use of B or another credential source. A’s recoverable pre-refresh login is preserved where file-backed preservation applies. |
| R11 | Valid credentials; unrelated provider error, malformed stream, or interrupted connection without an explicit recoverable rejection. | Fail before substantive output. | Failure is not misclassified as signed-thinking rejection, expired credentials, or missing entitlement. Neither specialized recovery is triggered solely by an early disconnect. |
| R12 | A valid; request requires both recovery classes sequentially. | Test signed-thinking rejection followed by genuine authentication rejection, and the reverse order. | Each failure receives only its appropriate recovery; attempts remain bounded, A/source stay selected, and no answer or tool action is duplicated. |

## P0 — File-backed credential preservation

For preservation cases, begin with a selected record that remains useful for recovery, even if its access credential is expired. Include nested unknown metadata and unrelated account records.

| ID | Starting state | Action | Observable outcome |
|---|---|---|---|
| F1 | A selected from a credential file; recoverable record intact. | Refresh process deletes, truncates, or corrupts A’s record, then exits unsuccessfully. | After failure, A retains its previously recoverable login and unknown metadata. B and unrelated records remain unchanged. |
| F2 | Same as F1. | Refresh damages A’s record, then hangs until timeout. | Timeout is reported; the damaged refresh does not leave A’s prior login destroyed. No account/source substitution occurs. |
| F3 | Same as F1. | Refresh exits successfully but leaves A missing, malformed, or incomplete. | Exit status alone does not establish recovery success. A remains recoverable; unusable credentials are not used or represented as a successful login. |
| F4 | A selected; unknown metadata present. | Refresh succeeds with a usable replacement credential record. | Subsequent use authenticates as A with the renewed record. Unknown metadata survives; unrelated account data remains intact. |
| F5 | A selected. | Refresh succeeds only in creating or modifying B’s credential record. | A’s refresh is not declared successful on B’s behalf. B is not silently selected or used; A’s recoverable login is retained. |
| F6 | A’s refresh is running. | Another actor writes a newer valid A record; the original refresh then fails. | Failure handling does not overwrite the newer valid login with an older snapshot or damaged data. |
| F7 | A’s refresh is running; A and B share a backing file. | Another actor updates B or unrelated metadata; A’s refresh then fails. | A remains recoverable without erasing concurrent changes to B or unrelated metadata. |
| F8 | Refresh has timed out or been cancelled. | A delayed child process or descendant later attempts to write or delete credentials. | Check immediately after termination and after the late activity: the unsuccessful refresh does not subsequently destroy the preserved login or change the selected identity. |
| F9 | A refresh has damaged the selected record. | User cancels the request while refresh is pending. | Cancellation does not silently convert recoverable login state into a lost login. No continued inference or fallback occurs. |

## P0 — Account isolation, source authority, and confidentiality

| ID | Starting state | Action | Observable outcome |
|---|---|---|---|
| A1 | A and B have distinguishable valid credentials; B would succeed if used. | Exercise history rejection, entitlement denial, and failed credential refresh while A is selected. | Every attempt remains associated with A and its explicit source. B’s availability never masks A’s failure. |
| A2 | Native Keychain is selected and authoritative; conflicting file credentials exist. | Make the selected Keychain item unavailable, rejected, or refresh-failing. | No fallback to file credentials and no file-based repair of Keychain state. Test only with an isolated synthetic Keychain context; otherwise leave this case unexecuted. |
| A3 | All fixtures contain recognizable synthetic secret sentinels. | Exercise success, errors, malformed records, subprocess stderr, timeout, cancellation, and resume. | No credential secrets appear in displayed output, diagnostics, logs, or error descriptions. |

## P1 — Stream boundaries, subprocess races, and resume

| ID | Starting state | Action | Observable outcome |
|---|---|---|---|
| E1 | Eligible in-stream signed-thinking rejection before output. | Split the error event across transport chunks; vary adjacent metadata and event boundaries. | Recovery classification is independent of chunk boundaries; one logical rejection produces one recovery decision. |
| E2 | Provider returns a normal answer that quotes error-like text, or a tool returns such text. | Include wording resembling signed-thinking, entitlement, or authentication errors. | Ordinary content does not trigger recovery merely because it contains matching words. |
| E3 | Stream contains partial tool-call identity/arguments, emitted thinking content, whitespace-only text, or non-text output. | Follow each variant with a recoverable rejection. | Classify each against the agreed substantive-output boundary. Never replay a dispatched tool action or duplicate visible output; unresolved boundary variants require the clarification below. |
| E4 | A selected; refresh begins. | User explicitly selects B while A’s refresh is pending. | No cross-account credential overwrite or silent use of B for A’s pending attempt. Subsequent requests honor explicit selection; pending-request behavior follows the clarified selection policy. |
| E5 | Two requests concurrently refresh A. | Let one refresh succeed and the other fail, in both completion orders. | The failed refresh cannot destroy the successful login. Neither request uses another account or dispatches duplicate actions. |
| E6 | Refresh replaces the credential file rather than modifying it in place. | Exercise successful replacement, failed replacement, and timeout during replacement. | Identity, recoverability, metadata, and unrelated-account guarantees remain the same. |
| E7 | History repair has succeeded once. | Continue the conversation, then close and resume it. | The conversation remains usable without duplicate turns/actions or loss of unrelated history. Repaired state does not create a recurring rejection cycle. |
| E8 | A failed or timed-out refresh damaged the file but preservation completed. | Restart Terminal and explicitly reuse A. | A’s previously recoverable login and unknown metadata remain available across restart; preservation is not merely in-memory. |
| E9 | Stream failed after visible text or a dispatched tool action. | Resume the conversation and make an explicit new request. | Prior partial output/actions are not automatically replayed. The failed turn is not silently treated as a completed successful answer. |

## Ambiguities requiring an explicit oracle

1. **Substantive output:** Does this include visible thinking, whitespace, tool-call headers, partial arguments, or non-text deltas? Is the boundary receipt, display, persistence, or tool dispatch?
2. **Error classification:** Which provider error codes or structured signals distinguish recoverable signed-thinking rejection, entitlement denial, and genuine credential rejection? What wins when signals conflict?
3. **Recovery bounds:** What retry limit applies per failure class and across alternating history/authentication failures?
4. **Credential validity:** What constitutes a successful refreshed record beyond process exit status? What does “preserve unknown metadata” require when the refresh intentionally updates an existing unknown field?
5. **Concurrent intent:** If the user deliberately deletes/logs out A during refresh, should that action override preservation? How is it distinguished from refresh-induced deletion?
6. **Selection during recovery:** Does an in-flight request retain A, cancel, or require confirmation after the user explicitly selects B?
7. **Persistence and interruption:** Must history repair survive resume? Does preservation cover abrupt Terminal termination or machine failure, beyond refresh failure, cancellation, and timeout?
8. **Post-output authentication failure:** May credentials be refreshed for future requests without replaying the current request, or must recovery stop entirely?

**Frozen:** Original design only; no execution, implementation recommendations, or child-agent review.
