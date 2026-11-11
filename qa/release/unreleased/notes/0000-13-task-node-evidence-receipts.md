## Task Node evidence receipts

- **An evidence receipt that confirms nothing now fails.** If Task Node answers
  `corbanu tasknode task evidence` or `verification respond` with success but
  no event id, transaction hash or CID, the command exits 1 with
  `task_evidence_receipt_unconfirmed`, the original receipt and
  `evidenceGateOpen` (whether the task still accepts that evidence). It used to
  exit 0 (#1280, reported by @0xzoz).
- **Confirmed receipts name what was recorded.** `pfterminalLifecycle` now has
  `submissionRef` (event id, transaction hash and/or CID), reports an empty
  `currentVerificationRequest` (`{}`) as `null`, and only says the initial
  evidence gate is closed when the refreshed task says so. If a task stays
  submitted with no verification request, report the task id and
  `submissionRef` to Task Node operators rather than cancelling it.
