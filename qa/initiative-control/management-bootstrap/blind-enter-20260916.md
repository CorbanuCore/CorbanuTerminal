# The model drift was my dispatcher pressing Enter on a modal

**Fable, 2026-09-16.** I escalated a quota policy question to Travis. It was not
a quota policy question. My own launcher was answering a dialog it could not see,
and the dialog's highlighted default was "switch to a cheaper model".

## What the code actually did

`dispatch.py`, after sending START, waited for the session to begin work like
this:

```python
for _ in range(40):
    subprocess.run(T + ['send-keys', '-t', short, 'Enter'], check=True)
    time.sleep(4)
    if re.search(r'Working \(|esc to interrupt', pane()): break
```

Forty blind Enters, one every four seconds — over two and a half minutes of
pressing a key into a full-screen terminal application without ever asking what
was on it.

`codex-rs/tui/src/chatwidget/rate_limits.rs` raises a modal when usage crosses
`RATE_LIMIT_SWITCH_PROMPT_THRESHOLD = 90.0`. Its options are, in order: switch to
`gpt-5.6-luna`, keep the current model, keep it and stop asking. The first is
highlighted. One of my Enters confirmed it.

Both symptoms fit exactly and I had already written both down without connecting
them: the ACK named astra, the rollout then contained Luna, and the run printed
`WARNING: START sent but no Working observed` — because the Enters were being
consumed by the dialog rather than starting the turn.

## The fix, and the proof

The launcher now reads the pane before every Enter. When it sees the rate-limit
modal it selects "keep current model and stop asking" explicitly, logs that it
did, and retypes START, because a modal that appears mid-typing swallows it and
leaves the session idle with its ACK already given.

Dispatched again on the same allocation:

```
MODAL: rate-limit switch prompt; keeping gpt-6-astra
```

and the session's rollout now records `gpt-6-astra` only, with the footer
reading `GPT-6-Astra high`. The modal happened, was answered correctly, and the
frozen model held.

## What I got wrong, in order

1. Saw the drift, correctly refused the work, built a detector. Good.
2. Concluded the account was rate limited and escalated a policy question to
   Travis. Half right: usage really is above 90%, which is why the modal appears
   at all. Wrong about the mechanism.
3. Probed with `corbanu exec`, which is non-interactive and therefore never sees
   the modal, and concluded the limit was transient. Resumed on that basis and
   lost a second dispatch. Wrong, and I published it before checking.
4. Finally read my own launcher.

Step 4 should have been step 2. The detector I built in step 1 was pointed
outward — at the model, at the service, at the quota — when the thing that had
changed was my own code path under a condition it had never met before. I spent
two dispatches and an escalation establishing that a dialog box exists.

## What still stands

Usage above 90% is real and Travis should still see it; the modal is a symptom of
something true. But it is a warning, not a refusal, and it no longer costs us a
downgrade. The decision I raised is downgraded from blocking to informational,
and the drift check stays in the poll cycle — it is what caught this, and it is
what will catch the next thing I have not thought of.
