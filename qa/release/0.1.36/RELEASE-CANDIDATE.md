# Corbanu Terminal 0.1.36 release candidate

Date: 2026-08-28 UTC
Branch: `feat/glm-5-3-flash-vast-preset`
Base: `eeb048ca68` (last 0.1.35-era commit)

Candidate commits (in order):

- `caf9ffe8dc` — bench: isolation, binary pinning, explicit reasoning, loop caps
- `5f45c11229` — fix(core): stop clamping zai chat reasoning effort to xhigh
- `0f3182ad5b` — bench: GLM 5.2 repro configs (clean, exact-route, old-protocol)
- `54d81e91ef` — refactor: stop shipping AGENTS.md; move policy to docs and config
- `a2845c9023` — feat(models): default GLM 5.3 reasoning to low; max opt-in
- `f592905d8e` — fix(gpu): keep the rental controller alive under broken config
- `6d1b18f196` — release: bump version to 0.1.36
- plus `just fmt` and benchmark-config commits

This candidate has been built and packaged locally. It has **not** been
tagged, published, promoted, or deployed.

## Change classification

| Change | Class | Product citation |
| --- | --- | --- |
| zai reasoning-effort clamp removal | Bounded fix | restores the authorized behavior that `model_reasoning_effort` selects the effort actually sent on the wire; the clamp made low/medium silently serialize as `max` on GLM 5.3's preserved-reasoning protocol |
| GLM 5.3 catalogue default low | Bounded fix | prevents unconfigured sessions from silently running maximum preserved reasoning (5-7x wall-time impact measured) |
| AGENTS.md de-shipping | Bounded fix + docs | the repository/product must not inject its own policy file into user or benchmark agent sessions; Telegram no longer seeds a default `AGENTS.md`; identity moved to `[telegram].identity_instructions` |
| GPU controller startup hardening | Bounded fix | the TTL/spend-enforcement controller silently exited under an invalid global config, leaving authorized rentals unenforced |
| Benchmark harness isolation overhaul | Routine (harness) | see `MODEL_EVAL_FINDINGS_2026-08-28.md` |

## Source gates

| Gate | Result |
| --- | --- |
| `python3 -m unittest discover -s benchmarks/coding/tests` | 10/10 pass |
| `cargo test -p codex-models-manager` | 61/61 pass |
| `cargo test -p codex-telegram --lib` | 89/89 pass |
| `cargo test -p codex-core --lib client` | pass after intentional default-low update |
| `cargo test -p codex-core --lib config::` | 2 pre-existing failures unrelated to this candidate (`project_layers_disabled_when_untrusted_or_unknown`, `codex_home_is_not_loaded_as_project_layer_from_home_dir`); they fail identically on the base commit on this host |
| `just fmt` | clean after style commit |
| `python3 scripts/check_no_shipped_agents_md.py` | pass (new CI gate) |
| `python3 scripts/check_portable_skills.py` | 25 files match |
| Release build `cargo build --release` (corbanu, corbanu-acp, corbanu-walletd, codex-code-mode-host) | pass, `corbanu 0.1.36` |
| Package archive (`scripts/build_codex_package.py`) | `dist/x86_64-unknown-linux-gnu/corbanu-terminal-package-x86_64-unknown-linux-gnu.tar.gz` (125M); **verified to contain zero AGENTS.md content** |
| `just build-for-release` (Bazel) | fails on this host with a pre-existing rules_rs toolchain configuration error (`target_triple` selection); unrelated to candidate changes; CI release workflow is cargo-based |

## Benchmark evidence (clean harness, this candidate's binary)

GLM 5.3 Flash, explicit reasoning verified on the wire per run, isolated
homes/sandbox, run root outside the repository
(`/home/pfrpc/bench-runs/glm53-flash-clean-full-20260828`):

| Route | Task | Effort | Wall | Official result |
| --- | --- | --- | ---: | --- |
| Z.AI plan | EventForge | low | 276.8s | fail (hidden 2 miss) |
| Z.AI plan | EventForge | max | 1,249.9s | **pass** |
| Z.AI plan | LogTriage | low | 93.2s | fail |
| Z.AI plan | LogTriage | max | 1,800s | timeout |
| Z.AI plan | QueueCraft | low | 66.9s | fail |
| Z.AI plan | QueueCraft | max | 322.3s | **pass** |
| OpenRouter | EventForge | low | 655.4s | **pass** |
| OpenRouter | LogTriage | low | 81.8s | fail |
| OpenRouter | QueueCraft | low | 73.9s | fail |

Interpretation: low effort is 4-27x faster and passes visible tests but misses
hidden edge-case contracts most of the time; max effort is the correctness
setting. This is a coherent speed/accuracy dial, not a regression: the
2026-08-28 investigation (`MODEL_EVAL_FINDINGS_2026-08-28.md`) showed the
previous "GLM 5.3 is 13-19x worse" numbers came from the zai effort clamp,
harness contamination, and a historically leaky comparison protocol.

B300 rental comparison: see `/home/pfrpc/bench-runs/B300_VIABILITY_ANALYSIS_20260828.md`.
Verdict: not viable for benchmarking or normal agent traffic (6-70x API cost
per output token; provider host reliability issues); qualified recipe retained
for fleet-concurrency or privacy use. Re-run of agentic B300 lanes is blocked
on Vast funding (balance -$0.28; ~$65 needed).

## Hard release gates still open (per benchmarks/README.md and docs/development-policy.md)

1. **Qualifying benchmark cycle (bootstrap):** three-way Corbanu/Hermes/Kilo
   live-repository component and the full all-tasks coding matrix across the
   frozen relevant model set are not run in this candidate. Blockers recorded:
   OpenRouter primary key at limit (funded alternates identified on-host),
   Hermes has no configured providers, Vast unfunded.
2. **True-TUI interactive QA with keys sent** for affected interactive flows
   (/gpu controller messaging, /model reasoning defaults).
3. **Named human tester sign-off.**

Per policy these gates block publication, not development. The candidate is
ready for the qualifying cycle once credentials/funding are restored.
