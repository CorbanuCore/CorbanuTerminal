You are the broker-lane worker for PF-27-S06 "Windows broker and secretless launch"
(docs/sprints/current/p1-security-hardening/pf-27-s06-windows-broker-and-launch.md). Read sec-common.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first (binding: keychain rule, disk hygiene incl. never
deleting worktrees referenced by the live owner settings, per-sprint gate) and the PF-27-S02/S04/S05 archived records.
Travis approved starting this sprint now. No Windows host is available yet (Travis can provide one later); use the
repo's Windows CI runners (windows-2022) for real probes: add a focused Windows CI job or extend an existing one so
pf_27_s06 tests and containment probes actually run on Windows on PRs touching this code.
Deliver: named-pipe broker transport (DACL to the controller, client process id check, no inheritable handles);
containment probes (agent cannot open Core/broker with PROCESS_VM_READ, read their environment, or inherit handles);
turn the launch contract's Windows refusal into a measured pass, or keep it with a stated reason. Allocate the sprint
record (worktree/branch/base) properly; keep slices reviewable (<~800 lines each, several PRs is fine); Opus 5.5 High
review per slice; Linux clippy on the RTX box; merge each PR when green
(`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts, never force-push).
The tmux run and videos part of the gate needs a real Windows machine: do everything else, and list precisely what you
need on that machine (OS version, tools, access) so the coordinator can ask Travis. Report as in sec-common.md.
