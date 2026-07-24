# PFTerminal 0.1.22

## Added

- Claude Opus 5 is available through both direct Anthropic API-key
  authentication and Claude Code Plan authentication. Opus 4.8 is removed
  from the model picker while existing saved 4.8 sessions remain resumable.

## Fixed

- Kimi Code action turns now distinguish a finished answer from a progress
  checkpoint before ending the turn. Unfinished work continues automatically,
  repeated no-progress responses stop with a bounded warning, and other
  providers retain their existing terminal-stop behavior.
- The Windows installer now detects x64 and ARM64 safely across Windows
  PowerShell and .NET variants where `RuntimeInformation.OSArchitecture` is
  unavailable.

## Qualification status

- Direct Anthropic and Claude Code Plan live requests both completed using
  Claude Opus 5, and the provider, model-picker, Telegram alias, and Claude
  pane regressions pass.
- Kimi lifecycle regressions cover progress-to-tool continuation, terminal
  answers, malformed assessments, latency, repeated no-progress stops, and
  unchanged behavior for reliable-stop providers.
- The installer architecture suite passes under Windows PowerShell 5.1 and
  PowerShell 7, including the environment-fallback path, and is now a required
  step in the native Windows release job.

Previous release: 0.1.21.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).

---

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
