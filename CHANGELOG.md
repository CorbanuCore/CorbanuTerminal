# PFTerminal 0.1.21

## Fixed

- Telegram now carries the configured sandbox policy through thread creation,
  thread resume, and every turn instead of silently falling back to
  `workspace-write`. This fixes shell commands failing before execution with a
  `bwrap` loopback error.
- Kimi Code and other chat-compatible providers now use the native turn
  lifecycle without a hidden model-based completion assessment. Tool calls
  continue normally, while a final text response ends the turn without up to
  three extra inference requests or a misleading completion warning.
- Increased the Intel macOS release-build timeout so healthy cold builds have
  time to finish packaging.

## Qualification status

- The Telegram connector suite passes 119 tests, including sandbox-mode
  propagation coverage.
- Live Telegram qualification resumed the configured Kimi thread with
  `danger-full-access` and successfully ran `pwd` and `rg --version` without a
  sandbox or `bwrap` failure.
- Two chat-provider lifecycle regressions prove that a text stop uses one
  inference request and a tool-call turn uses exactly the expected two.
- All 47 model-provider-info tests pass.

Previous release: 0.1.20.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
