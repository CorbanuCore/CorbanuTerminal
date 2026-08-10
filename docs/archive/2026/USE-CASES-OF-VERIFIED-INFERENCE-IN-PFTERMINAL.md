# Use Cases of Verified Inference in PfTerminal

**Research report — 2026-07-20**  
**Recommendation:** ship a narrow, experimental **Verified Evaluation Receipt** before attempting a general “proof of work” or “proof of adversarial audit” product.

## Executive decision

Ambient verified inference is useful when the inference itself is the disputed event: which model processed which fixed input and produced which output. It is not, by itself, proof that an agent ran commands, changed a repository, tested a deployment, found every vulnerability, or completed a Task Node assignment.

The best first PfTerminal use case is therefore a content-addressed evaluation or review packet:

1. PfTerminal freezes and hashes the exact material to evaluate: benchmark case, diff, source bundle, policy, or Task Node evidence packet.
2. A dedicated Ambient verified-inference call evaluates that immutable packet with a pinned prompt, model, and output schema.
3. PfTerminal refuses to issue a receipt unless Ambient returns `verified: true` for an explicitly supported verified model.
4. PfTerminal writes a local receipt containing the packet, prompt, output, configuration, response ID, hashes, and Ambient verification fields.
5. If the receipt is submitted to Task Node, the user's Task Node wallet signs the receipt digest. The signature identifies the submitter; Ambient attests the inference. Neither is presented as proof of the underlying shell work.

This is useful for reproducible benchmark judging, a second opinion on Task Node evidence, and fixed-input code review. It is not enough to market “this repository was adversarially audited.” The honest cyber claim is narrower:

> An Ambient-verified model produced this review over this exact, hash-addressed evidence bundle. The cited tests and reproductions remain independently checkable artifacts.

The previously drafted [Proof of Adversarial Review](docs/features/proof-of-adversarial-review.md) is valuable design exploration, but its proposed third-party receipt and pre-run on-chain commitment should not be implemented as written until Ambient exposes the missing verification surface described below.

## 1. What Ambient actually verifies

Ambient describes Proof of Logits as a useful-proof-of-work system in which model logits act as execution fingerprints. Miners commit hashed progress markers and validators recompute a small portion of an inference rather than repeating the entire job. Ambient's on-chain design includes a job input hash, an output hash, a Merkle root, assigned verifier ranges, and verification state. See the [Ambient overview](https://docs.ambient.xyz/What-is-Ambient-27ee653486a380e3a5c1ecd6754c6ec9?pvs=21), [on-chain documentation](https://docs.ambient.xyz/On-Chain-27ee653486a380c1b652d649e90d6636?pvs=21), and [litepaper](https://ambient.xyz/Ambient_Litepaper_V1.pdf).

That supports a narrow proposition:

> The network accepted evidence that the designated inference computation produced the committed result.

It does not establish that the result is correct, complete, unbiased, secure, or useful. It also does not automatically cover work performed outside the inference call.

### Current public API contract

Ambient's live [OpenAPI schema](https://api.ambient.xyz/openapi.json) exposes two request fields on `POST /v1/chat/completions`:

- `emit_verified`: return verification status;
- `wait_for_verification`: wait for verification before completing the request.

The non-streaming completion response schema exposes:

- `merkle_root`;
- `verified`.

The same OpenAPI document currently exposes no public endpoint for taking a completion ID or Merkle root and independently resolving its chain job, verifier assignments, inclusion proof, or final status. A Merkle root is a commitment, not a self-verifying proof. Until Ambient documents a public lookup or proof-verification path, `verified: true` is best described as **provider-reported, network-backed verification**, not a portable trustless receipt.

### Live behavior observed on 2026-07-20

Small production probes using the current Ambient API produced materially different results by route:

| Route | Requested verification | Result |
| --- | --- | --- |
| `z-ai/glm-5.2` | wait + emit | `merkle_root: null`, `verified: null` |
| `ambient/large` (resolved to GLM 5.2) | wait + emit | `merkle_root: null`, `verified: null` |
| `zai-org/GLM-5.1-FP8`, truncated response | wait + emit | non-null root, `verified: false` |
| `zai-org/GLM-5.1-FP8`, complete non-streaming response | wait + emit | non-null root, `verified: true` |
| `zai-org/GLM-5.1-FP8`, complete streaming response | wait + emit | terminal SSE event `{"verified":true}`; no Merkle root in the stream |

These probes are not a protocol guarantee. They show the current product boundary:

- “Ambient provider” does not mean “verified inference.” PfTerminal's default Ambient model is currently GLM 5.2, while the tested verified path was GLM 5.1 FP8.
- Verification must be fail-closed and model-specific.
- Streaming and non-streaming receipt shapes differ today.
- Truncation must invalidate the receipt.

## 2. What verified inference can and cannot prove

| Claim | Ambient inference helps? | Additional evidence required |
| --- | --- | --- |
| A named model produced output Y from fixed input X | Yes, this is the core use case | Portable verification locator is still needed for independent third parties |
| A benchmark judge applied a pinned rubric to a pinned answer | Yes | Dataset, rubric, parser, score calculation, and sampling config hashes |
| A model reviewed a particular source bundle or diff | Yes | Canonical bundle construction, source/commit hashes, full output |
| The review was correct or found every vulnerability | No | Human review, independent tools, reproductions, coverage and threat-model evidence |
| An agent ran shell commands or tests | No | Captured command/results, CI attestations, reproducible scripts, signed manifest |
| A repository was changed as claimed | No | Commit and tree hashes, diff, build/test artifacts |
| A Task Node task was completed | No | The task's normal signed evidence packet and independently resolvable artifacts |
| A Task Node account endorsed a receipt | No | Wallet signature over the receipt digest |
| A provider did not silently substitute another computation | Potentially | Ambient must document model/version binding and third-party verification semantics |

The important design principle is compositional: use Ambient for the inference boundary, Git hashes and reproducible commands for source artifacts, CI/build provenance for execution, and Task Node wallet signatures for identity and endorsement. Do not ask one proof system to stand in for all four.

### Why not use another attestation primitive?

The alternatives cover different boundaries:

| Primitive | Strongest claim | Main tradeoff |
| --- | --- | --- |
| Ambient Proof of Logits | Network-verified model inference | Does not cover host tools; portable public receipt path is presently unclear |
| Deterministic replay | Same pinned runtime reproduces the same output | Expensive duplication; reproducibility is not an independent attestor |
| TEE audit | Measured enclave ran pinned audit code/model/data | Trusts hardware/vendor and the enclave packaging boundary; see [Attestable Audits](https://arxiv.org/abs/2506.23706) |
| [SLSA provenance](https://slsa.dev/spec/v1.1/provenance) / CI attestation | A build or CI process produced an artifact from identified inputs | Does not attest model inference quality or reviewer identity |
| Task Node wallet signature | A key holder submitted or endorsed a digest | Says nothing about how the signed content was computed |

PfTerminal should compose these primitives when the claim crosses several boundaries. Ambient is compelling because it can attest the expensive model call without requiring a second full inference, not because it subsumes artifact provenance or identity.

## 3. Current PfTerminal gap

PfTerminal already supports Ambient as an OpenAI-compatible provider, but it does not implement verified inference.

At inspected revision `6a279e8104279202b22c85ebadf147910d732001`:

- `codex-rs/codex-api/src/common.rs::ChatCompletionsRequest` has no `emit_verified` or `wait_for_verification` fields.
- `codex-rs/codex-api/src/endpoint/chat_completions.rs::ChatCompletionChunk` does not deserialize `verified` or `merkle_root`; unknown fields are discarded.
- `codex-rs/codex-api/src/common.rs::ResponseEvent::Completed` has no receipt/provenance field.
- the standard chat path is streaming, while Ambient currently emits only a terminal `verified` boolean in the tested SSE path and omits the Merkle root;
- the default Ambient route is `z-ai/glm-5.2`, which did not produce verification data in live probes.

This should not be solved by adding a green checkmark to ordinary Ambient turns. A verified call needs an explicit policy, eligible model set, fail-closed completion semantics, durable receipt storage, and a claim label that distinguishes provider-reported status from independently verifiable proof.

The cleanest first implementation is a separate one-shot evaluation client or job, not a mutation of every agent turn. That client may use non-streaming chat completions until Ambient exposes a complete streaming receipt.

## 4. Current Task Node boundary

Task Node's verification loop is evidence-oriented, not cryptographic inference attestation.

At inspected revision `aa1acd4fe3f770724ec77c7966e1bad3ede141ba`:

- a user submits a wallet-signed `pf.task.submission.v1` evidence packet;
- the review worker uses [`verification_request_v1.md`](../tasknodeofficial/prompts/task_engine/verification_request_v1.md) to ask one evidence-specific follow-up;
- the verification response is another wallet-signed packet;
- [`reward_scoring_v1.md`](../tasknodeofficial/prompts/task_engine/reward_scoring_v1.md) evaluates the offer, submissions, processed artifacts, and evidence classifications;
- public artifacts may be classified as independently resolvable, while text and local-file claims remain self-attested;
- `pf.reward.v1` records the authority's terminal decision and payment.

This is a good integration seam. A verified inference receipt can become a new processed-evidence attachment or scorer-provenance attachment. It should not replace the second verification step, the artifact resolver, or the reward authority.

The layers should remain explicit:

```text
Task Node wallet signature
  -> who submitted or endorsed the packet

Ambient verified inference
  -> which verified inference produced the review/judgment

Artifact hashes + resolvers + reproducible commands
  -> what repository, evidence, and observable results were reviewed

Task Node reward event
  -> what the task authority decided and paid
```

“Signed by someone on Task Node” is useful, but it is an endorsement. It does not make the model right and does not expand Ambient's proof to cover tools.

## 5. Ranked use cases

### 1. Verified benchmark/evaluation receipt — strongest first use

This is the best fit because the full disputed event can be made inference-shaped: a fixed case, answer, rubric, and structured judge output.

Useful examples:

- judging PfTerminal versus another harness on a held-out task;
- scoring a model-generated patch against a fixed rubric after deterministic tests have run;
- evaluating visual or text outputs from a frozen evidence packet;
- replaying a governance or routing packet with a pinned schema.

The receipt does not prove that the benchmark harness reported latency or billing honestly. Those remain separate machine/billing artifacts. It does make a model-judge result harder to silently replace.

### 2. Verified Task Node evidence review — valuable second use

A Task Node reviewer could request an Ambient-verified second opinion over the canonical task offer, initial evidence, follow-up, processed evidence, and prompt version. The resulting receipt would show which model produced which recommendation over which packet.

This is useful for disputes, high-value tasks, random audits, and comparing an authority's ultimate reward decision to an independently generated recommendation. It should be advisory at first. Automatically paying based only on an LLM receipt would confuse reproducible inference with correct adjudication.

### 3. Verified fixed-input code review — useful with careful wording

PfTerminal can freeze a review packet containing:

- repository and commit/tree identifiers;
- canonical diff or source-bundle hash;
- threat model and review mandate;
- deterministic static-analysis/test outputs;
- requested structured findings schema.

Ambient can then verify the model inference that generated the review report. Each finding should cite files/lines and, where possible, a reproduction or deterministic check.

The seal must say **Verified model review**, not **verified secure** or **audit passed**. A verified weak review remains weak. Tool execution and dynamic exploitation are outside the inference receipt.

### 4. Governance/replay work items — credible, but not autonomous decisions

The latest Post Fiat research strongly supports hash-addressed replay. [LLM Governance Replay](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/llm-governance-replay.md) separates replay fidelity from decision quality and treats the model output as a public, contestable work item. [Cross-Hardware SGLang Replay](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/sglang-cross-hardware-replay.md) binds packet, prompt, runtime profile, machine receipt, raw output, and parsed output hashes, and correctly rejects comparisons when prompt hashes differ.

Ambient can reduce the need to reproduce identical inference across rented machines, but it should not remove packet hashes, output hashes, challenge paths, or human overrides. Verified execution and good governance judgment remain separate properties.

### 5. Routine per-turn verification — poor use

Most PfTerminal turns are not externally disputed events. Verifying all of them adds latency, cost, storage, and confusing badges while leaving tool execution unproven. Receipt generation should be explicit and reserved for claim-bearing boundaries.

### 6. “Proof the task was done” — unsupported as a standalone claim

Ambient can prove the reviewer/scorer inference, not the user's work. Treating it as proof of task completion would reduce evidence quality by replacing inspectable artifacts with a model provenance claim.

## 6. Recommended v0: Verified Evaluation Receipt

### User promise

> PfTerminal can produce a tamper-evident packet showing that an Ambient-verified model evaluated an exact, hash-addressed input and returned an exact output. The receipt states what it proves and what remains ordinary evidence.

### Canonical receipt

Use versioned canonical JSON, with secrets and private reasoning excluded:

```json
{
  "schema": "pfterminal.verified_evaluation_receipt.v1",
  "claim": "ambient_verified_inference_over_content_addressed_packet",
  "packet": {
    "kind": "benchmark_judge | tasknode_review | source_review",
    "sha256": "...",
    "manifest": "relative/path/or-cid"
  },
  "request": {
    "provider": "ambient",
    "model": "zai-org/GLM-5.1-FP8",
    "prompt_sha256": "...",
    "prompt_template_sha256": "...",
    "configuration": {},
    "started_at": "..."
  },
  "response": {
    "id": "...",
    "output_sha256": "...",
    "finish_reason": "stop",
    "merkle_root": "...",
    "verified": true,
    "completed_at": "..."
  },
  "pfterminal": {
    "version": "...",
    "git_revision": "..."
  },
  "limitations": [
    "does_not_attest_tool_execution",
    "does_not_attest_correctness_or_completeness",
    "ambient_public_third_party_lookup_not_recorded"
  ],
  "signatures": []
}
```

Store the full canonical input and output beside the receipt or refer to a durable content-addressed object. A hash without retrievable content is not reviewable evidence.

### Fail-closed policy

A receipt is valid only when all of the following hold:

- provider is Ambient;
- model is on an explicitly tested verified-route allowlist;
- `wait_for_verification` and `emit_verified` were requested;
- the response is complete and has the expected structured shape;
- `finish_reason` is `stop`, not `length`, filtering, disconnect, or unknown;
- `verified` is exactly `true`;
- the required commitment fields for that transport are present;
- input, prompt, output, model/config, and PfTerminal version hashes are recorded;
- no secret, API key, hidden reasoning, or private unredacted evidence enters a publishable receipt.

`null`, `false`, missing fields, retries that change input, or a provider fallback must produce **no verified receipt**. PfTerminal may still preserve the ordinary response, clearly labeled unverified.

### Product surface

Avoid a global verification toggle. Start with an explicit action attached to a fixed artifact:

- **Create verified evaluation** from a benchmark result, evidence packet, or source-review bundle;
- **View receipt** with model, packet hash, output hash, verification status, and limitations;
- **Export receipt** as JSON plus its referenced manifest;
- **Attach to Task Node submission** only after the user reviews what is being signed.

The UI should use different labels for:

- `Ambient verified` — API returned verified status and required receipt data;
- `Independently verifiable` — a third-party lookup/proof was successfully checked;
- `Task Node signed` — a wallet signed the receipt digest;
- `Artifacts reproduced` — deterministic external checks were actually rerun.

Do not collapse these into one green “verified” badge.

## 7. Cyber-review extension

After v0 works, a source-review packet can support a useful public statement:

> Commit C was reviewed by Ambient model M under mandate hash P; Ambient reported the inference verified; the resulting report hash is O; the listed tests and reproductions are available in manifest A.

To make “adversarial” meaningful, the mandate should define attack classes, excluded surfaces, available evidence, and the required finding schema. The report should preserve failures and residual risk. A Task Node reviewer can sign the receipt digest to endorse that they inspected the packet, and multiple reviewers can independently attach findings.

What it still cannot say:

- the repository is secure;
- every reachable path was reviewed;
- the model executed the reproduction commands;
- the reviewer is socially or economically independent merely because the inference ran on Ambient;
- the receipt covers a later commit.

The earlier PoAR proposal's committed-before-run mandate is optional defense against cherry-picking, not a prerequisite for v0. It only becomes meaningful if Ambient exposes an enumerable chain job/receipt surface. Otherwise an operator can omit abandoned attempts without an external observer being able to discover them.

## 8. Implementation sequence

### Phase A — receipt spike

1. Add a small Ambient-specific non-streaming verified-evaluation client outside the ordinary turn loop.
2. Parse and persist response ID, root, status, usage, finish reason, exact output, and timestamps.
3. Add fixtures for verified true, false, null, missing root, truncation, malformed output, transport loss, provider fallback, and retry with changed body.
4. Run live probes against every candidate Ambient model and publish a compatibility matrix with date and API schema digest.
5. Do not expose a trust badge yet.

### Phase B — benchmark packet

1. Define canonical JSON and hashing for one existing PfTerminal benchmark/judge packet.
2. Produce a verified structured judge output.
3. Recompute all hashes in a standalone verifier with no Ambient credential.
4. Label the result provider-verified unless the standalone verifier can independently resolve Ambient network state.
5. Compare ordinary versus verified latency, cost, failure rate, and output consistency across at least 20 fixed cases.

### Phase C — Task Node attachment

1. Add the receipt as a bounded evidence reference, not pasted model context.
2. Preserve the existing second verification request and reward scorer.
3. Let the Task Node scorer distinguish `provider_verified_inference`, `independently_verified_inference`, and ordinary self-attested evidence.
4. Sign the receipt digest with the submitting wallet only after explicit user action.
5. Keep reward decisions advisory until replay, injection, privacy, and disagreement behavior have been measured.

### Phase D — source review

1. Define a canonical source/diff evidence manifest.
2. Run a fixed-input review with structured findings and honest scope labels.
3. Bind external test/CI artifacts without claiming Ambient executed them.
4. Require a new receipt whenever the covered tree changes.
5. Evaluate whether reviewers actually find useful defects versus an ordinary unverified review.

## 9. Acceptance criteria

The v0 is shippable only if:

1. A third party can recompute packet, prompt, and output hashes from exported artifacts.
2. Verified and unverified Ambient responses are reliably distinguished, including null and truncated cases.
3. A model/provider fallback cannot inherit a verified badge.
4. The exact verified model and API-schema digest are recorded.
5. Receipts never contain provider credentials, vault values, hidden reasoning, or unbounded private evidence.
6. Existing PfTerminal turns are unaffected unless the user explicitly requests a verified evaluation.
7. Task Node keeps its ordinary evidence and follow-up verification semantics.
8. A published receipt states that it does not attest tool execution, task completion, correctness, or review completeness.
9. If a third-party Ambient verification locator is unavailable, the UI and receipt say `provider verified`, not `independently verified`.
10. Live qualification covers success, verification false/null, disconnect after output, truncation, retry, duplicate receipt prevention, and resume.

## 10. Questions Ambient must answer

Before PfTerminal markets these receipts as independently verifiable, obtain authoritative answers to:

1. How does a completion ID or Merkle root map to the on-chain `JobRequest` and `VerificationState`?
2. What public RPC, program address, account, transaction, or API endpoint lets a third party verify status without the original API key?
3. What exactly is committed: raw prompt bytes, normalized messages, token IDs, logits, output token IDs, model weights/version, tokenizer, quantization, sampling settings, and tool definitions?
4. What inclusion proof connects one completion to the returned Merkle root?
5. What finality threshold and verifier quorum make `verified: true` final?
6. How are failed, retried, cancelled, and truncated jobs represented?
7. Why does the tested streaming path omit the Merkle root, and is that contractual?
8. Which production models are currently verified, and can that eligibility change without a versioned API signal?
9. Does verification cover only generated tokens, or also prompt processing and cached prefixes?
10. How should private prompts be independently verified without publishing sensitive contents?

Until these are answered, claims should remain deliberately conservative.

## 11. Fit with Post Fiat's existing evidence doctrine

The most recent Post Fiat material consistently treats proof labels as claim boundaries:

- [Proof of Disclosed Leverage](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/proof-of-leverage.md) says a proof over disclosed accounts is not proof of global solvency.
- [LLM Governance Replay](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/llm-governance-replay.md) says replay fidelity is not decision quality.
- [Cross-Hardware SGLang Replay](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/sglang-cross-hardware-replay.md) binds packet, prompt, runtime, receipt, and output hashes and quarantines prompt drift.
- [Cobalt Implementation Evidence](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/cobalt-implementation-evidence.md) makes a narrow devnet claim and publishes expected-fail readiness boundaries rather than calling the system adopted or decentralized.
- [pfUSDC: A Stablecoin Bridge Secured by Proofs, Not Committees](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/pfusdc-trustless-bridge.md) explicitly says a proof is only as sound as the program it proves.
- [Proving on Apple Silicon](https://github.com/postfiatorg/postfiatorg.github.io/blob/main/content/blog/proving-on-apple-silicon.md) is labeled an R&D plan rather than a result and requires measured gates.

Verified inference should follow the same doctrine. The useful product is not a larger “verified” adjective. It is a smaller claim that can survive hostile reading.

## Inspection basis

- Ambient documentation and live API/OpenAPI were checked on 2026-07-20. The observed OpenAPI document SHA-256 was `94c4192707d3915056f08c7a03151bdad63da3128393bf6e27699eb82b9e2002`.
- PfTerminal code was inspected at `6a279e8104279202b22c85ebadf147910d732001`; the worktree already contained unrelated local changes, which this report did not modify.
- Task Node Official was inspected at `aa1acd4fe3f770724ec77c7966e1bad3ede141ba`, including its evidence processing, verification-request prompt, reward-scoring prompt, signed submission lifecycle, and forensics documentation.
- The Post Fiat site was fetched from `origin/main` and read at `54d1eea8bdfcc92d35ef7279fcea6795ec0fc6f4`. The dirty local content worktree was not merged or altered.
- The live probes used small requests and did not print or persist the Ambient credential in the report or receipt artifacts.

## Conclusion

Ambient verified inference is not pointless, but its value is narrower than “proof an AI did the work.” It is a provenance primitive for claim-bearing model calls. PfTerminal can use it effectively by freezing the input, pinning the policy, failing closed on verification, preserving exact output and hashes, and composing that receipt with ordinary artifact evidence and Task Node signatures.

Ship the benchmark/evaluation receipt first. Add Task Node advisory review second. Add a carefully labeled source-review receipt third. Do not ship a universal “adversarially audited” seal or automatic reward authority until Ambient provides a portable verification path and PfTerminal separately attests the tool and artifact boundaries.
