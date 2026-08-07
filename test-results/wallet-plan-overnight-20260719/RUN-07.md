# Run 07 — credential disconnect, wallet recovery, and paid work

- Stories: US-4, US-7, US-9
- Qualified commit: `db062b429de21486bf842dfe3b4ac53a4a689812`
- Binary SHA-256: `2e2493f1d09b1bb1ff9f5eef8de09f163530bf056a1d6ad8dbe8bfdda9da7d7f`
- State: resumed compacted thread plus fresh credential actions in a copied home
- Verdict: **PASS**, with two rejected pre-final attempts retained below

## Exact-binary acceptance

1. The paid provider completed a tool turn on the resumed compacted thread:
   one `pwd`, one final answer, and one completion assessment. The turn settled
   from 49,353,646 to 49,398,438 tokens, left zero reserved, and the local log
   recorded `completion assessment accepted the final response` for turn
   `019f7a47-956f-7cf1-a5c6-3789bc7c7f8b`.
2. `/wallet` showed the authoritative Pro period, 0.010000 SOL, 0.00 USDC, and
   49,398,438 / 50,000,000 weekly tokens used.
3. `Disconnect plan` reported that the credential was removed while the paid
   period and wallet were unchanged. It immediately opened provider replacement
   instead of leaving a dead PfTerminal Plan route selected.
4. A one-action unlock exposed `Recover plan access`. Recovery issued a new
   credential without payment, printed no key, consumed the one-action grant,
   reselected `Ambient GLM 5.2 via PfTerminal Plan standard`, and restored the
   same Pro period and token totals.
5. The final gateway state was `/healthz = ok`, `/readyz = ready`, 49,398,438
   settled, and zero reserved. Balances remained 0.010000 SOL and 0.00 USDC.

## Substantive long-context evidence

The free-form architecture audit inspected wallet, daemon, login, TUI,
provider, request-lease, metering, and test boundaries. It crossed automatic
compaction and accumulated 1,802,416 settled tokens before the complete report
was delivered on the resumed compacted thread. The report contained the
requested architecture map, ten evidence-backed invariants, ranked risks,
human-TUI gaps, remediation order, and an explicit no-P0 conclusion.

The first long attempt exposed WTURN-005: exact commit `6f337693c` stopped after
a post-compaction progress announcement because a provider-renamed structured
classifier result was rejected. The next binary delivered the full report from
the preserved compacted state. The final exact binary then proved that the
provider's boolean completion schema is parsed successfully on a fresh tool
turn, with no warning or extra continuation.

## Rejected pre-final attempts

- `a374ab638`: copied-home vault could recover a credential but disconnect
  failed to decrypt the copied encrypted store (WUX-017).
- `6f337693c`: disconnect/recovery passed, but a post-compaction progress
  announcement was incorrectly accepted after the classifier returned a JSON
  shape without the requested `decision` property (WTURN-005).

No payment, SOL transfer, USDC transfer, source edit, build, or test was
performed by the paid model during this run.
