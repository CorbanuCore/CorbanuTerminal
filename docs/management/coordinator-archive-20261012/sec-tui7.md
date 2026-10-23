You are the TUI lane worker (round 7). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding: keychain rule, disk hygiene, gate). PF-23-S03 is merged (#246-#249), so PF-24-S02 is unblocked.

Implement PF-24-S02 "Security confirm, cancel, and downgrade"
(docs/sprints/current/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md), building on the flagged
/security picker (PF-24-S03, Aggressive = the `corbanu-aggressive` permission profile, decision A), PF-23-S03's
durable/confirmed transitions and revocation fan-out, and PF-29's preflight. Include the picker follow-ups recorded
earlier as PF-24-S02 items (tamper check on the level file, a "restart now" path, the second-Permissive-launch issue).
Then PF-25-S01 (temporary grant TUI) if time allows.
Per-sprint gate per sec-common.md. GLM 5.2 is currently out of Z.AI credit: use a mock/fake model for TUI flows and
record that in the gate record (a GLM run can be added later). Linux clippy on the RTX box. PR to main; merge when
green. Report as in sec-common.md.
