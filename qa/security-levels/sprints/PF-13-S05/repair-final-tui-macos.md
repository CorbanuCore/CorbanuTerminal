# Final candidate — macOS actual PTY proof

Result: **PASS**, automated keys/visible checkpoints; human acceptance pending.
Candidate source: f6ec1c75f6c389f68c5350df795a7f3d30e7fde4, version 0.1.35.
SHA-256: 107573e81ce1ab0eb808778e1e9eaacbcc883fde24e1991e69ed39ae6920104b.
This hash matches the final Mac canary report, the rebuilt workspace executable,
and the frozen byte-identical copy used for this PTY run. Copying avoided
concurrent Cargo probe builds replacing the executable during interactive QA.

Executable: /tmp/corbanu-pf13-tui.tIRb0a/candidate/corbanu, copied from
/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02/codex-rs/target/debug/corbanu.
Test cwd: /tmp/corbanu-pf13-tui.tIRb0a/work; home/log isolation and loopback
fixture are the same as repair-tui-macos-checkpoints.md. RUST_LOG=trace; env -i;
no real credentials or external provider requests. tmux socket pf13fixqa,
sessions final-repair and final-resume, 120×40 terminal.

Run began 2026-08-28 06:36:46 UTC. Corbanu session:
01a04715-cdcf-78f3-96e9-735ea818fbf0. Started the exact branded executable
with --no-alt-screen, -c log_dir and -C pointing to the isolated directories;
restarted it with resume --last and the same arguments/home. This follows the
test-tui keys/trace/log procedure while substituting the exact Corbanu candidate
for the guide's generic just codex launcher.

1. Startup displayed Corbanu Terminal v0.1.35 and an available composer.
2. Typed PF13_SUCCESS prompt; Enter in a separate call; observed PF13_TUI_OK.
3. Typed PF13_CANCEL prompt; Enter separately; observed Working at four seconds,
   then Escape; observed Conversation interrupted and a usable composer.
4. Typed PF13_RECOVERY prompt; Enter separately; observed PF13_TUI_OK.
5. Typed /exit; Enter separately; restarted with resume --last; prior success,
   cancellation and recovery history were visible.
6. Typed PF13_RESUME_SUCCESS prompt; Enter separately; observed PF13_TUI_OK.
7. Typed /exit; Enter separately; stopped the disposable mock server with Ctrl-C.

The fixture's uncatalogued model name produced the known metadata warning shown
below; no real-model performance/compatibility claim is made. This verifies
immediate exit/resume, not long-idle remote reconnect behavior. The separate
production-hook subprocess tests prove secret-bearing panic containment; this
PTY run does not deliberately crash a live user terminal. Live-repository release
flows, Windows final-tree proof and Travis's human sign-off remain pending.

## Pending response before Escape

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     loading   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› Explain this codebase

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the final candidate qualification marker.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› PF13_CANCEL: Hold the final candidate request for cancellation.


• Working (4s • esc to interrupt)


› Explain this codebase

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```

## Resumed session and final successful request

```text

╭──────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.35)                │
│                                              │
│ model:     gpt-5.6   /model to change        │
│ directory: /tmp/corbanu-pf13-tui.tIRb0a/work │
╰──────────────────────────────────────────────╯


› PF13_SUCCESS: Return the final candidate qualification marker.


• PF13_TUI_OK


› PF13_CANCEL: Hold the final candidate request for cancellation.


■ Conversation interrupted - tell the model what to do differently. Something went wrong? Hit `/feedback` to report the
issue.


› PF13_RECOVERY: Return the final marker after cancellation.


• PF13_TUI_OK


› PF13_RESUME_SUCCESS: Return the final marker after restart.


⚠ Model metadata for `gpt-5.6` not found. Defaulting to fallback metadata; this can degrade performance and cause
  issues.

• PF13_TUI_OK


› Find and fix a bug in @filename

  gpt-5.6 default · /tmp/corbanu-pf13-tui.tIRb0a/work · Corbanu Terminal · TPS: -- tok/s
```
