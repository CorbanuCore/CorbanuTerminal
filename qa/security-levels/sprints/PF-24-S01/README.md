# PF-24-S01 — read-only security profiles

Product initiative under **P0 `/security` levels**: “Existing approval, sandbox,
vault, wallet, tool, network, and agent policies are unchanged.” Allocation:
`4f263ca73`, branch `feat/security-round5-ui`; source base
`07791288b6feeccfaee5a57c12452359cc666957`.

## Implemented slice

`/security` opens keyboard-navigable Permissive, Moderate and Aggressive profile
descriptions. It displays configuration intent separately from unavailable live
effective-protection evidence. Required protected controls are explicitly
unqualified. `/status` shows the requested level and an unverified-protection
warning, wrapped at narrow widths.

Navigation uses the configured list bindings. Enter explains that nothing was
changed; Escape or the configured cancel key closes the view. The view has no
event sender, policy controller, config writer, grant, or activation API.
Reopening reconstructs configuration intent, not stale runtime health.

The existing `SecurityViewState` / `SecurityInspectorEvent` contract remains an
unconnected future seam. This sprint does **not** manufacture a trusted live
observation from editable preferences. PF-24-S02/PF-41 retain authenticated
run-generation, effective-readiness, transition and detailed-inspector work.

## Qualification ledger

All compilation is on the authorized RTX host, in
`/home/travis/worktrees/security-round5-ui`, serialized using the coordinator's
build lock. Remote evidence root:
`/home/travis/security-round5/evidence/ui/`.

| Evidence | State |
| --- | --- |
| `just fix -p codex-tui` | Pass on intermediate tree; final rerun pending |
| `just fmt` | Rust edits applied; complete formatter initially unavailable because `uv` was missing; final complete pass pending |
| Focused TUI nextest | Initial 235 cases: 212 pass; intentional snapshots and the side-conversation allowance were corrected. Snapshot-generation run: 235/235; final normal-mode rerun pending |
| Actual-key TMUX | 2/2 pass on `6ed694c86`, run `9baff18c-db43-44fe-8b12-8dfb455d81a0`; final rerun/hash pending |
| Astra High review | Not started; coordinator allocates numbered invocation |
| Fable 5.1 High via Corbanu/TMUX | Not started; coordinator allocates numbered invocation |
| Human acceptance | Pending; no human sign-off claimed |
| PF-26 composed/live-repository release qualification | Pending; no protected mode or release qualification claimed |

The typed harness uses isolated temporary homes and synthetic authentication,
`RUST_LOG=trace`, separate text and Enter sends, and no inference requests.
Success captures go to the explicit `CORBANU_SECURITY_UI_EVIDENCE` directory;
failure bundles use `CORBANU_TMUX_ARTIFACT_DIR`. Raw authentication is not part
of committed evidence.

## Human/interactive script

1. Start the exact candidate with existing Permissive configuration.
2. Type `/security`, then separately press Enter. Confirm configuration-only
   requested state, unverified effective protection and blocked protected modes.
3. Move through all three profiles. Read the differences; Enter must say
   “Nothing changed.” No confirmation or activation occurs.
4. Escape, open `/status`, and verify the requested level did not change.
5. Repeat at 40 columns and with non-default navigation bindings. Reopen the
   view and restart: exploration must never rewrite configuration.
6. A synthetic unknown configured level must fail visibly at startup, without
   falling back to Permissive. Do not change a real profile to perform this test.

Moderate/Aggressive fixture configurations prove honest presentation only, not
qualified enforcement. The complete protected-mode, live-repository and
cross-platform acceptance gates remain owned by their later sprints.
