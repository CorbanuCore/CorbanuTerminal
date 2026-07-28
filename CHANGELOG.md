# PFTerminal 0.1.24

## Added

- The canonical model catalogue now declares model-specific Chat Completions
  reasoning protocols. Kimi K3 routes expose their supported low, high, and
  max effort levels and require preserved reasoning across tool turns.

## Fixed

- Kimi K3 now replays returned `reasoning_content` on the same assistant
  message as its tool calls, as required by Moonshot's multi-turn protocol.
  Selecting no reasoning is rejected instead of silently running a different
  effective model.
- OpenRouter requests now use the gateway's sticky `x-session-id` header rather
  than an unsupported `prompt_cache_key`. Hosted web search is opt-in on
  OpenRouter so its prompt transform does not defeat provider prefix caching.
- OpenRouter and Ambient reasoning controls now reach the provider using the
  supported wire fields. Vercel routes pin the intended upstream when needed
  and apply native reasoning control on Anthropic, Responses, and Chat wires.
- Deep reasoning accepts the catalogue's first-class `xhigh` value on Ambient,
  Z.AI, and Vercel instead of silently disabling reasoning.
- Chat streams ending with provider error finish reasons enter the bounded
  retry path without committing partial output as a successful turn.
- Provider capability detection uses normalized gateway endpoints instead of
  mutable display names, and non-reasoning OpenRouter models no longer receive
  parameters that can make valid upstream routes ineligible.

## Qualification status

- Kimi's preserved-thinking request contract is covered by model-catalogue,
  request-builder, stream parser, retry, and multi-turn integration tests.
- A two-wave live Queuecraft comparison through OpenRouter passed 35/35 tests
  and 7/7 hidden probes on every run. PFTerminal solved 2/2 in 225.626 seconds
  for $0.525786 of directly billed spend; current Hermes solved 2/2 in 547.410
  seconds for $2.217944.
- Provider, protocol, model-catalogue, and exact debug-package builds pass on
  the release candidate.

Previous release: 0.1.23.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).

---

# PFTerminal 0.1.23

## Added

- Native multimodel orchestration now supports durable v2 mailboxes, persisted
  crew identity and runtime metadata, exact provider/model overrides, bounded
  worker residency, and root/descendant recovery across resume.
- Spawn-time runtime metadata now exposes billing class, model tier, estimated
  token cost, reasoning efforts, service tiers, and vision capability so agent
  allocation can distinguish plan capacity from metered API spend.
- The canonical model catalogue now owns orchestration eligibility, capability
  tier, vision support, reasoning effort, and typed billing metadata. Spawn
  guidance is generated from that catalogue and prefers compatible plan-backed
  runtimes before metered routes when policy allows.

## Fixed

- Anthropic Messages and chat-completions streams that terminate without text,
  reasoning, or a tool call now fail the turn instead of reporting a successful
  empty completion. Assignment managers therefore enter the normal
  retry/error path rather than silently completing and leaving their pane
  unresponsive.
- Anthropic prompt-cache tool definitions remain stable across ordinary turns.
  The edit fallback activates as state after repeated grammar failures instead
  of mutating the advertised tool schema, while structured edit/write tools
  remain available as stable low-frequency fallbacks.
- Native mailbox delivery, provider-auth preflight, restored descendant
  identity, root resume reconciliation, and manager addressability under worker
  saturation now share the same native scheduling path.
- Native spawn paths now enforce exact provider/model catalogue membership and
  operator authorization. Full-history forks inherit their parent's runtime;
  fresh or partial forks may use an explicit eligible override. GPT-5.5 is
  classified as legacy and is never selected for orchestration.
- Child turns stop with a visible bounded-budget warning after the configured
  model-request limit instead of looping indefinitely. The default limit is 24
  model requests per child turn.

## Qualification status

- Provider adapter tests cover empty Anthropic/chat streams and tool-only
  completions. Prompt-cache, structured-edit/write, spawn runtime, hierarchy,
  sandbox, and model-economics regressions pass on the release candidate.
- Model-catalogue, provider/model authorization, runtime inheritance,
  model-aware spawn-description, and bounded child-turn regressions pass on the
  release candidate.
- Native orchestration qualification covers required Claude, Grok, Fable, and
  Kimi runtime mappings plus live mailbox/resume evidence recorded in the
  multimodel orchestration journal.

Previous release: 0.1.22.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).

---

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
  PowerShell and .NET variants, selects and extracts the published Windows ZIP
  package, and upgrades by retargeting the versioned install junction without
  deleting a locked legacy executable.

## Qualification status

- Direct Anthropic and Claude Code Plan live requests both completed using
  Claude Opus 5, and the provider, model-picker, Telegram alias, and Claude
  pane regressions pass.
- Kimi lifecycle regressions cover progress-to-tool continuation, terminal
  answers, malformed assessments, latency, repeated no-progress stops, and
  unchanged behavior for reliable-stop providers.
- The installer suite passes under Windows PowerShell 5.1 and PowerShell 7,
  including architecture fallback, ZIP extraction, and a locked-executable
  upgrade, and is now required by the native Windows release job.

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
