# PF-60-S09 verification drivers

These let the PF-60-S10 executor re-run each flow without reading source.

- **Build:** a debug build with `--features codex-core/developer-accounting,codex-tui/developer-accounting`; without
  it there is no `/cost` and no ledger. Keep `codex-code-mode-host` next to the `corbanu` binary (the image tool
  needs it).
- **Homes:** every run uses one disposable directory, `<home>`, as `HOME`'s sibling and as `CODEX_HOME`,
  `CORBANU_HOME` and `PFTERMINAL_HOME`, with `CORBANU_TEST_NO_NATIVE_KEYRING=1`. Delete it afterwards: an API-key
  login writes the key to `<home>/auth.json`.
- **Real keys (AC2, AC3):** in S10 the key reaches the product only through the owner-approved mediated path; the
  executor never reads it. Outside S10, fetch it at use time, scoped to the one command:
  `OPENAI_API_KEY="$(corbanu vault auth-helper openai-api-key)" sh -c 'printenv OPENAI_API_KEY | corbanu login --with-api-key'`.
- **Throwaway TLS material:** delete the `make_certs.sh` directory after the run; it holds the throwaway CA's key.

| File | What it is |
| --- | --- |
| `make_certs.sh <dir>` | Throwaway CA, a certificate for `auth.openai.com`/`chatgpt.com`, and a WebRTC SDP offer. Never commit `<dir>`. |
| `proxy.py <listen> <target> <host>...` | CONNECT proxy: tunnels only the named hosts' port 443 to a local TLS server; refuses the rest. Prints one JSON line per tunnel or refusal (hosts only). |
| `chatgpt_standin.py --port P --log F --cert C --key K` | ChatGPT sign-in (device code, token refresh) and Responses stand-in, from PF-84-S06's mock. Fake tokens only. `POST /_next_user {"user":U,"ttl":S}` picks who signs in next; `POST /_script {"replies":["imagegen","text"]}` makes the next Responses calls request the image tool, then answer in text. `*/images/generations` returns a 1x1 PNG with 48 input and 229 output tokens. |
| `realtime_driver.py <corbanu> <workspace> [--transport webrtc --sdp-file F] [--modality audio] [--version v3]` | One realtime session through `corbanu app-server` (stdio JSON-RPC): start, `appendSpeech` "Reply with the single word OK.", wait for the reply, stop. Prints the methods seen and the reply. |

## AC1: ChatGPT login (stand-in; Travis 2026-10-14: mock sign-in only)

```sh
make_certs.sh /tmp/s09tls
python3 chatgpt_standin.py --port 18601 --log /tmp/s09-events.jsonl --cert /tmp/s09tls/chatgpt.pem --key /tmp/s09tls/chatgpt.key &
python3 proxy.py 18600 18601 auth.openai.com chatgpt.com ab.chatgpt.com &
export HTTPS_PROXY=http://127.0.0.1:18600 CODEX_CA_CERTIFICATE=/tmp/s09tls/ca.pem
curl -s -X POST -H 'content-type: application/json' -d '{"user":"s09tester","ttl":3600}' \
  --proxy "$HTTPS_PROXY" --cacert "$CODEX_CA_CERTIFICATE" https://auth.openai.com/_next_user
corbanu login --device-auth          # signs in against the stand-in; plan "plus"
corbanu -m gpt-5.4                   # one turn, then /cost
corbanu -m gpt-mock-unlisted         # an uncatalogued model
corbanu -c service_tier='"priority"' # the priority tier
```

The client keeps its built-in routes (`https://auth.openai.com`, `https://chatgpt.com/backend-api/codex`); only the
network path is redirected. Expect every turn to read `Covered by your subscription (not billed per request)`, with no
spend and no "no price".

## AC2: image generation

- **API key (real):** sign in with the key (see "Real keys" above), then in the TUI ask for one small image with
  the image tool (the demo spec `qa/demos/specs/pf60s09-image-cost.toml` has the exact prompt), then `/cost`. Expect a `gpt-image-2` request priced at OpenAI's image rates: input × $5/M
  plus output × $30/M (2026-10-09 table), matching the token counts on its technical page.
- **ChatGPT login (stand-in):** with AC1's setup, run
  `curl -s -X POST -H 'content-type: application/json' -d '{"replies":["imagegen","text"]}' --proxy "$HTTPS_PROXY" --cacert "$CODEX_CA_CERTIFICATE" https://chatgpt.com/_script`,
  then send one message in the TUI. Expect a generated image, then `/cost` showing
  `gpt-image-2 — Covered by your subscription`. The stand-in's log shows `images` on
  `/backend-api/codex/images/generations`.

## AC3: realtime (real, API key)

Add `[features] realtime_conversation = true` to the home's `config.toml`, sign in with an API key, then:

```sh
realtime_driver.py corbanu <workspace>                                       # WebSocket, text output
realtime_driver.py corbanu <workspace> --transport webrtc --sdp-file /tmp/s09tls/offer.sdp --modality audio
```

Then read the ledger's attempts in `<home>/pfterminal_state_5.sqlite`:

```sql
select json_extract(payload,'$.thread_id'), json_extract(payload,'$.dispatched_at_ms'),
       json_extract(payload,'$.turn'), json_extract(payload,'$.model')
from draft_accounting_attempts order by 2;
```

Expected by the accounting design: each realtime session paid on an API key appears as a `realtime:` attempt under
the thread the driver prints. On 2026-10-14 the WebSocket run had none (FAIL, #824); see the [gate](../gate.md).
