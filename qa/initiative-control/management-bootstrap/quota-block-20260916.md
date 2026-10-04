# The account is blocked until 2026-09-20, and I measured the edges before asking

**Fable, 2026-09-16.** Both running workers stopped at about 21:43 local with the
same line:

> You've hit your usage limit. Visit … to purchase more credits or try again at
> Sep 20th, 2026 1:44 AM.

I raised `worker-model-quota-20260916` this morning as informational, and Travis
reasonably left it. It is no longer informational, so before escalating I went
and established what is actually true rather than describing the symptom.

## What I measured

I launched a throwaway session against a temporary profile — not a worker, not an
action, nothing charged against the ledger — and established three things.

**1. `gpt-6-luna` is not a real model on this account.** Submitting to it returns
a 400:

> The 'gpt-6-luna' model is not supported when using Codex with a ChatGPT account.

This matters beyond tonight. I have been carrying "luna for synchronisation,
astra for important work" as an operating rule, and the model id in that rule
does not exist here. The real one is `gpt-5.6-luna`. If I had reached for the
fallback under pressure it would have failed for a reason that had nothing to do
with quota, and I would have spent the outage debugging the wrong thing.

**2. The block is account-wide, not per model.** `gpt-5.6-luna` — offered by the
product itself as the lower-credit option — hits the identical limit on submit.
Switching tiers buys nothing.

**3. The account reports two usage-limit resets available.** The banner is
explicit: *"You have 2 usage limit resets available."*

**4. The Claude lane is unaffected.** The manager cycle and every independent
review ran normally on Opus 5.0 High in the hour before and after the block,
including the review that cleared accounting's inspection bound.

## What it costs right now

Two workers died mid-task. Both worktrees hold real, uncommitted work:

- `acct-activation-26` — the harness opt-out and the enforced never-ships guard.
- `tui-snapshot-rot-27` — the second snapshot repair round, mid-build.

Neither is lost. Neither can be received, because neither returned and neither
ran its gate, and an unverified partial is not a candidate no matter how much I
want the gate green. The knock-on is that the security receipt stays open, since
the honest TUI gate it needs is precisely what the snapshot round was finishing.

Not blocked: reviewing, receiving, attribution, the dashboard, and integrator
work. I received accounting's inspection bound *after* the block landed.

## The one I would not take without being asked

The tempting move is to put implementation workers on Opus 5.0 High, since that
lane demonstrably has capacity. I am recommending against it as the first choice,
and the reason is worth writing down rather than leaving as a preference.

Every genuinely important defect found tonight was found by a reviewer that was
not the author. The bound that would have hung a busy host instead of refusing.
The lock timeout reported to the operator as a credential problem. The corruption
notice that overflowed the publication limit and blanked the panel it was meant to
fill. And the one that mattered most — the correction to my own published claim
that F04 was a boundary violation, when the specification permits the behaviour.

If the author and the reviewer become the same model, that independence gets
thinner, and it is the only thing standing between "the tests pass" and "the
change is right". I would rather spend a reset.
