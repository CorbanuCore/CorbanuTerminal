# Round-three security handoffs

Dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`

Integration owner: Codex ingress/classifier lane.

This packet activates three dependency-complete, disjoint sprints under the
active P0 security plan. Each agent works only in its recorded CorbanuDrive
worktree and writes all build, cache, model, corpus and temporary data beneath
`/Volumes/CorbanuDrive/Corbanu/.codex-work/<lane>/`.

| Lane | Sprint | Owner | Handoff |
| --- | --- | --- | --- |
| classifier-corpus | PF-35-S01 | Raman | [Classifier corpus](classifier-corpus.md) |
| credential-reservations | PF-13-S06 | Pauli | [Credential reservations](credential-reservations.md) |
| durable-events | PF-41-S03 | Huygens | [Durable events](durable-events.md) |

All lanes must format before final tests, record actual test counts, run the
governance checkers and `git diff --check`, produce a real TMUX/Corbanu smoke,
and obtain a read-only Claude Opus 5 Plan review at Max effort in the Corbanu
Terminal harness. Prompt text and Enter are sent separately. Findings are
verified, fixed only within scope, rerun and rereviewed to no actionable
findings. Review transcripts live outside Git on CorbanuDrive; evidence records
their SHA-256 digests. No private blind corpus, credentials, model weights or
signing keys enter Git or reviewer context.

Agents commit their complete handback branches but do not merge or push. The
integration owner audits scopes, serializes shared registration and navigation,
merges in dependency-safe order, reruns combined-tree tests and TMUX proof,
updates `humanTest.html` and `securityProgress.html`, performs the final review,
then pushes `main`.
