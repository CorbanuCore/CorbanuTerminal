# #368: real Moonshot requests (2026-10-15)

PF-60-S07 priced Moonshot's Kimi Open Platform (`https://api.moonshot.ai/v1`) by route, checked only against a stand-in.
This is the first check with a real key (vault label `kimi_api3`). **The rates were right, but `/cost` under-reported
both models.** Moonshot leaves out cache counts that are zero, so `/cost` showed only a lower bound. Fixed in #859.

## Run

- **Setup:** a debug build of `origin/main` `7666bdf3d4` with `developer-accounting`. Disposable home,
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`, and a user-configured provider `kimi` (`wire_api = "chat"`,
  `env_key = "MOONSHOT_API_KEY"`). The key was fetched by `corbanu vault auth-helper` only inside the launch command.
- **Requests:** one turn per model ("Reply with the single word OK."): `kimi-k3` at 21:42Z and `kimi-k2.7-code` at
  21:43Z.
- **Logging:** `RUST_LOG=codex_api::endpoint::chat_completions=trace` logged the raw SSE events. The final usage chunks
  are [k3-final-usage-chunk.json](k3-final-usage-chunk.json) and [k2-final-usage-chunk.json](k2-final-usage-chunk.json).
- **Rates:** platform.kimi.ai/docs/pricing/chat, [re-read at ~21:31Z](../data/moonshot-pricing-20261015.txt), is
  identical to the [2026-10-14 reading](../data/moonshot-pricing-20261014.txt).
- **Models:** `/v1/models` on the key lists `kimi-k2.7-code`, `kimi-k3`, `kimi-k2.6` and `kimi-k2.7-code-highspeed`,
  the four priced rows ([readings](models-and-balance.txt)).

## What Moonshot reported, and what `/cost` showed

| | `kimi-k3` | `kimi-k2.7-code` |
| --- | --- | --- |
| Usage reported | 11,984 prompt, 45 completion (29 reasoning). `prompt_tokens_details` has only `cache_write_tokens: 11776`, with no `cached_tokens` | 11,561 prompt, 11 completion (8 reasoning). No `prompt_tokens_details` and no `cached_tokens` |
| Rates bound | $3 / $0.30 / $3 (5 min) / $15, correct | $0.95 / $0.19 / $0.95 / $4, correct |
| `/cost` on main ([K3](real-before/k3-technical.txt), [K2](real-before/k2-technical.txt)) | **at least $0.036003**: the uncached input and the cache read were "unknown" | **at least $0.000044**: the output only |
| Correct ([Moonshot's rule](../data/moonshot-chat-usage-20261014.txt): read + write + uncached = prompt) | **$0.036627**: 208 uncached + 11,776 written at $3, plus 45 at $15 | **$0.01102695**: 11,561 at $0.95, plus 11 at $4 |
| `/cost` with #859 ([K3](standin-after/k3-fix-technical.txt), [K2](standin-after/k2-fix-technical.txt)) | $0.036627 | $0.011027 (rounded); exact USD 0.01102695 |

- **The schema doesn't match the responses.** Moonshot's schema lists `cached_tokens` and `cache_write_tokens` as
  required ([reading](../data/moonshot-chat-usage-schema-20261015.txt)), but the real responses leave a count out when
  it is zero.
- **What #859 changes:** on the Moonshot route, with `prompt_tokens` stated, a missing count is read as zero. A read
  missing from the details is taken from `usage.cached_tokens`, where Moonshot also states it.
- **Not observed live:** a cache hit. These were first requests, so neither cached anything. A K2 hit stated only as
  `usage.cached_tokens` is covered by a test.
- **K3 caching:** K3 wrote 11,776 of 11,984 prompt tokens to the cache unprompted. At the 5-minute rate that costs the
  same as uncached input.
- **No `prompt_cache_options`:** none of the requests sent it, as S07 found.

## After the fix: stand-in replay

There was no third real request. To show the fix end to end, [standin/standin.py](standin/standin.py) replays the exact
real usage chunks.

- **How it works:** it is a local CONNECT proxy (127.0.0.1 only; it refuses any other host). It presents a throwaway-CA
  certificate for `api.moonshot.ai` and answers with the usage chunk file it is given. The client reaches it through
  `HTTPS_PROXY=http://127.0.0.1:<port>` and `CODEX_CA_CERTIFICATE=<dir>/ca.pem`. Make the certificates next to the
  script (they are git-ignored):

  ```sh
  openssl req -x509 -newkey rsa:2048 -nodes -keyout ca.key -out ca.pem -days 2 -subj "/CN=throwaway CA" \
    -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign"
  openssl req -newkey rsa:2048 -nodes -keyout leaf.key -out leaf.csr -subj "/CN=api.moonshot.ai"
  printf 'subjectAltName=DNS:api.moonshot.ai\nextendedKeyUsage=serverAuth\n' > ext.cnf
  openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out leaf.pem -days 2 -extfile ext.cnf
  python3 standin.py 18461 ../k3-final-usage-chunk.json
  ```
- **Check of the stand-in:** the main build replayed through it reproduces the real `/cost` exactly (at least $0.036003
  and at least $0.000044; [K3](standin-before/k3-main-first-screen.txt), [K2](standin-before/k2-main-first-screen.txt)).
- **The fix:** the #859 build (first head `e10b8b45b0`; the review follow-up changed nothing these chunks reach) shows
  the correct figures above
  ([first screens](standin-after/k3-fix-first-screen.txt), [requests](standin-after/k3-fix-request.txt)).
- **Logged requests:** every request the stand-in [logged](standin/requests.jsonl) was `stream: true` with
  `stream_options.include_usage = true`, and none carried `prompt_cache_options`.

## Checks

- **Key scan:** `kimi_api3` was not found in this folder, the run homes, the logs or the PR diffs (counts only; see #368).
- **Moonshot's balance:** `/v1/users/me/balance` read 25 (whole USD) before and after the run
  ([readings](models-and-balance.txt)), too coarse to confirm a $0.048 charge. Moonshot's bill is the remaining check.
- **Spend:** two real turns, $0.047654 at Moonshot's rates.
