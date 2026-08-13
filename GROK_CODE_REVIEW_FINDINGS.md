# Grok Code Review Findings

## Verdict

**Do not ship** as-is.

The branch has real, valuable isolation and provider work (home isolation, Meta item IDs, Vercel server-state continuation, provider-key 401 gating), but the Chat Completions vision repair for tool results still mishandles the highest-frequency OpenRouter path: `view_image` results are serialized as a full base64 JSON dump in the `tool` message **and** again as a `user` multicol content image. That is a correctness and cost bug on the models the branch specifically promotes for vision (OpenRouter Grok/MiniMax/Gemini variants). Parallel tool + image mixtures can also emit illegal Chat Completions orderings. Those defects sit on the newly claimed vision surface and should block promotion.

## Coverage

### Range

- Repo: `/home/pfrpc/repos/PfTerminal-triage-clean`
- Branch: `integrate/pfterminal-20260707` @ `43d33461e`
- Baseline: `origin/main`
- Stats: **29 commits**, **146 files**, **+10473 / -1061** lines; `git diff --check` clean

### Commits reviewed (line-level for latest critical)

- `43d33461e` Keep provider API keys out of ChatGPT refresh
- `483680f06` Preserve Meta response item IDs
- `bc3aa019b` Fix vision input across model providers
- `8db470c1a` Use function edit tools for Meta
- `611b455da` / `c4b0b05d0` Meta + OpenRouter model catalog additions (via tree + models.json)
- `a0dc82140` PFTerminal home isolation + GPT-5.6 models
- `cbc34afd3` Vercel server-state user-message guard + 400 self-heal
- `17e6e1b91` dispatch-loss recovery / pending spawn dispatches
- `cbf00bcaa` orchestrate pane naming UX
- Installer / package isolation (`scripts/install/*`, `scripts/codex_package/targets.py`, `codex-cli/package.json`)
- Orchestration surface size audit (`orchestrate.rs` 2416 LOC, `spawn_orchestration.rs` large)

### Subsystems / clusters

| Cluster                           | Evidence                                                                                             |
| --------------------------------- | ---------------------------------------------------------------------------------------------------- |
| A Home / DB / installers          | `home-dir`, `state/runtime` migrate+foreign-checksum, install.sh/ps1 remove `codex`, package targets |
| B Model picker / catalog          | `model_popups.rs` provider tabs, models.json, manager_tests image-capability list                    |
| C Auth / 401                      | `ModelClient::unauthorized_recovery` env_key gate + unit test                                        |
| D Serialization / vision          | Chat+Anthropic request builders; multipath structural inspection of tool+image continuation          |
| E Session / resume / server state | Meta ID assignment paths; vercel_server_state integration tests                                      |
| F Orchestration / roles           | spawn crew builders, dispatch on busy, Nazgul→Troll prompt, module size                              |

### Commands actually run

```text
git status --short --branch
git log --oneline --decorate origin/main..HEAD
git diff --stat/--name-status/--check origin/main...HEAD
rg searches from Phase 5
just test -p codex-protocol response_item_assigns  → 1 passed
just test -p codex-core meta_repairs provider_api_keys chat_replay_preserves anthropic_messages_request_preserves → 4 passed
just test -p codex-utils-home-dir → 5 passed
```

Skipped / not completed: full `just test -p codex-tui`, full workspace suite, `codex-model-provider` package run (still compiling/long), live API probes (no dispatcher authorization for paid network, vault not used).

### Live probes

Not run (credentials not consumed; plan Phase 4 unauthorized without explicit spend approval).

## Findings

### 1. P0 — Chat Completions `view_image` tool results dump full base64 into the tool message and re-send the same image as a synthetic user turn

**Title:** OpenRouter Chat path double-emits vision tool results (JSON dump + user image parts)

**Where:**

- `codex-rs/core/src/client.rs:2965-2978` (`append_chat_messages_for_response_item` for Function/Custom tool outputs)
- `codex-rs/core/src/tools/handlers/view_image.rs:247-258` (image-only ContentItems payload)
- `codex-rs/protocol/src/models.rs:1764-1785` + `2074-2082` (`to_text` drops images; `Display` emits full JSON of content items)
- Triggered for every Chat-wire provider, including OpenRouter (`model-provider-info` `create_openrouter_provider` → `WireApi::Chat`)

**Trigger / repro:**

1. Select OpenRouter provider + vision model (`x-ai/grok-4.5`, `minimax/minimax-m3`, or Gemini Flash on OpenRouter).
2. Turn that issues `view_image` on a local PNG/JPEG.
3. Observe the next Chat Completions request built for the continuation (or dump via `PFTERMINAL_DUMP_CHAT_REQUEST`).

**Actual:**

1. `ViewImageOutput` records `FunctionCallOutputBody::ContentItems([InputImage{data URL, detail}])` only — no text item.
2. Chat serializer does `output.body.to_text().unwrap_or_else(|| output.to_string())`.
3. `to_text` is `None` for image-only payloads, so `Display` serializes the content items as a JSON string that **includes the entire base64 data URL**.
4. That string is placed in `role=tool` `content`.
5. Separately, `chat_tool_result_image_parts` builds a second `role=user` message with the same image as `image_url` content parts.

Request shape (structurally enforced by code):

```json
[
  {"role":"assistant","tool_calls":[{"id":"call_image","function":{"name":"view_image",...}}]},
  {"role":"tool","tool_call_id":"call_image",
   "content":"[{\"type\":\"input_image\",\"image_url\":\"data:image/png;base64,...FULL_BYTES...\",\"detail\":\"high\"}]"},
  {"role":"user","content":[{"type":"image_url","image_url":{"url":"data:image/png;base64,...FULL_BYTES...","detail":"high"}}]}
]
```

**Expected:**

- Tool message carries only a short, non-image textual acknowledgment (or structured multimodal tool content if the provider supports it).
- Image bytes appear **once**, in a provider-valid content shape.
- Context window should not grow by ~2× image payload per `view_image` call.

**Why tests miss it:**

- `chat_replay_preserves_user_and_tool_result_images` only asserts:
  - the **user** image part
  - that message index 2 is `tool` and index 3 is `user`
  - it does **not** assert tool content is free of base64 or is short text
- Fixture uses mixed text+image tool output (`"loaded"` + image), so `to_text()` succeeds and hides the image-only path always taken by real `view_image`.
- Anthropic unit test covers Anthropic path, which correctly embeds image blocks inside `tool_result` content (different code path).

**Repair direction:**
In Chat mapping for `FunctionCallOutput` / `CustomToolCallOutput`:

1. Prefer `to_text()` for the tool role message text; if `None`, use a fixed short placeholder such as `"(image)"` or empty string — never `Display` JSON of content items.
2. Prefer attaching images only via `user` parts **or** only via native multimodal tool content if/when the wire format supports arrays, not both silently.
3. Unit-test an **image-only** `view_image`-shaped payload and assert tool content contains no `data:image`.

---

### 2. P1 — Parallel tool bag with an image result can produce illegal Chat Completions ordering (`tool` after a synthetic `user`)

**Title:** Image tool-result injection breaks assistant → tool\* → (next assistant) ordering under parallel tools

**Where:**

- `codex-rs/core/src/client.rs:2971-2978` (inserts synthetic `user` immediately after each image-bearing tool result)
- `codex-rs/core/src/tools/handlers/view_image.rs:79-81` (`supports_parallel_tool_calls = true`)
- OpenRouter Chat wire + models advertising `supports_parallel_tool_calls: true` (e.g. MiniMax M3 in models.json)

**Trigger / repro:**
Model issues one turn with concurrent tools, e.g. `view_image` + `exec_command`/`shell` (parallel allowed). History order is commonly:
`FunctionCall(view_image)`, `FunctionCall(shell)`, `FunctionCallOutput(view_image images)`, `FunctionCallOutput(shell text)`.

Serialized Chat messages become:

1. assistant with tool_calls (possibly one-at-a-time per FC — still sequential assistants if unbatched)
2. tool (image call, polluted content from Finding 1)
3. **user** (synthetic image parts)
4. tool (shell result)

Many strict OpenAI-compatible validators reject a `tool` message that is not immediately after an assistant message containing matching `tool_calls`.

**Actual:** Continual turns with mixed parallel tools after `view_image` can 400 at the provider or confuse tool_call matching.
**Expected:** After an assistant tool_calls bundle, only tool responses for those call IDs appear before the next user/assistant turn; images must not split the tool-result block with a user message.

**Why tests miss it:** Unit test serializes a single function call + one image result only. No multi-tool interleaving fixture for Chat vision.

**Repair direction:**
Buffer image parts until **all** tool results for the current assistant tool bag are emitted, then optionally append a single synthetic user multimodal message — or, cleaner, put multimodal content only in tool messages if the API allows, or record image as a non-splitting user attachment at the end of the turn before the next model call only when no pending tool results remain.

---

### 3. P1 — npm / CLI surface still publishes a `codex` entrypoint while install/package isolation claims PFTerminal must not ship `codex`

**Title:** Isolation contract incomplete on the published npm launcher

**Where:**

- `codex-cli/package.json:6-9` — `"bin": { "pfterminal": "bin/codex.js", "codex": "bin/codex.js" }`
- Contrast with deliberate isolation in this branch:
  - `scripts/codex_package/targets.py:60-65` removed extra `codex` binary on the **pfterminal** package variant
  - `scripts/install/install.sh` and `install.ps1` delete staged/stale `bin/codex` artifacts and only expose `pfterminal`

**Trigger:**

1. Install `@agticorp/pfterminal` via npm/pnpm (or keep existing package that creates a `codex` shim).
2. Invoke `codex` on PATH (or have an older global npm `codex` also present).
3. Launcher still defaults `CODEX_HOME` to `~/.pfterminal` (good) **but** the public command name collides with stock OpenAI Codex and with residual packages already present on machines (QA evidence still records `/usr/bin/codex` and npm-global `codex`).

**Actual:** Dual-branding via `bin.codex` remains; contradicts the installer policy hard-reset away from shipping `codex`.
**Expected:** PFTerminal packages/installers never install or advertise a `codex` command that users can mistake for stock Codex (plan invariant).

**Why tests miss it:** Unit tests cover argv0-independent `.pfterminal` home and installer path removal; nothing fails CI on npm `package.json` bin map.

**Repair direction:**
Remove `"codex"` from `codex-cli/package.json` `bin` (and any residual docs that still instruct `npm i … && codex`). Keep only `pfterminal`. If compatibility is required, gate behind an explicit opt-in package, not the default.

---

### 4. P2 — Anthropic image source parsing is fragile for non-`data:*;base64,` encodings

**Title:** Anthropic image builder degrades malformed/odd data URLs to a remote URL source

**Where:** `codex-rs/core/src/client.rs:3076-3091`

**Trigger:** Data URLs such as `data:image/png;base64;foo` (wrong separator), `data:image/png,BASE64` (no `;base64`), or unexpected parameter ordering.

**Actual:** Fallback is `{ "type":"url", "url": "<entire data: string>" }`, which Anthropic will not fetch as a remote URL and may hard-fail the request.
**Expected:** Reject with a clear serialization/preparation error, or normalize known variants; never send a giant `data:` string as a URL source.

**Why tests miss it:** Anthropic vision unit test only uses well-formed `data:image/png;base64,...` and `https://...` URLs.

**Repair direction:**
If base64 parse fails and the URL is not `http(s)`, return an error (or drop the image with a trace) instead of constructing a URL source.

---

### 5. P2 — `.codex` refusal is name-based only; diverted CODEX_HOME using upstream-shaped files elsewhere is not identically hard-blocked by the basename check

**Title:** Upstream DB rename guard keys off directory basename `.codex`, not ownership markers alone

**Where:** `codex-rs/state/src/runtime.rs:376-388` (`migrate_legacy_runtime_db_names`)

**Trigger:** `CODEX_HOME` / `PFTERMINAL_HOME` points at a directory that is **not** literally named `.codex` but contains stock Codex `state_5.sqlite` with foreign migrations (less common; usually users set `~/.codex` which is covered). Basename guard refuses rename. Foreign migration validation (`validate_applied_migrations`) still rejects migration with “different distribution” for unknown checksums, so write risk is limited. Residual hole: legacy PFTerminal-named files co-located inside an unusual path could be renamed even if another product placed them there.

**Actual / Expected:** Safety mainly relies on foreign checksum validation after basename intercept; basename intercept is incomplete as a sole ownership heuristic.

**Why tests miss it:** Tests cover `.codex` basename + foreign migration reject + happy rename in temp dirs, not other explicit upstream home layouts.

**Repair direction:** Prefer refusing **any** open when foreign migrations appear **before** rename, which already mostly holds; optionally also refuse rename when `AUTH.json`/upstream markers exist. Keep namespaced filenames (good).

## Test Gaps

1. **Image-only `view_image` Chat request fixture** — assert tool `content` has no `data:image` and image appears once.
2. **Parallel `view_image` + shell Chat ordering** — assert no synthetic `user` between tool results.
3. **`provider_api_keys_do_not_fall_back_to_chatgpt_unauthorized_recovery` is shape-only** — does not shoot a 401 through the HTTP loop to prove recovery is skipped under stream_chat / stream_responses paths (logic now shared via `unauthorized_recovery()`, good, but end-to-end mock 401 desired).
4. **Meta ID stability across re-request of the same history without re-recording** — `assign_id_if_missing` is UUID-v7; Memory path records on append so integration test covers continuations, but pure retry of a locally mutated unpersisted buffer could re-mint IDs for any still-missing items.
5. **npm package isolation test** — assert published bin stems for pfterminal package do not include `codex` (mirrors install.sh).
6. **Bundled image capability list** advertises and tests image for MiniMax / Claude / Gemini but does **not** assert `x-ai/grok-4.5` in either the allow or deny list despite the model being vision-tagged in models.json — catalog/assertions are inconsistent.
7. **Vercel suite** uses `skip_if_no_network!` so offline CI skips the most important continuation regressions unless network is on.

## Clean Areas

1. **Home default isolation (binary argv0)** — `find_codex_home` no longer keys on executable stem; PFTerminal builds default `~/.pfterminal`; `PFTERMINAL_HOME` beats `CODEX_HOME`. Unit tests pass (`codex-utils-home-dir` 5/5).
2. **Namespaced SQLite files + foreign migration refusal** — `pfterminal_*` filenames; refuses opening foreign SQLX migrations without rewriting the DB; refuses rename inside a directory basenamed `.codex`. Code+tests in `state/runtime.rs`.
3. **Standalone installer isolation** — Unix/Windows install scripts stop shipping/completing on `bin/codex`, delete stale package `codex` artifacts, verify `pfterminal`.
4. **Provider-key 401 recovery gate** — env_key providers skip ChatGPT `unauthorized_recovery`; OpenAI-shaped provider still retains recovery. Unit test `provider_api_keys_do_not_fall_back_to_chatgpt_unauthorized_recovery` passes.
5. **Meta item IDs** — `assign_id_if_missing` in protocol; Meta forces IDs in client prepare + session history path even without Feature::ItemIds; preserves server /already-assigned IDs; integration test `meta_responses_assigns_ids_across_tool_continuations`; unit test for assignment.
6. **Meta edit tools** — structured_edit/write as Function tools instead of custom apply_patch; unit coverage.
7. **Anthropic direct attachments + tool_result images** (non-chat) — structured multiproove unit test; correct base64/url source for well-formed inputs.
8. **Vercel previous_response_id continuation** — skips incremental when no user message; 400 self-heal clears server state once; suite documents the invariant (network-gated).
9. **Model picker provider grouping** — tabs for OpenAI/Ambient/Z.AI/Claude Plan/Anthropic/Meta/Vercel/Baseten/OpenRouter with slug→provider mapping helpers and snapshot coverage for GPT-5.6.
10. **Secrets scrypt compatibility commit** + dispatch-loss pending queue plumbing appear coherent from review of interfaces; no production secret leakage found in reviewed logs/error paths (tool output log preview for view_image omits bytes).

## Residual Risk

- **No live API verification**: Grok/MiniMax/Meta/Anthropic vision and 401 behavior not exercised against real endpoints or vault material.
- **Orchestration**: `orchestrate.rs` alone is 2.4k lines with state split across App maps, Claude panes, spawn parent maps, and pending dispatch queues. Failure/cleanup ownership is not proven by a single Drop/cleanup owner in that module; rename/status UX inherits large recent surface area. Not turned into a ship-blocking defect without a concrete leak repro, but operational risk remains high.
- **codex-tui full suite not run here** (plan allows targeted tests only); snapshot churn for model picker/grouping may still hide selection edge cases.
- **Resume of pre-branch rollouts with legacy image shapes** under OpenRouter Chat specifically inherits Finding 1 on every replay of historical `view_image` outputs.
- **Remote model catalog overlays** can still change modalities at runtime; only bundled models.json was asserted.
- **Windows path isolation** reviewed via `install.ps1` only; not executed on Windows.
- Some longer nextest invocations for larger crates were not finished in this review window.

---

_Reviewer note: Treat this as adversarial verification, nottest-green endorsement. Passing unit selections above confirm the happy-path guards stakeable for those crates; they do not clear Findings 1–3._
