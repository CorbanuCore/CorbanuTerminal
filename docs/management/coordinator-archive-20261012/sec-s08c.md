You are the PF-27-S08 lane worker (P1 security). Read sec-common.md (binding) and sec-s08.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, the sprint record
docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md and
qa/security-levels/sprints/PF-27-S08/README.md on origin/main ("Open decision for Travis: how the vault key reaches the broker").
Travis DECIDED on 2026-10-09: option (c) — the Windows broker reads Credential Manager itself, the same as the macOS and
Linux brokers read the OS keyring (PF-27-S05); PF-27-S09 ports S05 unchanged.
1. Docs PR: record the decision (date, by Travis) in the sprint record and the evidence README; amend acceptance criterion 5
   so it matches the decision (stored keys come from Credential Manager read by the broker's own token; drop "the broker's
   token reading Credential Manager is denied"), and add the accepted consequence to Known limits: a compromised broker can
   read, write or delete the user's other generic credentials in Credential Manager (measured), mitigated by the broker's
   other confinement. Note that sec-s08 merged #333 with checks still pending (they later passed on main: windows-security-
   probes and corbanu-terminal-ci green on d0544c1c91) so the record is honest.
2. Verify on current main that the S08 gate evidence still holds where cheap (the pf_27_s08 probes on CI's
   windows-security-probes job for the latest main commit); if the record's checklist now has everything ticked, set the
   sprint to `ready` for Travis's acceptance. Do NOT archive.
3. Update PF-27-S09's record (docs/sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md) so its
   dependency/key-path assumption reflects decision (c).
Run the repo's plan/sprint checkers. Docs-only PR; merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Report: PR number, what changed, and
the plain list of known limits Travis is accepting with S08.
