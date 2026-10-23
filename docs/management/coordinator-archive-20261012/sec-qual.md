You are the convergence-lane worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding: keychain rule, disk hygiene, gate). Then read the sprint
docs/sprints/current/p0-security-levels/pf-13-s07-integrated-credential-boundary-qualification.md, the plan
docs/plans/active/p0-security-levels.md, and the result files sec-broker2..6b, sec-url*, sec-mig, sec-taint* in this
folder for what each flag does and its known limits.

Sprint: PF-13-S07 "Integrated credential boundary qualification". Its dependencies are all merged behind flags on
main (PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02; PF-13-S05/S06 archived). Follow the merged-behind-flag precedent.

Deliver final-tree canary and adversarial evidence for the composed credential boundary with ALL the protection
flags on together (isolated_credential_broker, secretless_agent_launch, secret_output_gate, url_destination_policy,
protected_mode_preflight, source_envelopes, security_levels=Aggressive and Moderate), on main's current tip:
- run the existing credential canary harness (scripts/security-credential-canary, as CI does) against a locally built
  candidate with all flags on, on macOS here and Linux on the RTX box;
- an adversarial pass with GLM 5.2 in tmux: an agent that tries every known route to a canary credential (env,
  files, vault, broker socket, nested launch, MCP/hook, echo-back, encodings, redirects, DNS rebinding, Claude pane);
- record each route's result in a matrix; any route that leaks is a bug: file an issue with the security label, and fix
  it if it's small (own PR), otherwise hand it to the owning lane in the issue.
Also check the flags compose: no crash/deadlock, normal coding workflows still work (a short real task end to end).
No new credential behaviour in this sprint except bug fixes. Never touch the real keychain or real credentials: fake
canaries and disposable homes only. Evidence under qa/security-levels/sprints/PF-13-S07/, one independent Opus 5.5
High review of the evidence, demo videos of the key routes. PR to main; merge when green. Report as in sec-common.md.
