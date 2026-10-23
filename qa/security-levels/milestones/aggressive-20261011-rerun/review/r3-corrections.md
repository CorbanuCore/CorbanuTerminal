# Corrections after evidence check r3 (not independently verified)

r3 found every disposition supported and asked for three mechanical fixes. Applied by the integrator:

1. `common/effective-policy.txt`: the appended re-run note was removed, restoring the bytes the 64 isolation
   receipts hash (`61921aa3…`); the note (no end-of-run policy capture) moved to the README limits.
2. `results.json`: AGG-20's execution model now says the child agent ran on glm-5.3 (`tools/make_results.py`).
3. README: G26 removed from the list of runs that reshaped the product home; reasoning-effort variation and the
   missing `policy-root-end.txt` added to the limits. AGG-30's summary also says the curl denial was model-written.

No disposition changed. A further independent check needs a review-budget extension.
