# PF-60-S05 real-provider checks, 2026-10-09

## Setup

- **Binary:** debug `corbanu` with `codex-core/developer-accounting` and `codex-tui/developer-accounting`, built at `8a3c7498d0` (slice 4 head with slices 1–3). sha256 `eb012eaace478c7f0db964abcc65af0eae0552a61998217f144157b0f1824076`.
- **Environment:** each case ran in its own disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`. Keys came from the vault on the consuming command line only (`X="$(corbanu vault auth-helper <label>)" corbanu exec …`).
- **How results were read:** straight from each home's ledger (`draft_accounting_*`).
- **Recompute:** done by hand from the recorded token counts and the provider's published per-million rates.

## Results

| Case | Route and credential | Declared | Recorded | Check | Result |
| --- | --- | --- | --- | --- | --- |
| `zai`, GLM 5.2 | `api.z.ai/api/paas/v4`, API key | pay per use (built-in) | `Billed`; $1.40 / $0.26 cached / $4.40 | 12,705 in, 0 cached, 2 out: $0.0177958, equal to the recorded figure. Z.AI's published GLM-5.2 price is $1.40 / $0.26 / $4.40 | PASS |
| `zai-anthropic`, GLM 5.2 | Z.AI Anthropic route, API key | subscription (built-in) | `PlanEquivalent`, no rates; spent $0 | The request was served with this key. Whether the account holds a Coding Plan isn't visible to the client | PASS (basis) |
| custom `zai-coding` | `api.z.ai/api/coding/paas/v4`, API key | subscription (route row) | `PlanEquivalent`, spent $0 | — | PASS |
| `deepseek`, deepseek-v4-flash | `api.deepseek.com`, API key | pay per use | `Billed`, peak or off-peak by dispatch time | 09:12 UTC (peak, $0.30 / $0.006 / $1.20): $0.003687384. 10:06 UTC (off-peak, half): $0.000074988. Both equal the recorded figures and DeepSeek's published V4.1 Flash rates | PASS |
| `openrouter`, z-ai/glm-5.2 | `openrouter.ai/api/v1`, API key | pay per use | `Billed`, no catalogue rates; OpenRouter's stated charge recorded | First run: $0.0131858 stated; 9,409 in and 3 out at $1.40 / $4.40 recomputes to the same figure. A later run was served upstream at other rates; its stated charge is recorded as stated | PASS |
| custom `my-deepseek-beta` | `api.deepseek.com/beta`, API key | none | `Undeclared`, spent $0, ledger format 2 | `/cost` says "Billing basis not declared" with the next step (demo video) | PASS |
| same, with `billing = "pay_per_use"` | same | pay per use (user config) | `Billed`, source `UserConfig`, no rates, format 2 | The attempt page says "(set in your config)" | PASS |
| `exec --ephemeral` (AC9) | `zai` | — | no ledger | Exactly one `accounting.excluded="ephemeral_session"` warning in the run's log | PASS |
| Guardian review (AC8) | `zai`, `approvals_reviewer = "auto_review"`, escalated command | pay per use | 3 attempts, all under the reviewed conversation's thread; the reviewer's is turn `review:…` | Reviewer request: 4,073 in, 512 cached, 112 out, $0.00561132, equal to the recorded figure. All three day contributions sit under the reviewed conversation. The reviewer's own persisted thread has none, so nothing is counted twice | PASS |

## AC7 in the real TUI (tmux, GLM 5.2 on `zai`)

- **Ledger refuses every new attempt** (a trigger on `draft_accounting_attempts`): `/compact` finished ("Context compacted"), with exactly one "Developer accounting could not record…" warning in the turn. `/cost` afterwards lists the three recorded requests and not the refused one. **PASS.**
- **Another writer holds the database for 60 s:** the session's own state writes waited as well, so the whole turn paused until the lock was released. The turn and compaction then finished and were recorded. Exceeding the budget on accounting alone can't be isolated in a real session; `accounting_waits_out_another_process_holding_the_state_db_past_its_busy_timeout` and the injected-failure tests cover it.
- **Not run:**
  - a read-only ledger, which would also make the session's other writes fail;
  - web search and image generation, which need OpenAI credentials. They are covered by `accounting_store_failure_never_fails_search_or_image`.

## NOT VERIFIABLE (credential not in the vault): for Travis

- **Kimi Code membership key:**
  - AC5 (`/cost` shows Subscription and $0 spent);
  - the `kimi-code` part of AC10;
  - AC12's Kimi Code / Kimi Open Platform pair.
- **Claude plan login usable in a disposable home:**
  - the `claude-plan` part of AC10;
  - AC4 for pane-bridge Claude work.
- **ChatGPT login:**
  - the ChatGPT parts of AC4 (unpriced model, priority tier, image, realtime);
  - the ChatGPT part of AC10;
  - AC12's ChatGPT/API-key pair, including the realtime route row.
- **OpenAI API key:**
  - AC12's API-key side;
  - AC7 for web search and image generation.
- **Anthropic API key:** the `anthropic` pay-per-use default.
- **BigModel (`open.bigmodel.cn`) keys:** both BigModel route rows.
