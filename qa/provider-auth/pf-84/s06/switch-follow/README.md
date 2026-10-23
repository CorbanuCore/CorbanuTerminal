# Mid-session account switch: tmux check

macOS debug build of `fix/pf84-switch-follow` (developer-accounting on), private tmux socket, disposable
`HOME`/`CODEX_HOME`, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, `RUST_LOG=info`, `named_accounts = true`.

- `work` was signed in with `X="$(corbanu vault auth-helper openai-api-key)" sh -c 'printf %s "$X" | corbanu login
  --account work --with-api-key'` (installed helper, real key, never printed). GLM 5.3 Flash ran on
  `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the launch command.
- Run 1 (`tui-pane.txt`, `log-excerpts.txt`, `ledger.txt`): `-m gpt-5.4-mini -c model_provider=openai --account work`;
  turn answered via `work`; `/model` → Z.AI GLM 5.3 Flash (default account), turn answered; `/model` → OpenAI
  GPT-5.6-Luna (`work`): OpenAI answered 404 for a GLM reasoning item id in the history (see the gate).
- Run 2 (`*-run2.txt`): `-m glm-5.3-flash -c model_provider=zai`; off screen `login --account work` and a fake
  default OpenAI key; `/providers` → *OpenAI · work* → *Use for this session*; `/model` → OpenAI GPT-5.6-Luna
  (`work-ok`); `/model` → GLM (`default-ok`).
- Run 3 (`*-run3.txt`, review head `df68e14642`): `-m gpt-5.4-mini -c model_provider=openai --account work`
  (the process list follows); `work-first` → `/model` GLM → `/model` OpenAI GPT-5.6-Luna (`work-again`, no GLM
  turn in between, see #578) → `/model` GLM (`default-glm`).
- Run 4 (`*-run4.txt`, final head `9a68191d9d`): the run 3 flow again (`work-first`, `work-again`, `default-glm`).
- Each switch: core logs `account=… connectors, plugin installs, accounting and the model list follow the switch`,
  then the TUI's `model/list` request; the TUI says *Connectors, usage and the model list now use …*. The ledger
  attributes each attempt to the account that served it (`default` for GLM).
- `leak-scan*.txt`: the real OpenAI and Z.AI keys (whole, first and last 16 characters) are in none of the run
  directory's or this directory's files.
