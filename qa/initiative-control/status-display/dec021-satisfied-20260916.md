# DEC021's mixed-feed expectation is met, demonstrated with its own control

**Fable, 2026-09-16.** The delivery sprint has carried this since 12 September:
"Resolve retained DEC021 mixed-feed useful-content failure and baseline-provenance
limit without rewriting cases." Both halves are now answered, and neither needed
a code change.

## What DEC021 asked for

From `age-evidence-20260912.md`: *"Original DEC021 expects useful sanitized
evidence to remain available. Invalid mixed feeds currently suppress valid
siblings too."* And separately: *"The independent reviewer also could not find
retained valid pre-poison baselines/validation receipts: exact one-field change
and prior baseline validity remain executor assertions, not independently
supported evidence."*

Two things: valid siblings must survive an invalid neighbour, and the experiment
that shows it must have a verifiable baseline rather than an assertion.

## The demonstration

Run against a private temporary fixture directory, never the live one, with four
real decisions taken from the current feed.

**Control — the pre-poison baseline is valid, verified rather than asserted:**

```
BASELINE status      : valid | decisions: 4
baseline sha256      : d6e18786defa8a44
```

**Exactly one field on exactly one entry is then changed:**

```
one-field change     : decisions[1].revisions[-1].status -> not-a-valid-status
poisoned sha256      : 07354eb0c99ee827
```

**Result:**

```
POISONED status      : invalid | withheld: 1
surviving            : 3 of 4
```

The three valid siblings survive, sanitized, through the schema-3 inspection
path. The poisoned entry, `slack-private-reply-access`, is excluded and never
returned. The count of what was withheld is reported rather than hidden.

That is DEC021's expectation, met.

## Why no code change was needed

`decision_inspection.extract` already does this: it validates the envelope with
an empty decision list, then re-validates each decision individually and keeps
only those that pass, returning the survivors and a withheld count.
`decision_feed.capture` calls it whenever whole-feed validation fails and emits
schema 3. The capability arrived with the decision-inspection work and the ledger
was never updated to match.

The original case was not rewritten or waived, which the item explicitly
forbade. It was run.

## What this does and does not close

Closes: the mixed-feed useful-content expectation, and the baseline-provenance
limit — the baseline's validity is now a verified observation with a digest, and
the one-field change is stated exactly, not asserted.

Does not close, and I am not claiming it: this is a fresh demonstration on
today's code, not a retroactive receipt for the original September run. The
original executor's assertions about *that* run remain assertions. What can be
said is that the behaviour they claimed is the behaviour the system now has, and
that is what the expectation was about.
