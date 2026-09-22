# The delivered binary records: an end-to-end check, not a unit test

*PF-60 S03, 22 September 2026. Author: Fable. Build
`integration-build-20260922a`, integration commit `ddcf127cd`, sha256
`e938a1c9…`.*

## Why this exists

Everything this workstream has claimed so far rested on unit and lane tests.
Those prove the code paths behave; they do not prove that the signed artifact
on Travis's shortcut records anything at all. Configuration resolution,
feature gating, the state runtime's installation, the never-ships guard - all
of that sits between a passing test and a working build, and none of it was
covered by the evidence I had been reporting.

So: run the signed binary, against a provider that does not exist, and read
what it wrote.

## What was run

A provider was stood up on loopback that answers one Responses turn and states
its usage: `input_tokens` 1234, of which 200 cached, `output_tokens` 77, of
which 12 reasoning, `total_tokens` 1311. A throwaway `CORBANU_HOME` named it
as `model_provider`, with model `gpt-5.6-terra`. Then, with nothing else
configured:

```
corbanu exec --skip-git-repo-check "say recorded"
```

The turn completed against the mock and the CLI reported 1,111 tokens used.

## What the ledger held afterwards

The state database installed its accounting tables on its own, and for the
successful session:

- **One attempt**, provider `smoke`, model `gpt-5.6-terra`, with its own turn
  identity, request identity and dispatch time.
- **One observation**, carrying exactly what the provider stated:
  `input 1234, read 200, output 77, reasoning 12, total 1311`.
- **No price snapshot at all**, which is the correct answer: `smoke` is not a
  provider any catalogue quotes, so tokens are recorded and no money is
  claimed. This is the case the views now name rather than leaving blank.

The earlier failed run in the same home also left its evidence: a chain of
five attempts linked by `retry_of`, one per physical send, which is what
per-send admission is supposed to look like when a provider keeps dropping the
stream.

## What this proves, and what it does not

**Proved, at runtime, in the artifact that ships to the shortcut:** collection
is on by default in a developer build, the ledger installs itself, a turn on a
provider the build has never seen records an attempt and its tokens, retries
record per send, and a route no catalogue prices records no money instead of
inventing some.

**Not proved here:** money. A priced row needs a built-in provider's own
route, and those are real endpoints this check cannot intercept honestly. The
arithmetic and the catalogue resolution are covered by unit tests; the number
a human should look at is the one Travis's pass on a `gpt-5.6-*` model will
produce against the real route. That remains the outstanding verification for
this workstream, and it is his.
