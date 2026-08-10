# Run 04 — interrupted provider stream and bounded recovery

- Stories: US-4, US-9
- Qualified commit: `fe155ab96b708c5c1f8dd7f3a7d62c709eb5d327`
- Binary SHA-256: `a2bbea089985051f35189a95d81006dfb26fc95bb761f10b0c5b841834849625`
- State: fresh Pro-plan process; same server-authoritative wallet account
- Started: 2026-07-19T10:27:43Z
- Starting usage: 47,157,070 settled, 0 reserved
- Ending usage: 47,254,684 settled, 0 reserved
- Verdict: PASS

## Required evidence

1. Start substantive free-form work and observe an active reservation.
2. Interrupt the client once during a live provider stream.
3. Confirm bounded visible recovery, no retry flood, and no provider switch.
4. Continue once and finish useful work.
5. Reconcile the ledger to exactly one disposition per request and zero orphaned
   reservations; confirm zero SOL/USDC movement.

## Evidence

An initial concise read-only turn completed before the driver could interrupt
it and was excluded from the fault result. The driver then submitted a
deliberately long no-tool response and injected Escape within 400ms of the
active reservation. Request `86d3166b-4755-4f6d-9e15-8e34d28ce51e` moved from
`reserved` to `released` at 10:28:18.647Z, 366ms after creation, with 31,352
reserved tokens, zero charged tokens, and no retry request.

The interrupted draft remained in the composer. The automation appended its
follow-up rather than clearing that draft, so the next user message combined
both instructions; this is expected editable-draft preservation, not task
duplication. The same TUI completed the combined follow-up once, returned
useful work, retained PfTerminal Plan, and ended with zero reserved usage. The
external ledger accounted for every request exactly once. No wallet unlock,
payment path, or chain transaction occurred, and SOL/USDC balances were
unchanged.

Run 4 passes US-4 and US-9: a real in-flight paid request was cancelled,
released without charge inside one second, did not retry, and the process
accepted and completed subsequent work.

## Final same-binary candidate — PASS

- Started: 2026-07-19T14:38:26Z
- Commit: `c0bbab5727fa8268d40be727959702165f449677`
- Binary SHA-256:
  `1b70e4ed3a51a52bdab833d2f4d72fc91d96b96f6ce93eea6c70e3f66dc63b94`
- Wallet-daemon SHA-256:
  `337bf8ff83b444183b5631247544b2411b7f10ea2283ee555010716d8a03f39c`
- Isolated home: `/tmp/pft-wallet-final-run4-home`
- Terminal: isolated tmux socket `pftwalletfinal4`, 94×57
- Starting usage: 8,997,449 settled, 0 reserved
- Ending usage: 13,261,883 settled, 0 reserved
- Run ledger: 82 settled requests, 1 released request, 0 reserved requests

The driver submitted a substantive read-only review, observed request
`ca9ef1cf-0ba8-40da-9cdb-5c3267c8774c` in the live `reserved` state with an
18,657-token reservation, and sent Escape once. PostgreSQL recorded the request
as `released` 109ms after creation with zero charged, input, or output tokens.
No second request appeared until the driver deliberately submitted the one
continuation.

The TUI remained responsive, preserved the interrupted user request visibly,
retained `Ambient GLM 5.2 standard` via PfTerminal Plan, and accepted exactly
one continuation. That continuation completed a four-minute, tool-using
network-to-consensus reliability review with exact production call paths and a
prioritized final report. The gateway remained healthy and ready. All later
requests settled, reserved usage returned to zero, and the worktree-state hash
remained empty and unchanged. No unlock, payment, provider switch, retry flood,
SOL movement, or USDC movement occurred; the known passphrase did not appear in
the copied home evidence.

Final-candidate verdict: **PASS** for Run 4 / US-4 and US-9.
