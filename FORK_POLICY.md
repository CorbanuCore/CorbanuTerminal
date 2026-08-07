# PF Terminal Runtime Divergence Ledger

PF Terminal follows the pinned upstream Codex runtime by default. A semantic
runtime difference belongs here only when a provider contract or a PF product
surface requires it. New entries require an owner, evidence, configuration
surface where appropriate, tests, and a removal condition.

## Model and provider catalogue

- Owner: PF Terminal model-routing maintainers.
- Difference: provider/model pairs, billing class, route prices, reasoning,
  vision, service tiers, output limits, and native-agent eligibility are
  resolved from PF Terminal's canonical catalogue.
- Evidence: Codex natively exposes OpenAI routes; PF Terminal supports multiple
  independently billed providers and must never silently substitute a route.
- Configuration: `model`, `model_provider`, `agents.provider_allowlist`, and
  role-specific agent configuration.
- Tests: `codex-models-manager` catalogue tests and
  `multi_agent_v2_spawn_*provider*` core tests.
- Removal condition: none while PF Terminal remains a multi-provider product.

## Provider wire adapters

- Owner: PF Terminal provider-adapter maintainers.
- Difference: Anthropic Messages and compatible Chat Completions routes
  translate the upstream tool and lifecycle model at the serialization
  boundary.
- Evidence: these providers do not accept the OpenAI Responses wire format.
- Configuration: the catalogue selects a typed `wire_api` and exact provider.
- Tests: mock-wire client tests plus route-specific live smoke tests.
- Removal condition: a provider gains full Responses compatibility or upstream
  Codex adopts the same adapter.

## Anthropic payload protection

- Owner: PF Terminal Anthropic adapter maintainers.
- Difference: inline images are omitted oldest-first from a request that would
  exceed Anthropic's documented Messages payload limit; durable history stays
  unchanged and the model sees an explicit omission marker.
- Evidence: reproduced HTTP 413 failures on long visual sessions; Anthropic
  documents a 32 MB Messages API request limit.
- Configuration: `model_providers.<id>.runtime_policy` exposes the request
  body budget, post-413 retry budget, and optional web-search call limit. The
  default Anthropic headroom is 30 MB for the first request and 15 MB for the
  bounded retry.
- Tests: `anthropic_payload_tests`.
- Removal condition: Anthropic supports durable image references or an
  upstream compaction policy handles the same failure without losing evidence.

## Provider cooldown and cross-process lease

- Owner: PF Terminal provider reliability maintainers.
- Difference: metered third-party routes can coordinate cooldown and leases
  across local PF Terminal processes.
- Evidence: provider hammering incidents after retry storms.
- Configuration: provider retry/cooldown settings.
- Tests: lease acquisition, expiry, and crash-recovery tests.
- Removal condition: upstream supplies an equivalent cross-process provider
  admission controller.

## Native agent model allocation

- Owner: PF Terminal orchestration maintainers.
- Difference: native Codex agents may select an eligible provider/model route
  from the canonical catalogue, prefer an authorized plan route when suitable,
  and report the exact runtime and rationale.
- Evidence: multi-provider orchestration is the core PF Terminal product
  requirement.
- Configuration: operator allowlist, explicit provider/model request, role
  requirements, effort, and service tier.
- Tests: native V1/V2 spawn, explicit-pair, refusal, resume, cancellation, and
  metadata-reporting tests.
- Removal condition: upstream Codex supports equivalent multi-provider
  catalogue-aware allocation.

## Reported model identity

- Owner: PF Terminal model-routing maintainers.
- Difference: a provider-reported model string is recorded as route evidence
  in `ModelResponseCompleted`; a string mismatch alone never fabricates a
  cyber-safety reroute or warning.
- Evidence: Claude Plan and gateways can report an upstream alias that differs
  from PF Terminal's catalogue slug. Treating every mismatch as OpenAI's
  high-risk-cyber fallback produced false downgrade warnings for successful
  Opus runs.
- Configuration: accepted aliases belong in typed catalogue/provider metadata;
  cyber policy is recognized only from an explicit typed provider response.
- Tests: `safety_check_downgrade` core integration tests.
- Removal condition: upstream represents requested, reported, and reroute
  reason as structured identities without inferring policy from model names.

## Acknowledged model selection

- Owner: PF Terminal TUI and routing maintainers.
- Difference: `/model` and provider-backed model selection persist a new
  default only after the running thread acknowledges the exact model, provider,
  and reasoning settings.
- Evidence: request acceptance precedes runtime application; persisting at the
  request boundary could make the header and next resume advertise a route that
  failed during auth, validation, or compaction.
- Configuration: none; this is a transactional state invariant.
- Tests: TUI model-selection acknowledgement and refusal regression tests.
- Removal condition: upstream exposes the same acknowledged, transactional
  provider/model selection contract.

## PF product surfaces

- Owner: PF Terminal product maintainers.
- Difference: binary/home naming, release packaging, wallet, vault, provider
  credentials, Telegram, PF slash commands, and branding.
- Evidence: released PF Terminal contracts and user data.
- Configuration: `~/.pfterminal`, provider configuration, and PF command
  surfaces.
- Tests: migration fixtures, CLI/TUI tests, packaging smoke tests.
- Removal condition: feature retirement with an explicit migration and release
  note.

## Explicitly removed global policies

The convergence branch uses upstream behavior for compaction, turn completion,
tool-loop handling, shell execution, and native agent lifecycle. It does not
retain the former global five-continuation stop, transcript-tail completion
classifier, natural-language shell-budget regexes, three-dispatch-cycle pause,
12-report durable truncation, or Claude Code identity prompt. Anthropic web
search no longer injects PF Terminal's former fixed eight-use cap.
