# PF83 — user-authorized integration checkpoint

September14: Travis explicitly requested merging the fixes into
`integrate/management-workstreams-20260911` and pushing that branch. This
authorizes the integration/push, not installation or a functional-acceptance claim.
The running application, profile, launcher and stable executable links remain
unchanged. Sprint PF-83-S01 stays in_progress; PF27 stays paused/draft.

## Source and supporting verification

Source base005cc644f59b1e762e5497b329e106c67925d4ed, branch
fix/live-permission-transition-20260913. Frozen v3 patch SHA256
b5031c64617512dcab488c8ad6c96690a6d5ad9d47b9957a2744d1bc13408142.
34candidate paths,639non-test+1043test=1682 hand-authored Rust lines within
recorded750/1750 exception. Generated schemas counted separately.

Focused final v3:353/353passed,8695skipped,48.045s;
nextestb8d44616-f3e3-4db7-99c9-77becff03574. Exact command and hashes:
[verification-command-v3.md](verification-command-v3.md),
[candidate-manifest-v3.md](candidate-manifest-v3.md).
No source changes after this run. Earlier failed attempts and v1/v2 packages,
patches and raw logs remain locally preserved, not silently replaced.
Repository handoff includes selected text evidence rather than build caches,
private artifacts, raw session histories or redundant generated source patches.

## Final Fable review disposition

Fresh Fable5.1High review03 through Corbanu/TMUX completed2026-09-14T08:27:40Z.
Raw [review03](fable-review-03.json) and [readable result](fable-review-03.txt)
are retained. Reviewer verdict **patch is correct**, confidence0.72; helper
exit1 because two P3 findings remain. This is NOT a zero-findings review.
Prior P2 defaults-propagation and held-input issues were resolved and reviewed.

Known nonblocking follow-ups, accepted for this integration checkpoint:

- First attempt against an unknown legacy server can retain server-owned
  permission routing after explicit method rejection; known-unsupported routing
  is fixed. Do not claim complete old-binary compatibility qualification.
  Any later repair must distinguish a rejected method from legacy empty accepted
  responses that may actually have applied settings; no unsafe blind retry.
- Server uncertain-outcome hint can contain Rust Debug text. Classification is
  still uncertain, not false success.
- Deferred/resubmitted input can leave a duplicate optimistic transcript cell;
  previously recorded cosmetic follow-up, not duplicate command execution proof.

Two repair cycles are complete. No more changes were made merely to seek a clean
review. Review budget4/5 (design+three code reviews), independent evidence check
reserved. User-requested merge does not erase findings or waive a frozen case.

## Open qualification, not waiting on application installation

Independent exact-package real-key F01–F11 execution in declared test contexts,
separate evidence review and named-human acceptance remain open. Synthetic
native isolation denies source/auth/network/IPC/process probes, but that strict
policy times out launching the held-v1 package; earlier permissive positives
cannot qualify it. Mediated inference is not yet wired. V3 is not installed or
packaged by this merge. See both preparation/native-infrastructure receipts.
No Windows run or actual old-server binary matrix is claimed.

Receiving owner uses existing single-writer integration lock, frozen source and
destination commit, clean-checkout checks and durable receiving-test receipts.
Non-fast-forward pushes are forbidden. Preserve concurrent integration changes;
actual resulting commits and remote verification are recorded in private receipt
and user handoff, not invented here before Git operations.
