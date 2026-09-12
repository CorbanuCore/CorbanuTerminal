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
| `just fix -p codex-tui` | Pass on final source tree; `fix-qualified.log` (existing warnings only) |
| `just fmt` | Complete pass with coordinator-supplied uv 0.11.3; exit 0 and no file changes (`fmt-complete.log`), so the final tested source is unchanged |
| Focused TUI nextest | Final normal-mode 235/235 pass; run `36c926b9-7b0b-4c54-aa47-da1ce10471be`, `final-tests.log` |
| Actual-key TMUX | Final 2/2 pass (three profile/configuration widths plus startup rejection); run `c5c6f41d-d93e-423a-8c7a-8495f677b760`, `final-tests.log` |
| Astra High review | Attempt 1/5 was rejected before evaluation by old CLI 0.145.0. Attempt 2/5 with app-bundled 0.153.1 passed: no findings, helper exit 0; `review-2-astra.json` and `.txt` preserve the verdict |
| Fable 5.1 High via Corbanu/TMUX | Attempt 3/5 passed: no findings, helper exit 0; `review-3-fable.json` and `.txt` preserve the verdict. No further UI review planned |
| Human acceptance | Pending; no human sign-off claimed |
| PF-26 composed/live-repository release qualification | Pending; no protected mode or release qualification claimed |

The typed harness uses isolated temporary homes and synthetic authentication,
`RUST_LOG=trace`, separate text and Enter sends, and no inference requests.
Success captures go to the explicit `CORBANU_SECURITY_UI_EVIDENCE` directory;
failure bundles use `CORBANU_TMUX_ARTIFACT_DIR`. Raw authentication is not part
of committed evidence.

Final tested source: `0ecb19969`. The remote run started at `a56027455` plus
the exact Rust formatter patch subsequently committed as `0ecb19969`; the
remote mirror was fast-forwarded to that commit and is clean. Candidate:
`/home/travis/security-round5/evidence/ui/candidate/codex`, `corbanu 0.1.38`,
SHA-256 `7eebcd43caf33dfeb76a3c3460889b3637799ed64263fb4d17e36b3a05f9fbd8`.
It was copied while holding the build lock, before any other lane could replace
the shared cache binary. Final rendered captures are in [`tmux/`](tmux/).

Commands after Rust fix/formatting (no `INSTA_UPDATE` on these final runs):

```sh
just test -p codex-tui --lib -E 'test(security_view) | test(status::tests) | test(slash_command)' --retries 0
cargo build -p codex-cli --bin codex
just test -p codex-tui --test all -E 'test(security_profiles)' --retries 0
```

Four new security-view snapshots and 21 affected status snapshots were inspected
and accepted. Changes are the requested/readiness row, wrapping, corresponding
card dimensions and the current 0.1.38 version (some inherited snapshots still
contained 0.1.31). No unrelated provider/permission behavior is changed.

Review 1 froze `51bc1b2f4` against allocation `4f263ca73`, used the global
structured autoreview helper with `/opt/homebrew/bin/codex`, explicit
`--model gpt-6-astra --thinking high`, and kept artifacts in
`/Volumes/CorbanuDrive/Corbanu/.codex-work/security-round5-ui/review-1-astra.log`.
It failed solely on CLI/model compatibility; no model substitution or uncounted
retry occurred. The coordinator owns subsequent numbered attempts and the
five-invocation ceiling.

Coordinator review 2 used the same structured helper with app-bundled
Codex 0.153.1, `--model gpt-6-astra --thinking high --mode branch --base 4f263ca73`.
Review 3 used the Corbanu wrapper with `--model claude-fable-5-1-plan --thinking high`
through private TMUX. These were code reviews, not substitutes for the actual-key
TUI tests. Both verified the observation-only boundary and reported no findings.
The combined source f60d15f16 plus module sort subsequently passed UI 235/235
and all three actual-key workflows. Final combined source dd2adb72b, including
the realtime fix, passed Core 94/94 and actual-key TMUX 3/3 after fix/format;
the unchanged UI suite evidence is reused explicitly. The coordinator archived
PF-24-S01 with the [final evidence](../../planning/parallel-handoffs-2026-09-04-round-5/combined-qualification.md).
This completes only the observation-only sprint, not protected activation or release.

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
