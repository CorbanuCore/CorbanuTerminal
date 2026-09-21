# Human testing — September 20 build

This replaces the older handoff notes as the thing to read before testing. It
says what is in the build, what is worth your hour, and what this build cannot
show you no matter how you drive it.

## The build

- Integration commit `3882e39d5` on `integrate/management-workstreams-20260911`,
  which is the pushed tip. Built from a clean detached checkout of that exact
  commit, not from the working tree.
- Two packages were produced from that one commit, both signed with the same
  Developer ID and the same identifiers as every previous install, so Keychain
  and TCC identity are unchanged:
  - the **distribution-clean** package from the canonical builder, and
  - the **developer-accounting** package, which is the clean package with
    `corbanu` rebuilt with collection compiled in and re-signed.
- **Your shortcut opens the developer-accounting build**, because you asked to
  actually be able to test the features. The launcher itself was not modified -
  it follows the stable links under `.codex-work/corbanu-terminal/bin`, and
  those now point at that package.
- One command switches back to the distribution-clean build and one switches
  forward again; both are named in the install record beside this file, along
  with the commit, package paths and per-executable SHA-256.
- The developer build carries the never-for-distribution marker on purpose. The
  package builder refuses to package anything carrying it, which is the guard
  that keeps it out of a shipped binary. Do not hand this package to anyone.

## What actually changed since the app you have been using

The previous package was built on 14 September at `8320109b1`. Most of what
landed since then is manager infrastructure - coordinator, broker, evidence -
which you cannot see from the TUI and should not spend your hour on. Three
things in the product did change.

1. **Permission wording and next-turn semantics**, which is the question you
   ruled on. Applied now means new turns, and a prompt typed during a running
   turn is held and then runs under the latest permissions rather than the ones
   captured when the turn started.
2. **The `/usage` inspector**, roughly a thousand new lines behind
   `/usage requests`. See the honest limitation in test 2 before you try it.
3. **Execution rendering**, from the security lane's display-truthfulness
   finding: a command that never ran should no longer read as though it ran.

## Test 1 — permission semantics, about fifteen minutes

This is the one I most want your eyes on, because it encodes your ruling and
the wording is a judgement call rather than a test result.

1. Open `/status` and look at the Permissions line. It should now read
   `Next turn: ...`. That prefix is deliberate: the card reflects session
   settings, not the authority the running task captured.
2. Start a long turn - something that will run for a minute.
3. While it runs, change permissions. Read the confirmation carefully. It should
   say permissions are confirmed for new turns, and should tell you that a
   prompt sent now is held until the running turn finishes and then runs with
   the latest permissions, and that running work, granted approvals and pending
   approvals are unchanged.
4. Type a prompt while the turn is still running and send it. It should be held,
   not run under the old authority, and it should execute after the turn ends
   under the permissions you just set.

What I want to know is whether the wording tells you the truth in the moment,
not whether it is technically defensible. If you read it and still had to guess
what would happen to your prompt, that is a finding.

## Test 2 — accounting, about twenty minutes

Collection is compiled into the build your shortcut now opens, so `/usage` has
something to inspect after you have done real work in it. Two things are worth
knowing before you judge what you see.

First, a default build genuinely cannot collect, and that is your own decision
working as intended: collection sits behind a non-default Cargo feature, the
feature refuses to compile in an optimised build, the binary carries a
never-for-distribution marker, and the package builder refuses any input
carrying that marker. Your local build is the developer case that ruling
allowed for, not an exception to it.

Second, and this one will catch you out: **collection only activates for the
provider ids `openai` and `anthropic`.** The profile your launcher uses is
`model_provider = "claude-plan"`, which the lane's own test asserts is excluded
along with `openrouter`, `corbanu` and `custom`. So if you open the app and run
`/usage requests` on your usual Claude plan session, you will see nothing, and
that is the provider gate, not a broken inspector.

To actually exercise it, switch the session to an OpenAI model with `/model`
before doing the work you want to measure - your other profile already runs
`openai` - and then inspect. A subscription Claude plan session will not collect
no matter how many turns you run.

Third, the ledger is separate from collection and installs itself on first use,
so the first turns after switching are the first data it can have. An empty
inspector immediately after switching is expected.

After a few real turns on an `openai` session:

- `/usage` for the summary views.
- `/usage requests` and `/usage requests YYYY-MM-DD` for the day inspector.
- Ranges with hour, ISO week and calendar month grouping, and the
  provider-and-model breakdown.

The lane's own reviews left two wording findings open, so they are known rather
than new: the coverage label can over-attribute clips to retention when the
clipping actually came from the window you asked for, and range-scoped counts
sit next to bucket-scoped ones without the scope being obvious. Everything else
about a wrong number is worth reporting.

Collection stays off in every build a user could receive.

## Test 3 — the decision surface, about ten minutes

Two questions are open and both are yours: the packaged-authorisation question
for security, and the accounting acceptance. The security one is parked because
you paused that workstream, so it needs no answer until you unpause. The
accounting one is the acceptance you are about to form an opinion on by doing
test 2.

The recurrence health line on the dashboard should read as recurring, with an
observation age in seconds rather than unknown. If it says unknown, the
published observation has gone stale again and I want to know.

## What this build does not show you

- No live qualification evidence. The isolated executor run on the guest is
  built and proven up to the point of a real turn, and has not been run.
- Security PF-83 is paused at your request. Its last piece of work is committed
  but deliberately unreviewed and unreceived, and is not in this build.
- Nothing here is a release qualification. It is a local signed build of the
  integration tip for your testing, on this machine only.

## Update, 21 September: collection actually works now

The build this note originally described had accounting compiled in and still
collected nothing, on every provider. The routing-key check read the request body
as plain JSON while the transport sees it after preparation, which for this client
means zstd-compressed bytes, so every prepared turn was judged uninspectable and
refused before admission. That is fixed, along with five rounds of review findings
including two defects in the repair itself.

Your shortcut now opens integration commit `333ade4084`, same signing identity and
identifiers as before. What changed for your testing:

- **`/usage requests` will have data** after a few real turns, on any provider and
  dialect this code understands - Responses, Chat or Anthropic - including your
  `claude-plan` launcher profile. The provider-id restriction that made me tell
  you to switch to an OpenAI model is gone.
- **Money is deliberately not shown for plan routes.** A plan has no per-token
  rate, so those turns record tokens with cost reported as unavailable, which is
  distinct from zero and from absent. Metered API-key routes on the built-in
  OpenAI and Anthropic providers are priced.
- **Two shapes are refused on purpose**, and a refusal now means "served but not
  recorded" rather than a failed turn: AWS-signed providers, and providers whose
  requests carry gateway routing keys that could send the work to a different
  upstream vendor. When that happens the log says so.

Verified on the RTX workstation rather than here, because this Mac has been
manufacturing timeout failures while Spotlight re-indexes: core 132/132, core with
the developer feature 136/136, state 166/166, usage 92/92. Raw logs are committed
under `qa/portfolio/agent-cost-accounting/pf-60-s03/rtx-20260921/`.

Still true: collection remains impossible in any build a user could receive, and
the plan-burn and API-equivalent numbers you asked for are the next round, not
this one.
