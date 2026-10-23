# My compaction fix created a new hazard, and the coordinator caught it

**Fable, 2026-09-16.** A manager cycle came back `owner_hold` with
`core_rejected`. The coordinator was right to refuse and the fault was mine,
two layers deep.

## What happened

Every accounting unit's cycle script built its allocation by copying the
previous one — `copy.deepcopy(s['allocations'][previous])` — and then
overriding the brief, the base commit and the timeout. That inherited the
model, the worktree, the branch and the twenty-one-path write scope without
restating them, which seemed tidy.

Earlier this morning I put allocation compaction inside the poll loop, because
allocations accumulating had pushed the briefing over its byte limit. Compaction
replaces a terminal allocation with a consumed stub:
`inputs: {consumed: true, original_digest: …}`, `resources: ["consumed"]`,
`scope: ["consumed"]`.

So the copy source became a moving target. By the time I built the next
allocation, its neighbour had been compacted, and I produced a hybrid:
`consumed: true` sitting beside a real `brief_file` and `base_commit`, with a
write scope of the literal string `"consumed"`. The manager could not see frozen
inputs for it — the briefing collapses consumed allocations to a digest index,
exactly as designed — so it proposed an action with a prose placeholder instead
of verbatim inputs, and the coordinator rejected the action because inputs must
match the allocation exactly.

Three mechanisms behaved correctly and one input was garbage. That is the right
ratio, and it is why nothing bad reached a worker.

## The fix

The cycle no longer copies a neighbour. The allocation is built from the
sprint's own `write_scope` front matter, which is authoritative, is checked by
`docs/sprints/check.py`, and cannot be turned into a stub by compaction. The
script asserts the scope parses to twenty-one paths before using it, so a
malformed front matter fails loudly rather than shipping a short scope.

The live hybrid was replaced through `put_allocation(replace=True)` with the
same reconstruction, and the held manager run was failed with the reason
recorded rather than left claimed.

## The lesson, which is not "be careful"

I added automatic compaction to fix one problem and did not ask what else read
the thing it mutates. The copy pattern was safe until the moment it wasn't, and
nothing about the copy site said "this may be a stub by the time you read it".
Deriving state from a neighbour is a dependency on that neighbour's lifecycle;
deriving it from the sprint document is a dependency on a record that is
supposed to be stable and is checked. The second is the right shape, and it was
available the whole time — I chose the copy because it was shorter.
