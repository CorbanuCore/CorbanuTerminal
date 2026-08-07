# GPU rental release qualification — 2026-07-16

> Historical evidence only. Any pre-0731 DeepSeek recipe mentioned below is
> retired and is not launchable in PF Terminal 0.1.27. The sole selectable
> DeepSeek rental is `deepseek-flash-0731-2xh200` for
> `deepseek-ai/DeepSeek-V4-Flash-0731`.

Status: **PASS — live qualification complete; release may proceed with stated limitations**

Candidate:

- Branch: `feat/gpu-market-v2`
- Release-candidate commit: `77082da56c911b82569461d74eab4fd91a3a87b1`
- Release-candidate debug binary SHA-256: `c969cb880ac55a8dff1facffa9bc020fea39f8ed1f8c8499e71ea75a7571ce47`
- Live TUI client commit/hash: `eb93301b2` / `95a7722083dba44d5823736fa4ec9bd2a58ff3dcf970f909c9b8744053d2c3df`; release-candidate commit `77082da56` adds only the pre-download CUDA-device gate described under rental 5.
- Recipe: `huihui-glm-5.2-iq1m-2xh200-experimental`
- Release recipe revision: `huihui-glm-5.2-iq1m-llamacpp-2xh200-r4`; live serving recipe was r3 and the exact r4 device gate was executed successfully against rental 6 before model load.
- Advertised maximum context: 300,000 tokens

## Automated qualification

- `just test -p codex-gpu-market`: PASS, 71/71.
- `just test -p codex-tui`: 3,263/3,269 PASS. Six reproducible failures are outside the GPU credential patch and are not represented as green.
- Transient endpoint credential regression before provider create: PASS; zero creates on the failed read, exactly one create after retry.
- Transient endpoint credential regression during authenticated readiness: PASS; live provider resource preserved, no termination, readiness succeeds after retry.
- Controller-owned endpoint refresh regression: PASS; a replacement SSH endpoint advances from the durable runtime-catalog sequence even after repeated health publications.
- Active-session provider refresh regression: PASS; reselecting an already-loaded rental replaces its stale base URL and rebuilds the model client.
- `just test -p codex-state`: PASS, 173/173.
- Focused active-session provider refresh test: PASS, 1/1.
- Full `codex-core` attempt: NOT GREEN and not represented as green. It reached 2,719/2,937 before being interrupted after producing an unbounded failure transcript: 2,628 passed (8 flaky), 91 failed, 14 skipped, 218 not run. Failures included missing test support binaries and unrelated hierarchy/shell/timing assertions; the touched active-session regression passed in that run. This existing suite condition must remain visible in release disposition.

## Live rental 1

- Start: 2026-07-16T02:58:52Z.
- Rental: `gpu-6a5f638c-3a8f-48b5-99a8-822e23ecfdf2`.
- Vast resource: `45046561` (US, 2× H200, $8.2950/hour offer).
- Authorization: $10/hour maximum, $10 total maximum, 90-minute TTL.
- Provider create operations: exactly one, succeeded.
- Provider bootstrap step: succeeded on attempt 1.
- Readiness: PASS on attempt 1; authenticated chat, streaming, cancellation and tool-call probes all passed.
- Long-context run: 13 completed incremental rounds reached 132,798 retained server tokens with zero truncation. Round 14 reached 138,837 retained tokens before an SSH-forward loss exposed stale runtime endpoint publication.
- Remediation: commits `5392ae211` and `eb93301b2`; exact candidate was rebuilt and the controller, all local SSH transports, and the TUI were restarted while the paid provider resource remained live.
- Recovery observation: after the restart, the durable rental endpoint and runtime provider endpoint both changed to the new controller-owned loopback forward. The same conversation was resumed and the recovered runtime processed more than 139,264 tokens through the new endpoint, past the original transport-failure point.
- Spending-cap result: the original $10 authorization ended the recovered turn before its answer. The TUI rejected the now-dead endpoint, provider termination was confirmed, and durable desired/observed state reached `terminated_confirmed`. Final estimated accrued cost was $10.0536, a $0.0536 reconciliation-poll overshoot; this is not represented as exact-to-the-cent enforcement.

## Live rental 2

- Start: 2026-07-16T03:03:47Z.
- Rental: `gpu-f63cd586-5622-40bf-97cd-5c18353feb2e`.
- Vast resource: `45046893` (Japan, 2× H200, $7.7090/hour offer).
- Authorization: $10/hour maximum, $10 total maximum, 90-minute TTL.
- Purpose: concurrent-controller and independent-token qualification using the 300K GLM r3 recipe.
- Provider create operations: exactly one, succeeded.
- Readiness: PASS on attempt 1; registered with a 300,000-token maximum.
- Multi-process run: PASS. A second simultaneous PFTerminal process selected this rental, submitted a 16,575-token turn with zero truncation, invoked the shell tool exactly once, and returned exact output `GLM2_RENTAL_OK` in 65 seconds.
- Termination: requested through the `/gpu` TUI after the passing turn; provider absence was confirmed and both desired/observed state reached `terminated_confirmed`.

## Live rental 3

- Start: 2026-07-16T03:05:35Z.
- Rental: `gpu-a9423e73-bc9d-47c4-94d0-024caeef189f`.
- Vast resource: `45047027` (Italy, 2× H200, $7.6955/hour offer).
- Authorization: $10/hour maximum, $10 total maximum, 90-minute TTL.
- Recipe: `huihui-deepseek-v4-flash-q4k-2xh200-experimental`, revision `huihui-deepseek-v4-flash-q4k-ds4-tp2-r3`, 131,072-token advertised context.
- Purpose: concurrent-controller and second fine-tune/runtime recipe qualification.
- Provider create operations: exactly one, succeeded.
- Bootstrap: PASS on attempt 1.
- Outcome: NOT QUALIFIED. The pinned download wrote approximately 84.28 GB but did not reach `model-ok` or authenticated readiness before the $10 spending cap. The controller requested cleanup and provider absence was confirmed. Final estimated accrued cost: $10.0091.

## Live rental 4

- Start: 2026-07-16T04:12:33Z.
- Rental: `gpu-e555a43f-7253-4be3-be8e-14d337287f7b`.
- Vast resource: `45051607` (2× H200, $8.2950/hour offer).
- Authorization: $10/hour maximum, $20 total maximum, 120-minute TTL.
- A first current-offer attempt (`gpu-d676ee79-bcd9-42f7-9830-0b5c628058ec`) lost a capacity race and failed with `offer-unavailable` before any billable resource existed.
- Purpose: complete the post-recovery approximately 164K-token recall turn without the intentionally small $10 cap interrupting it.
- Readiness: PASS; the rental reached `ready` under the exact candidate.
- Outcome: INCOMPLETE. No acceptance turn was submitted before the 120-minute TTL elapsed. The controller requested cleanup and provider absence was confirmed. Final estimated accrued cost: $16.5676. Readiness is evidence; this rental is not counted as a long-context inference pass.

## Live rental 5

- Start: 2026-07-16T11:39:55Z.
- Rental: `gpu-604f36f4-f424-41ea-9ea7-9349e19a4d4f`.
- Vast resource: `45085271` (Japan, 2× H200, $7.8981/hour offer).
- Authorization: $10/hour maximum, $20 total maximum, 60-minute TTL.
- Purpose: final continuously-driven boot, preserved long-context recall, real tool call, and provider-confirmed teardown.
- Bootstrap gates: `nvlink-ok`, `build-ok`, and all six pinned model-shard checksums passed.
- Failure: NOT QUALIFIED. On model load, CUDA device 0 returned `CUDA-capable device(s) is/are busy or unavailable`; an isolated runtime enumeration reported device 0 as 0 MiB/0 MiB while device 1 reported 143,157 MiB. `nvidia-smi` had misleadingly reported both devices present, idle, and in default compute mode. A second server load reproduced the failure and provider-level GPU reset was unsupported.
- Remediation: recipe r4 adds a post-build CUDA-runtime enumeration for every assigned device and rejects a zero-memory CUDA device before the 231 GB model download. The generalized recipe regression passes in the 71-test GPU-market suite.
- Termination: requested through `/gpu`; provider absence was confirmed and durable desired/observed state reached `terminated_confirmed`. Final estimated accrued cost: $2.5179.

## Live rental 6

- Start: 2026-07-16T12:02:18Z.
- Rental: `gpu-ffd3b5fc-f1e6-4a1d-a99c-a1d3a68ef070`.
- Vast resource: `45086682` (US, 2× H200, $8.2950/hour offer).
- Authorization: $10/hour maximum, $20 total maximum, 60-minute TTL.
- Purpose: replacement-host final long-context recall, real tool call, and provider-confirmed teardown after rental 5 exposed the defective-device class.
- Runtime-device check: PASS. The exact r4 gate was executed against the built runtime before model load; GPU 0 and GPU 1 each reported 143,166 MiB total and 142,639 MiB free.
- Readiness: PASS; authenticated chat, streaming, cancellation, and tool-call probes passed and the runtime registered with a 300,000-token maximum.
- Long-context run: PASS in the preserved real PFTerminal TUI session. The full prompt reached 167,210 input tokens; the server released the turn at 167,471 retained tokens with `truncated = 0` after 1,155,567.95 ms of prompt evaluation.
- Distant recall: PASS. Exact outputs were `LCTX-01-A7F3`, `LCTX-08-S3352`, and `LCTX-13-S2947` for rounds 01, 08, and 13.
- Tool round-trip: PASS. The model invoked the shell tool exactly once with `printf LCTX_TOOL_OK`; the captured output and final report were both exactly `LCTX_TOOL_OK`.
- Transport/session recovery: PASS. The turn crossed the prior 139,264-token failure boundary through the replacement rental endpoint after the same active TUI session reselected a provider with the same model ID.
- Termination: requested through `/gpu`; provider absence was confirmed and durable desired/observed state reached `terminated_confirmed`. Final estimated accrued cost: $5.0314.

## Release disposition

- Live GLM rental paths exercised: capacity race with zero resource, successful readiness, simultaneous PFTerminal processes, 16.5K tool turn, endpoint loss and replacement, 167K long-context recall/tool turn, spending-cap cleanup, TTL cleanup, defective-GPU rejection evidence, and provider-confirmed manual termination.
- DeepSeek fine-tune recipe remains experimental and NOT QUALIFIED in this run because its model download did not finish before the authorized cap.
- GPU rental qualification: PASS for the experimental Huihui GLM 5.2 2×H200 path, including long context. Release may proceed with the automated-suite limitations above stated honestly.
