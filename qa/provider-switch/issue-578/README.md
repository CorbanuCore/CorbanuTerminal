# #578: tmux check with real providers

These are macOS debug builds in a private tmux socket. Each run used a disposable `CODEX_HOME`/`HOME` and
`CORBANU_TEST_NO_NATIVE_KEYRING=1`.

**Providers:** OpenAI and Z.AI. Both keys were resolved on the launch command, the OpenAI key as
`OPENAI_API_KEY="$(corbanu vault auth-helper openai-api-key)"` and the Z.AI key as
`ZAI_API_KEY="$(… provider/zai_api_key)"`. Both used the installed helper, and neither key was printed.

**Steps** (the issue's): `-m gpt-5.4-mini -c model_provider=openai`.
1. One turn on OpenAI.
2. `/model` → Z.AI GLM 5.3 Flash (low), then one turn.
3. `/model` → OpenAI GPT-5.6-Luna (low), then two turns.

| Run | Build | Result |
| --- | --- | --- |
| [`tui-pane-before.txt`](tui-pane-before.txt) | `main` (`a6f30dad0f`, plus the unrelated #586 change) | Both later OpenAI turns fail with `404 … Item with id 'rs_202610121745…' not found … store is set to false` |
| [`tui-pane-after-manual.txt`](tui-pane-after-manual.txt) | `fix/578-cross-provider-item-refs` at `72d4f573b1` | `openai-three` and `openai-four` are answered. A further switch back to GLM (`glm-five`) is answered too. |
| [`tui-pane-after-video.txt`](tui-pane-after-video.txt) | final code head `b40f72784f` (the SOP video; earlier takes are in the index) | `openai-three` and `openai-four` are answered |

- **Rollout:** in the manual run after the fix, every model output item records its producer (`openai` or `zai`).
- **Recovery path:** no request needed the 404 retry; the log has no "not found".
- **Video:** [issue-578-pf578-provider-switch-history-b40f72784fae-2026-10-12.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-578-pf578-provider-switch-history-b40f72784fae-2026-10-12.mp4),
  spec `qa/demos/specs/pf578-provider-switch-history.toml`.
- **Leak scans:** the real OpenAI and Z.AI keys were searched for (whole key, first and last 16 characters) in every run directory and in the
  video run directory. Each scan found 0 hits.
