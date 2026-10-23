All three PRs are merged into main with all checks green, and each passed its gate (tests, GLM 5.2 tmux video, Opus 5.5 High review, Linux clippy on the RTX box). The PF-27-S05 work is a first slice, not the whole sprint: several Core paths still send provider keys directly with the new flag on.

**What shipped**
1. **#224 (`6d55f6ae8f`): the URL policy now survives a permission-profile change.**
   - Both config paths now share one helper that sets all the flag-controlled proxy protections, so they can't drift apart again.
   - The new regression test switches profile mid-session and checks that the URL policy and the response gate stay on. It fails on old main and passes now.
2. **#225 (`8cf46179f5`): requests that go through the credential broker are now pinned.** Applies only with `url_destination_policy` on.
   - Core passes the DNS answers it already checked inside the broker's signed request, so they can't be altered on the way.
   - The broker connects only to those addresses and never looks the name up again. A request with no answers, or answers for a different host, is refused by both Core and the broker.
   - Behaviour change: a pinned request is now held to its pin even with the URL policy off.
   - Recorded in PF-28-S02's record, which owns it; PF-33-S02's open item now points there.
   - Tests use names that can't resolve, so reaching the server proves the pinned address was used.
   - Opus review: approved; its small suggestions were applied.
   - Video `pf28s02-brokered-pinned`: the request returns 200 with the credential redacted, and Core's log shows the request was pinned to 8 checked answers.
3. **#229 (`c5bdadd322`): PF-27-S05 slice 1, behind the new `broker_model_auth` flag (off by default).**
   - When a provider's auth is one plain API key, Core hands it to a contained broker process and keeps only an opaque reference.
   - The key can come from the environment, the vault or stored provider keys, an OpenAI API-key login, or `experimental_bearer_token`.
   - Every request the main model client makes goes over the broker's private socket, each with a one-time signed authorization. That covers Responses, Chat, Anthropic, compaction, memories and the realtime call.
   - Core never sends the key itself as a fallback. A dead or failed broker, a provider URL the broker can't handle, or a non-Unix platform each give an immediate, non-retried error.
   - Websockets are off when the flag is on.
   - Opus review 1 asked for changes. I fixed the main one: a key the broker can't hold now fails instead of being sent directly. The other findings are recorded as open work (below). Review 2 approved.
   - Videos: one shows a real Z.AI session answered through the broker; the other kills the broker mid-session and the next prompt fails at once with a fatal error.

**Gate evidence**
- **Local tests:** 577 passed across network-proxy, secret-broker, model-provider, http-client and features; 910 passed in the affected Core test groups.
- **Linux:** clippy with warnings as errors is clean on all changed crates, and the new tests pass.
- **Records:**
  - Reviews and evidence: `qa/security-levels/sprints/PF-27-S05/` and `PF-28-S02/review-opus-4-pinning.md`.
  - Video indexes: `qa/demos/index/PF-27-S05.md` and `PF-28-S02.md`.
  - Short result file: `.codex-work/workers-20261002/sec-broker5.result.md`.

**Still open (PF-27-S05 Remaining)**
- With the flag on, these paths still send the provider key directly:
  - web search;
  - image generation;
  - the model catalog refresh;
  - the realtime conversation websocket;
  - other places that build auth from the OpenAI login.
- Vault-stored keys are still decrypted by Core rather than inside the broker, and env keys are not yet removed from Core's environment.
- ChatGPT sign-in is not brokered.
- Still missing:
  - a check that the raw key is no longer in Core's memory;
  - a test of the non-Unix branch (Windows is PF-27-S06).
- The PF-27-S05 record stays `draft`: the sprint checker won't accept `in_progress` until the plan worker records this worktree in the plan.

**Things you should know**
- The external drive hit 100% full mid-session. I removed 15 rebuildable build-output folders under `.codex-work/targets` belonging to sprints that are already merged. They will need a fresh build if anyone reuses them.
- One live check accidentally ran against my real Corbanu home (an environment variable overrode the test one). It added one non-interactive session's files there and changed no config.