# PF-80-S01 — native server identity observation

Manager allocation, executable after review/checks and recorded launch HEAD.
Product: **Internal delivery control — TO BUILD**, “Use sequential sprints per
initiative.” Travis approved this Mac/existing publisher for now and server-backed
null-expiry validity design. No live credential, GET, enrollment or send authority.
Baseline99001e9b79676f78b6bd941125c2a8fcfca6d90e; existing Task Node worker
coordinates in plan/S01. Parent fast-forwards clean idle worker to reviewed
amendment and records launch HEAD. No second same-sprint implementation worker.

## Literal scope

`codex-rs/tasknode-session/src/session_validity.rs`, sibling
`session_validity_tests.rs`, `client.rs` (async byte-reader extraction only),
`lib.rs` (one unconditional private module declaration), and
`qa/initiative-control/pf-80-s01/native-validity/receipt.md` plus `SHA256SUMS`.
Target404 non-test/724 total; hard500 non-test/800 additions+deletions. Parent
delegates that registration only. No manifests/dependencies/auth store/old receipts/
shared plans/CLI/TUI changes. Return coherent oversized diff for re-slicing.

## Actual production path, no enabled entry

Extract crate-private async `Client::request_bytes` returning status/original
bytes; existing public request delegates and retains decode semantics. Preserve
origin/bearer/timeout/no-redirect/16MiB bounds. Blocking and stream paths unchanged.
Private production-compiled `session_validity` calls this for exactly GET
`/api/terminal/tasknode/status`, no query/body/retry. No public facade or caller.
Share orchestration with a private FnOnce exchange seam for offline tests, not a
parallel algorithm/trait. No load/resolve/Vault/queue/enrollment call.

Expected binding is immutable invocation/generation, SessionScope, opaque account,
canonical HTTPS origin, Client identity and expiry metadata. Explicit current
callback is sampled before/after request and at single consumption; supplies
current ActiveSession, time and named Active/Cancelled/SessionChanged control.
Reject empty identity/token/named profile, changed origin/account/profile/token/
invocation/generation/expiry, unavailable state, known expiry<=now, invalid present
expiry and backward time. None must pass the server check, never a fabricated TTL.
Pending relink/unlink is held by the caller. No credential-bearing Debug/Serialize.

Strictly deserialize raw identity bytes, accepting only200, explicit ok:true,
exact nonempty accountId and typed github linked:true/terminalBridgeEligible:true/
string username. Reject duplicate/missing/type-invalid identity fields including
nested duplicates. Deliberately ignore unrelated status fields; no whole-schema,
entitlement, task, wallet or revocation-epoch inference. Source is pinned server
40d2df72710a644f33a2b30831061e8265716db0, native status route and terminal-auth
lookup; inspect locally, no fetch. Server accepts unrevoked null-expiry sessions.

Privately construct non-Clone/non-Copy/nonserializable CheckedIdentity only after
post-response fence; consuming it rechecks current binding. It proves acceptance
at check time for this observation, not a lease/send authorization or future GET.
Use fixed typed held diagnostics; no server/error bodies, token or fingerprint
output. Pre-cancel makes no request, in-flight cancel/rotation invalidates proof;
dropping future produces no proof but cannot undo an already transmitted GET.
Caller-supplied generation is not cross-process validity. Later runtime facade
must qualify durable cancellation/current-authority checks and exact event fence.

## Proof and continuation

Offline production-pipeline tests: null and future expiry require one exchange;
malformed/missing/duplicate identity, wrong account/linkage, scope default versus
named default; all fence mutations before/during/after; expiry crossing, invalid
expiry/backward time; cancel at every phase/drop pending future; fixed outcomes for
non200/redirect/401/409/rate-limit/server/transport errors, no canary leakage.
Verify exact synthetic Client-built GET/origin/bearer/no body without sockets.
Known-expiry/ExpiryUnknown delivery engine behavior remains untouched. Use ready/
pending futures and existing dependencies; no new executor or live test transport.

Guarded scoped fix/format before final affected tests, cached Rust1.95.0 offline
auto-install OFF, assigned target. Run nonzero session_validity, delivery_send,
adapter/reconcile/goal/recovery selectors and full `just test -p
codex-tasknode-session` (existing loopback regression tests allowed), normal
offline locked library check and root governance/whitespace checks. No direct
cargo test; preserve full logs/LEAK names and actual hashes/size. Parent owns one
independent Astra High review and combined-tree testing; worker does not commit,
push, review or launch children. No UI/live/human/release qualification claimed.

Next manager work: contextual [dashboard/Slack decisions](../../plans/decision-escalation.md)
within this same sprint, then scoped operator recovery facade/retained-record,
cross-process/restart and old-event observation contract. Local inspect/cancel must
precede credentials and remain accessible while posting OFF. Original uncertainty,
bytes/ID/digest survive restart;404 is not retry permission. No live TUI probe
(paused recording still flushes). PF79/PF81 stay dependent drafts.
