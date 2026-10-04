# Unblocking the manager cycle, and what the reviews caught

**Fable, 2026-09-16.** The outage recorded in `briefing-budget-20260916.md` is
closed. Every briefing overflowed 65,536 bytes and `fit_briefing` could not
shrink the cost, because it came from evidence referenced by actions and
allocations rather than events. No cycle could be prepared, so nothing could be
dispatched.

## What was wrong with my first fix

Expansion was made prospective: measure the briefing with the body included,
take the body back out if it does not fit, and name the skipped digest in
`evidence_omissions`. That is the right shape, and it worked on live state.

It was still wrong. The reserve I held back for the omission report was computed
at the moment a body was *accepted*, but every refusal *after* the last
acceptance adds a digest to the report and to the pre-existing `missing` lists
without any further measurement. The reviewer did the arithmetic: the entry
costs `97 + 67*D` while the reserve had held back `128 + 72*(d_last+1)`, so two
trailing refusals already overshoot — and trailing refusals are the normal
shape, because once the briefing is nearly full every remaining body is refused.
The failure mode is exactly the outage the change exists to fix.

**The lesson is not "reserve more".** An estimate cannot be what enforces a hard
limit. The report is now rebuilt from scratch and the *finished* briefing is
sized; if it does not fit, the largest remaining optional body is evicted and
reported, ties broken on digest so the same packet always yields the same
briefing. The reserve survives only as a cheap way to stay near the limit.

Live result: 19 events brief in 65,389 bytes, where 24 could not brief at all.

## Tests that would have passed anyway

My first attempt at a regression test swept a range of sizes and asserted the
briefing fit. Removing the eviction pass did not make it fail — the shapes I had
chosen never overshot. I searched the parameter space for fixtures that actually
discriminate and pinned three; two of the three fail without the pass.

A test that cannot fail is not evidence. Sweeping a plausible range is not the
same as finding the boundary.

One guard is honestly untested: a second reference to a refused digest cannot
reinstate it. No fixture discriminates it, because the eviction pass subsumes the
outcome. The second review judged it still worth keeping, since it fires during
collection and prevents a small body from being both present and reported as
absent. The property it protects — nothing reported unreadable may actually be
present — is asserted directly instead.

## The registration guard I wrote too tight

`register_sprint(replace=True)` exists to repair a draft sprint's document
reference through the audited API. Review asked it to also scan `action_history`,
because `state["actions"]` is pruned. Correct. I then wrote the guard to refuse
on *any* allocation as well, and it immediately refused the one repair it was
built for: PF-60-S03 carries `acct-inspect-impl-01`, which had never been
dispatched.

What disqualifies a repair is work, not preparation. An allocation nobody has
been dispatched against is a frozen offer and says nothing about the document.
The guard now refuses on any action, live or archived, and on a consumed
allocation as a second line of defence — and the note no longer claims the
consumed marker is set by dispatch, because nothing in the coordinator writes it.

## Sequence that followed, in order

1. Received the briefing fix and the registration confinement; pushed.
2. Repaired the live PF-60-S03 `source_path` from an absolute worktree path to
   the repository-relative path, through `register_sprint(replace=True)`.
3. Prepared a manager cycle — the first since the outage — proposing the
   `prepare_successor` action for PF-60-S03.
4. Activated PF-60-S03 from S02's verified receiving commit `533a16077`, under
   Travis's open accounting gate.
5. Prepared and dispatched `acct-inspect-impl-01` against the frozen coordinates.

## Standing correction

Keep `verify()` evidence terse. The bodies that caused this outage were my own
prose verdicts, each worth writing once and then paid for on every later cycle.
Reasoning belongs in a repository document like this one, which is read when
someone needs it rather than inlined into every briefing.
