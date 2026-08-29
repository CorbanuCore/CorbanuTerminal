# Repaired candidate — actual macOS PTY checkpoints

Candidate: a7ae94e4c9c01924c896d9f10b1f588f1727fc67, Corbanu 0.1.35.
Binary SHA-256: cc60e86beccffcbbbcf8fa5239054f5d7041b01619ebc94f63ec8fe699ddf30d.
tmux socket: pf13fixqa; 120 columns × 40 rows. Exact Corbanu executable
was launched directly (rather than the guide's generic codex target) to test
the intended branded candidate. RUST_LOG=trace, isolated log directory and homes;
env -i removed inherited credentials. This is immediate restart/resume proof,
not qualification of the separately reported long-idle remote reconnect issue.

Test work directory: /tmp/corbanu-pf13-tui.tIRb0a/work (disposable empty Git repository).
Home: /tmp/corbanu-pf13-tui.tIRb0a/home; logs: /tmp/corbanu-pf13-tui.tIRb0a/logs.
Provider: synthetic loopback Responses SSE, http://127.0.0.1:62306/v1; no API key,
no external provider call, no tools requested. Every success emits PF13_TUI_OK;
a PF13_CANCEL request emits response.created then delays 20 seconds.
The fixture model name gpt-5.6 produced the visible metadata fallback warning;
this run makes no real-model compatibility claim.

Keys: type PF13_SUCCESS prompt, send Enter separately; observe marker.
Type PF13_CANCEL prompt, Enter separately; observe Working, press Escape.
Type PF13_RECOVERY prompt, Enter separately; observe marker. Type /exit,
Enter separately. Relaunch the same binary with resume --last; observe restored
history. Type PF13_RESUME_SUCCESS prompt, Enter separately; observe marker.
Finally /exit, Enter separately; stop the disposable provider with Ctrl-C.
Session: 01a04706-a0b0-7480-bdbd-a4086c764664. Automated result: PASS.
Named human acceptance remains pending; this is not live-repository release QA.

## Startup

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     loading   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Successful request

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     loading   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the fixed qualification marker.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Pending request before Escape

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     loading   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the fixed qualification marker.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› PF13_CANCEL: Hold this request so I can cancel it.


◦ Working (4s • esc to interrupt)


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Cancelled request

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     loading   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the fixed qualification marker.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› PF13_CANCEL: Hold this request so I can cancel it.


■ Conversation interrupted - tell the model what to do differently. Something went wrong? Hit `/feedback` to report the
issue.


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Recovery

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     loading   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the fixed qualification marker.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› PF13_CANCEL: Hold this request so I can cancel it.


■ Conversation interrupted - tell the model what to do differently. Something went wrong? Hit `/feedback` to report the
issue.


› PF13_RECOVERY: Return the fixed qualification marker again.


• PF13_TUI_OK


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Restored history after resume

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the fixed qualification marker.


• PF13_TUI_OK


› PF13_CANCEL: Hold this request so I can cancel it.


■ Conversation interrupted - tell the model what to do differently. Something went wrong? Hit `/feedback` to report the
issue.


› PF13_RECOVERY: Return the fixed qualification marker again.


• PF13_TUI_OK


› Improve documentation in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Successful request after resume

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the fixed qualification marker.


• PF13_TUI_OK


› PF13_CANCEL: Hold this request so I can cancel it.


■ Conversation interrupted - tell the model what to do differently. Something went wrong? Hit `/feedback` to report the
issue.


› PF13_RECOVERY: Return the fixed qualification marker again.


• PF13_TUI_OK


› PF13_RESUME_SUCCESS: Return the fixed qualification marker after resume.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› Improve documentation in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

