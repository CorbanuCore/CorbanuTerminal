# `/usage requests` can only ever say the ledger is not installed

**Fable, 2026-09-16.** The code-blind qualification of the S03 inspector came
back blocked, and the reason is not a defect in the inspector. It is a gap
between the sprint that collects and the sprint that inspects, and closing it is
a product decision rather than mine.

## What the executor saw

The package was built from the integration tip `b4513f6cc`, staged into a fresh
sealed-guest root, and run. A real root-and-child inference pair completed —
`ROOT_OK CHILD_OK` — so the product worked. The inspector, before and after that
run, said:

> Unavailable — accounting ledger not installed. Collection remains off.

The three source hosts were denied from inside the same runs, so the executor
was genuinely code-blind.

## Why, established from the source afterwards

I looked, the executor did not. `Config::accounting` is an `AccountingMode`
whose `#[default]` is `Disabled`. The only places it is ever set to anything else
are in tests: `accounting_tests.rs`, `accounting_chat_tests.rs`,
`accounting_responses_tests.rs`, `accounting_policy_tests.rs`. There is **no**
`config.toml` key, **no** environment variable, and **no** slash command that
sets it. The enum's own doc comment says as much: *"Internal direct API-key Chat
sampling; no public activation."*

The store-side install path exists in `codex-state`
(`accounting_native.rs`, "explicitly installed accounting") but nothing in
`core` or `tui` calls it.

So in a shipped binary, on any machine, `/usage requests` has exactly one
reachable output. Everything the last four review rounds argued about — the
breakdowns, the unknown populations, the coverage wording, the bucket scoping —
is behind a door that cannot be opened.

## What this does and does not mean

It is **not** a defect in S03. The inspector is correct, extensively tested and
independently reviewed four times; its unit and integration tests construct the
store directly, which is why they pass and why nothing caught this earlier.

It **is** a gap in the sprint's acceptance. PF-60-S03's stated acceptance is
"the user can explain each displayed total using its constituent requests
without inspecting storage." No user can reach that state today.

It is also a fair reading of history rather than a surprise: PF-60-S02 was
archived with collection OFF and "no public activation" recorded as deliberate,
and S03 was written assuming data would exist. Nobody joined the two. I did not
join them either when I dispatched the qualification, which is why the executor
spent a run discovering it. That was my planning error and the run was still
worth it — it produced four real findings in the unavailable state alone and
proved the build-and-stage pipeline end to end.

## Why I am not deciding this

Making collection activatable is a decision about recording usage data on a
user's machine. Travis's grant to me is the accounting gate — receiving,
reviewing and integrating the work — not choosing whether and how the product
starts collecting. The shapes differ in ways that matter to him and not to me:
a config key, an explicit opt-in command, a first-run prompt, or a
developer-only activation that never ships. Each has a different privacy and
support story.

What I can say as the engineer: shipping `/usage requests` in its current state
would give every user a command that can only report its own unavailability, and
that is worse than not shipping the command.

## Work that does not wait on the decision

Four findings from the same run are actionable now and do not touch activation:
narrow-terminal scrolling hides the unavailable reason; the short-form
unavailable view omits the dates the user selected; hour queries are answered in
day-oriented wording; and the inspector command is rejected in the child
watcher. Those are dispatched separately.
