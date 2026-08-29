# PF-13 outside-expert review attempt

Date: 2026-08-27 Arizona time (2026-08-28 UTC).
**Incomplete: Fable High was interrupted by a provider model fallback.** No
completed findings, clean verdict or independent-review acceptance is claimed.

## Request, authority and integration

Travis requested Fable High as the outside expert for Sprint 13, with all prior
work merged into the branch before review. This is qualification/evidence for
the existing product initiative, not authorization for implementation repairs.
Product: **Required trust boundaries** — “Credentials are referenced by label
and resolved only inside a trusted execution boundary.” Active plan:
`docs/plans/active/p0-security-levels.md`; sprint PF-13-S05 stays `in_progress`.

Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02`.
Branch: `feat/pf-13-s02-scoped-vault-resolver`.

1. Committed the reviewed/tested PF-30 platform repair and all pending evidence
   as `75b7079033c0e4aae2b2bca9472e393b03a51ac2` on its own browser branch.
2. Fetched and preserved `434635dd23b7a35944524cf9fa2b069312a94236`, the remote
   PF-13 Windows follow-up evidence, before integration.
3. Merged all PF-27/PF-26/PF-30 work into the PF-13 branch at
   **`044491b8b02b24a65a84e8da61619d3444e63fe0`**. Those two commits are its
   parents. Only two shared-plan status conflicts required reconciliation;
   Windows completion and newer foundation/browser status were both preserved.
   No source conflict, source edit or history rewrite occurred.
4. Started the outside review only after that merge commit and a clean status.
   No push was requested or performed in this turn.

## Review setup and interruption

Reviewer selection: **Fable 5 / High** in the previously working logged-in
Claude desktop app. Computer Use retained that route after the earlier CLI
authentication failure; this was not a successful Autoreview helper invocation.
The Autoreview scope and no-substitution rules still applied.

Fresh session: `claude.ai/epitaxy/local_ed0e2179-fb33-45ee-a502-90dd3ca9c289`.
Read-only review export: `/private/tmp/corbanu-pf13-fable-review.7KxV5J`.
The snapshot came from the merged commit, excluded project instruction/hook
configuration, and contained the original six implementation diffs, current
source, sprint contracts and existing evidence. [Frozen scope](fable-outside-review-scope.md),
[35-file manifest](fable-review-candidate.sha256) and
[scope numstat](fable-review-scope.numstat) identify the target. No nested
reviewers, project execution, builds, installers or user credentials were used.

The app began under Manual permissions. Only read-only manifest commands were
approved once; a scratch-file redirection was denied, and the same review
continued using direct `shasum -a 256 -c PF13_REVIEW_FILES.sha256` instead. All
35 entries verified. Fable inspected capability/Vault/broker/transport and
native startup paths and began checking helper and policy lifecycle behavior.

At approximately 3m25s in the substantive continuation, the app displayed:

> Fable 5's safeguards flagged this message

It automatically selected **Opus 4.8** with a `[cyber]` category and also showed
Auto permissions afterward. On detecting that fallback, the controller stopped
the response. Some further read-only activity had occurred before the stop;
it is not represented as Fable-authored analysis. The
[interruption capture](fable-review-interrupted.txt) retains the notice and
partial transcript. No final findings JSON or verdict was produced. This is
**incomplete**, not a clean review and not a failed product security test.

No attempt was made to evade the service restriction, silently replace the
reviewer, or implement fixes. Further outside review needs Travis to authorize
another reviewer or restore access to the requested model. The sprint remains
active because other qualification work can proceed.

## Preliminary observation, not expert acceptance

Before the fallback, Fable noted the scoped credential pipeline is currently a
test-only integration seam. Local code inspection agrees: the scoped state
constructor is reached by tests, while `NetworkProxySpec::start_proxy` uses the
ordinary state constructor. The plan explicitly reserves profile composition
for PF-23, so this is a qualification/scope distinction, not a newly accepted
implementation defect. The interrupted review did not finish assessing
raw-secret reachability or the sufficiency of the canary evidence.

## Verification and remaining gates

- All 28 PF-30 final-source hashes verified in the merged worktree; source and
  scripts are identical to the tested browser tip. The Windows merge adds
  evidence/status only.
- All 35 review hashes verified in the export before and after the review.
- The Windows report hash matches the preserved record:
  `23d6861b78552d363e422bf9712f1fd43c970c13bc3c95de810bf8e903b5376b`.
- `python3 -B -m unittest scripts.test_security_credential_canary`: six passed.
- Plan/sprint checkers and whitespace validation pass; strict MkDocs build
  passed using the existing environment, output
  `/tmp/corbanu-pf13-review-docs.E3TmzR` (existing archived-link INFO only).
- No new complete Core, canary/transport, native TUI, live-repository or human
  acceptance run is claimed. Prior tests retain their original candidate IDs.

The 135 recorded macOS full-Core failures still need triage and a clean rerun.
Independent review is still incomplete. PF-13's historical Windows follow-up
is preserved; it does not certify this newly integrated candidate or close the
separate PF-30 Windows browser gap. Travis Good's final human acceptance,
integrated TUI/live-repository, due benchmark and release gates remain open.

The development skill kept this work within PF-13-S05 evidence and preserved
those gates. Autoreview's no-substitution rule caused the review to pause;
no product code was changed in response to partial analysis.
