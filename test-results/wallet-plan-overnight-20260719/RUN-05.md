# Run 05 — simultaneous processes and global lock

- Stories: US-7, US-10
- Qualified commit: `a374ab638b3ddce2230e3cb1bcc3ad01d87804a7`
- Binary SHA-256: `ea59e11260f6e3652507a3d6b19a8ab612f364df5ec38353d6acce42cb095270`
- State: two fresh processes sharing one isolated PfTerminal home
- Verdict: PASS

## Required evidence

1. Start both processes without a writable-state ownership failure.
2. Read and compare wallet, balances, plan, receipt, and usage in both.
3. Unlock in one TUI and prove the second TUI does not inherit its capability.
4. Lock globally from the other TUI and prove the first loses signing access.
5. Confirm plan-provider reads and inference credentials remain available in
   both processes and notifications do not duplicate or freeze input.

## Attempt 1 — capability semantics passed, paid-turn control failed

Two processes on commit `fe155ab96b708c5c1f8dd7f3a7d62c709eb5d327`
and binary `a2bbea089985051f35189a95d81006dfb26fc95bb761f10b0c5b841834849625`
started against the same fresh copied home without a writable-state ownership
failure. Both displayed the same wallet, 0.010000 SOL, 0 USDC, active Pro
period, 47,254,684 used weekly/monthly tokens, reset boundary, and zero in
flight.

Process A unlocked for five minutes. A refresh in process B explicitly showed
`unlocked elsewhere; passcode required here`, proving that B did not inherit
A's signing capability. B then selected Lock wallet without a passcode and
received `Wallet locked in every PfTerminal process.` Refreshing A showed
`locked`, proving global revocation. Both `/usage` views converged on identical
server-authoritative spend, totals, and boundaries.

Both processes then completed a paid shell-backed provider request concurrently
while `/healthz` and `/readyz` remained healthy. That final step exposed
WTURN-004: malformed classifier output caused duplicate paid continuations.
Usage rose from 47,254,684 to 47,382,884 tokens across 13 settled requests,
with zero reserved tokens and no SOL/USDC movement. This attempt is rejected;
the capability evidence remains valid defect evidence but does not count toward
the final same-binary matrix.

## Attempt 2 — exact-binary replay

Two fresh 94×57 processes on the rebuilt binary started against
`/tmp/pft-wallet-qual05-replay-home` and rendered immediately. Both wallet
views independently reported the same 0.010000 SOL, 0 USDC, Pro receipt,
47,382,884 settled tokens, zero in flight, and identical period/reset
boundaries.

Process A unlocked for five minutes. Process B refreshed and displayed
`unlocked elsewhere; passcode required here`; it had no inherited signing
capability. B selected Lock wallet and received exactly one
`Wallet locked in every PfTerminal process.` notification. A's refreshed view
then displayed `locked` and no signing actions.

The processes submitted paid shell-backed turns simultaneously at 11:14:11Z.
Both returned the requested branch and modified-file facts, returned input
control without duplicate final answers or a completion warning, and remained
responsive. The gateway stayed healthy and ready. The ledger contained exactly
six settled requests—one tool round, one final round, and one classifier round
per process—for 62,342 tokens. Both classifier responses were malformed, and
the repaired host accepted each final answer without injecting a correction.
Usage ended at 47,445,226 settled and zero reserved. SOL and USDC balances did
not move.

Run 5 passes US-7 and US-10 on the exact binary: simultaneous state reads,
process-scoped signing, global revocation, concurrent provider access,
notification deduplication, bounded paid work, and zero input freeze.

## Attempt 3 — final-binary confirmation

The final rendering repair changed the executable, so the complete Run 5 path
was repeated on commit `a374ab638`. Two 94×57 processes shared
`/tmp/pft-wallet-qual05-final-home`; both showed 47,445,226 settled tokens and
identical wallet/plan state. A five-minute unlock in A appeared in B only as
`unlocked elsewhere; passcode required here`; B's global Lock produced one
notification and A refreshed to locked.

Both processes then ran simultaneous paid shell-backed turns. Each completed
once without a correction or warning. The ledger recorded exactly six settled
requests and 61,466 tokens, ending at 47,506,692 settled and zero reserved.
Gateway health/ready stayed green and SOL/USDC did not move. This is the
authoritative Run 5 PASS on the final binary.
