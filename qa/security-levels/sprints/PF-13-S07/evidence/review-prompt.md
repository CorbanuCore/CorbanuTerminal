You are an independent security reviewer (Opus 5.5 High). Review the PF-13-S07
integrated credential boundary qualification evidence in
qa/security-levels/sprints/PF-13-S07/evidence/.

This is a read-only review. Do not modify any files. Only examine the evidence
folder and the route-matrix.sh harness.

Check:
1. The credential canary reports (macOS + Linux) — did the harness cover all
   required surfaces? Are there any canary leaks in the report? Is the candidate
   SHA-256 consistent across the macOS report and the route matrix?
2. The route matrix (route-matrix.json + results.jsonl) — are the routes
   comprehensive? Is the "BLOCKED" result credible for each route? The files
   route shows LEAK(raw) — is this finding correctly attributed to workspace-write
   full-disk-read behaviour (issue #239), not a protection-flag regression?
3. The composition test — does it prove flags compose without crash/deadlock?
4. The tmux adversarial outputs — are the blocked results real?
5. The README — does it honestly represent the evidence? Any discrepancy between
   the README claims and the actual evidence files?

Key context:
- The protection flags are: isolated_credential_broker, secretless_agent_launch,
  secret_output_gate, url_destination_policy, protected_mode_preflight,
  source_envelopes, security_levels (feature enabled, level not persisted).
- security_levels feature is on but the level defaults to Permissive (not
  persisted). The flags arm protections regardless of the persisted level.
- Workspace-write sandbox allows full disk read (has_full_disk_read_access()
  returns true for all sandbox modes). Read sandboxing requires Aggressive or
  Moderate level. The files-route LEAK is this behaviour, documented in issue
  #239.
- Claude panes are a known gap (PF-27-S02 documents this).

Give a verdict: APPROVE, APPROVE WITH NITS, or CHANGES REQUIRED.
List any P0/P1 findings. Be concise.
