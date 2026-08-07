# Proof of Adversarial Review

**A proposal for integrating Ambient verified inference into PFTerminal as a review-attestation primitive**

*Draft v0.1 — 2026-07-17*
*Status: proposal; no implementation committed*

---

## 1. Abstract

PFTerminal orchestrates multi-agent software campaigns: a Nazgul (CTO-class model) decomposes goals, Trolls supervise, Orcs implement. Its doctrine already treats every worker report as unproven until re-runnable evidence lands. But one claim resists internal verification entirely: **that an adversarial review actually happened, the way it is described, on the artifact that shipped.** The host is the root of trust for everything inside one PFTerminal instance — it owns the rollouts, the dispatch ledger, and the review pane itself — so no evidence produced inside the instance can prove the review wasn't collusive, softened, or retrofitted.

This paper proposes **Proof of Adversarial Review (PoAR)**: a protocol that binds an adversarial review to Ambient's Proof of Logits consensus so that a third party — a customer, an auditor, a TaskNode dispute arbitrator — can verify that a fixed external model executed an unsoftened adversarial mandate over exactly the shipped diff, that the mandate was committed before the outcome existed, and that the verdict and its attack artifacts have not been altered since. PoAR proves *process integrity*, not code correctness; we are explicit about the boundary and design the protocol so that everything beyond it remains checkable by ordinary re-execution.

## 2. Background

### 2.1 Ambient and Proof of Logits

Ambient is an SVM-compatible Proof of Work L1 (a Solana fork substituting PoW for PoS at full Solana speeds) whose consensus doubles as an inference-verification scheme. Its properties relevant here:

- **Proof of Logits (PoL).** In a language model, logits are the raw numerical outputs from which token probabilities are derived; they act as fingerprints of model execution. Ambient validators check hashes over these fingerprints so that an inference is verified as a byproduct of consensus — advertised at roughly 0.1% overhead, without ZK proof latency, TEE trust assumptions, or optimistic challenge windows.
- **What PoL attests.** That *a specific model, on a specific prompt, produced a specific output, at a specific time*. It does not attest that the output is *true* — only which computation produced it. This is exactly the attestation shape PoAR needs: we never claim the chain certifies code safety; we claim it certifies review execution.
- **Single canonical model, fixed architecture.** Ambient focuses all miners on one ever-evolving network model (600B+ class) and its fine-tunes, with weights, fine-tuning, and quantization auditable on-chain. "Which model ran" is therefore not a self-reported string — it is consensus state, so the reviewer cannot be silently downgraded or substituted.
- **PoW economics.** Useful-Proof-of-Work rewards for inference give the verification layer an existence independent of any single API operator: the attesting party has no business relationship with the review's subject.
- **OpenAI-compatible API.** `https://api.ambient.xyz/v1` serves chat completions for the network model, which PFTerminal already consumes as a provider (`ambient`, with vault-managed credentials).

### 2.2 PFTerminal orchestration and the trust gap

PFTerminal's evidence hierarchy is: benchmark output, then failing-then-passing tests, then diffs plus logs, then prose. This works for *results* because results are re-runnable. It fails for *review process* because:

1. **Self-review is indistinguishable from independent review.** Both panes are controlled by the same operator on the same host; "the reviewer was independent" is theater when one party holds both transcripts.
2. **The mandate is invisible downstream.** A published "red team report" carries no proof the reviewer's instructions were adversarial rather than "give it a quick look."
3. **Evidence substitution is undetectable.** Nothing proves the diff the reviewer saw is the diff that shipped.
4. **Verdicts are editable.** Rollout files are local, operator-controlled, and mutable after the fact.

Notably, the remedy is *not* more internal machinery: any ledger inside the trust boundary inherits the boundary's trust problem. PoAR therefore makes a single, narrow crossing of the boundary — one attested inference per review — and keeps everything else local and re-runnable.

## 3. Threat model

**Attacker.** The PFTerminal operator (or the agent hierarchy itself), who wishes to publish a fraudulent adversarial-review claim: fabricated verdicts, softened mandates, stale or partial diffs, self-review presented as independent, or cherry-picked review runs.

**Verifier.** A party who was not present: a customer buying an audited deliverable, a TaskNode arbitrator resolving a dispute, a downstream maintainer evaluating a contributor's security claims, or the operator's own future self after logs have aged.

**Trusted.** Ambient consensus (PoL verification of the attested inference); collision resistance of SHA-256; the verifier's ability to re-execute commands.

**Explicitly not trusted.** The PFTerminal host, its rollouts, its dispatch ledger, the reviewing model's honesty, and any prose in the review report.

**Out of scope.** Proving the review was *thorough* or *competent* (Section 7), proving properties of artifacts that are directly re-checkable (test results, diffs — these need no attestation), and Sybil resistance of the verifier population.

## 4. Protocol

PoAR is a three-phase commit-attest-publish protocol. One review = one on-chain commitment plus one attested inference plus one receipt document.

### 4.1 Canonicalization

All hashed artifacts use deterministic encodings: UTF-8, LF line endings, RFC 8785 (JCS) canonical JSON for structured values, and `git diff` output normalized against the merge-base of the shipped commit range. The diff digest is:

```
diff_digest = SHA-256( normalize(diff(merge_base..ship_commit)) )
```

`ship_commit` is the commit the review claims to cover; `merge_base` anchors the diff so the digest is reproducible by any clone of the repository.

### 4.2 Phase 1 — Mandate commitment (pre-run)

Before the review inference executes, PFTerminal constructs the mandate bundle:

```
mandate = {
  schema:         "poar/mandate/1",
  mandate_text:   <adversarial review instructions, verbatim>,
  model:          <Ambient network model id, e.g. "z-ai/glm-5.2">,
  diff_digest:    <from 4.1>,
  ship_commit:    <40-hex>,
  repo:           <canonical remote URL>,
  verdict_schema: <the output contract the reviewer must fill>,
  nonce:          <16 random bytes, hex>
}
mandate_hash = SHA-256( JCS(mandate) )
```

`mandate_hash` is anchored to the Ambient chain (a memo transaction from the operator's key) and **timestamped by consensus before the review run exists**. This defeats mandate-shopping: a mandate hash published after seeing outcomes proves nothing, so verifiers require the commitment to precede the attested run.

The nonce prevents grinding on mandate variants to find one that yields a friendly verdict while reusing a pre-committed hash.

### 4.3 Phase 2 — Attested review inference

The reviewer prompt is assembled deterministically:

```
prompt = mandate_text || "\n\n--- diff ---\n" || full_diff || "\n\n--- verdict schema ---\n" || verdict_schema
```

and submitted to Ambient's API as an ordinary chat completion against the network model named in the mandate. PoL does the rest: validators verify the logits fingerprint of the execution, and the completed inference is consensus-verified, yielding an **attestation receipt** `R` binding:

```
R = ( model, prompt_commitment, output_commitment, block_height, validator_set_proof )
```

The exact receipt format is whatever Ambient's API or chain exposes for a verified completion (Section 8.1 lists the integration spike to pin this down); PoAR treats it as an opaque, third-party-checkable object.

Because the reviewer executes on **infrastructure the operator does not control**, on a **model whose identity is consensus state**, over a **prompt whose hash was committed in advance**, the four failure modes of Section 2.2 collapse: independence is infrastructural, the mandate is pinned, the diff is hashed into the committed mandate, and the output is anchored at production time.

### 4.4 Phase 3 — Verdict schema and artifact binding

A plain-language verdict is unverifiable prose. PoAR requires the reviewer to emit a structured verdict:

```
verdict = {
  schema:       "poar/verdict/1",
  mandate_hash: <from 4.2>,
  disposition:  "pass" | "pass-with-findings" | "fail",
  attacks: [
    {
      class:         <attack category, e.g. "auth-bypass", "secret-leak", "supply-chain">,
      narrative:     <what was attempted>,
      repro:         <shell command or test path, re-runnable by the verifier>,
      artifact_hash: <SHA-256 of any generated exploit/poc file>,
      outcome:       "rejected-by-code" | "confirmed-fixed" | "exploitable"
    }
  ],
  residual_risk: <bounded free text, max 1 KiB>
}
```

Two properties matter:

- **Attacks must carry re-runnable artifacts.** "Tried to break it" and "barely tried" are indistinguishable on-chain; the checkable substance of adversarial effort is the set of reproductions. Each `repro` is an ordinary command a verifier can execute against `ship_commit`; each `artifact_hash` binds any generated proof-of-concept to this review so artifacts cannot be borrowed from a different engagement.
- **The output commitment in `R` covers this exact JSON.** Editing a finding, upgrading a disposition, or swapping a repro after the fact invalidates the attestation.

### 4.5 The PoAR receipt

The published artifact is a single JSON document committed to the repository (`.poar/<ship_commit>.json`) and referenced by a git trailer on the shipped commit:

```
Adversarial-Review: poar/1 sha256=<receipt_hash>
```

```
receipt = {
  schema:      "poar/receipt/1",
  mandate:     <full mandate bundle from 4.2>,
  mandate_tx:  <Ambient txid of the commitment>,
  attestation: <R from 4.3>,
  verdict:     <full verdict from 4.4>,
  verdict_raw: <the model's exact completion text>,
  artifacts:   { <path>: <artifact_hash> },
  pfterminal:  { version, campaign_id, hierarchy: { nazgul, troll } }
}
```

### 4.6 Verification procedure

A verifier with the repository and the receipt:

1. Recomputes `diff_digest` from `merge_base..ship_commit`; checks equality with the mandate's.
2. Recomputes `mandate_hash`; confirms `mandate_tx` anchored it on Ambient **before** the block height in `attestation`.
3. Verifies `attestation` against Ambient (validator proof / API), confirming the named network model produced `verdict_raw` from a prompt embedding the mandate and the diff.
4. Recomputes the prompt from receipt fields; confirms it matches the attested prompt commitment.
5. For each attack: checks out `ship_commit`, executes `repro`, compares results and `artifact_hash` values.
6. Checks the git trailer on `ship_commit` matches the receipt hash.

No step trusts the operator. Steps 1–2 and 5–6 are pure re-execution; steps 3–4 are Ambient's verification surface. Total verifier effort is minutes, dominated by step 5.

## 5. PFTerminal integration

### 5.1 The `pfterminal-poar` plugin

A single plugin exposing four commands:

| Command | Role |
|---|---|
| `pfterminal poar commit <range>` | Build mandate bundle, anchor `mandate_hash`, print the mandate file |
| `pfterminal poar review <mandate>` | Dispatch the adversarial review through Ambient, capture receipt `R` |
| `pfterminal poar seal <receipt>` | Validate verdict schema, write `.poar/<commit>.json`, amend trailer |
| `pfterminal poar verify <receipt>` | Run the Section 4.6 procedure, exit non-zero on any failure |

The Ambient API key is fetched at use time only: `AMBIENT_API_KEY="$(pfterminal vault auth-helper provider/ambient_api_key)"`. Keys never appear in receipts, transcripts, or mandates.

### 5.2 Orchestration semantics

PoAR slots into the existing hierarchy as a **gate**, not a participant:

- The Nazgul declares a campaign milestone *review-gated* by writing the mandate. Mandate authorship is a Nazgul decision (it defines what "done" means); PoAR changes nothing about command structure.
- The **reviewer is not a pane**. It is a single stateless Ambient inference constructed by the plugin. This is deliberate: an Orc or Troll reviewer lives inside the trust boundary and would reintroduce operator control over the review transcript. The hierarchy's role is limited to *preparing* the mandate and *remediating* findings.
- Findings with `outcome: exploitable` re-enter the normal flow as ordinary dispatches to the Troll; the remediated commit range produces a fresh mandate hash and a fresh receipt. Receipts form a chain: receipts may reference `supersedes: <prior mandate_hash>`.
- The dispatch machinery is untouched. PoAR adds no new inter-pane protocol; it consumes the same shipped-commit boundary a release gate already has.

### 5.3 UX in the TUI

- `/poar` opens the receipt viewer for the current worktree: disposition, attacks with one-keystroke repro re-runs, attestation status (chain-confirmed / pending / failed).
- A review-gated milestone shows a seal icon in `/spawn status`: grey (no receipt), amber (receipt predates HEAD), green (receipt covers HEAD).
- `poar verify` failures surface as ordinary error events; there is no silent state.

### 5.4 Economics

Each gated milestone costs: one memo transaction (commitment) plus one large-context inference (the review) plus negligible verification calls. The review inference is the dominant cost and is sized by the diff; the natural operating mode is gating *milestones*, not commits. Verification by third parties is free of operator interaction, which is the point: the marginal cost of convincing the Nth party is zero.

## 6. Security analysis

**Mandate substitution.** Committed-before-run `mandate_hash` (Phase 1) plus attested prompt embedding the mandate (Phase 2) closes it. The nonce blocks hash reuse across mandate variants.

**Evidence substitution.** `diff_digest` inside the committed mandate; verifier recomputation (4.6 step 1) closes it.

**Verdict doctoring.** Output commitment in `R` covers `verdict_raw`; any edit invalidates step 3.

**Self-review.** The reviewer executes on Ambient validators, not the operator's host; model identity is consensus state, not a config string.

**Cherry-picked runs.** Commitment ordering proves the mandate predates outcomes, but cannot prove the published run was the *only* run. Mitigation: acceptance policies may require "the first attested run against `mandate_hash` after the commitment block," which is enumerable from chain data. Residual risk: an operator can still abandon a mandate entirely and recommit; the receipt chain (5.2) makes abandoned mandates visible in-repo.

**Forged re-runs.** Attack artifacts are hash-bound and re-executed by the verifier against the shipped commit; a repro that does not reproduce fails verification regardless of the attestation.

**Model downgrade.** The network model is Ambient's consensus focal point; serving a weaker model is a consensus violation PoL exists to catch. This is the load-bearing property Ambient provides and no self-hosted alternative replicates.

## 7. Limitations — stated, not footnoted

1. **PoAR does not prove code is safe.** It proves a specified adversarial process executed faithfully. A faithful review by a weak model is faithfully weak.
2. **Adversarial effort is evidenced, not proven.** The protocol forces effort into re-runnable artifacts and binds them to the review, but a reviewer can still produce shallow artifacts. Acceptance policy (minimum attack classes, required repro coverage) lives outside the protocol.
3. **Selection is mitigated, not eliminated.** Commitment ordering plus first-run acceptance rules raise the cost of cherry-picking; abandoned mandates remain possible.
4. **Trust in Ambient is real.** PoAR exchanges trust in the operator for trust in PoL consensus, SHA-256, and the Ambient validator set. That is a good trade at a trust boundary and a pointless one inside it — PoAR should never be used to convince the operator of anything.
5. **Receipt format dependency.** The proposal assumes Ambient exposes a per-completion, third-party-verifiable receipt. If the current API only verifies at aggregate/block granularity, Section 4.3's `R` needs an on-chain lookup step and the integration spike (8.1) becomes a hard prerequisite.

## 8. Open questions and next steps

### 8.1 Integration spike (prerequisite)

- Determine the exact form of a per-completion PoL receipt from `api.ambient.xyz`: returned inline, or resolved from chain state by completion id? What does offline verification cost?
- Confirm the prompt/output commitment scheme (raw hash vs. structured commitment) and whether large diffs (100k+ tokens) fit the network model's context with the verdict schema.
- Price a representative milestone review end-to-end on testnet.

### 8.2 Protocol hardening

- Multi-reviewer configurations: k-of-n independent mandates with disjoint attack-class assignments, single receipt bundle.
- Continuous mode: mandate templates per repo with automatic recommit on milestone boundaries, so abandoned-mandate gaps are themselves auditable.
- TaskNode binding: receipts as first-class dispute evidence, with `poar verify` exposed to arbitrators as a hosted check.

### 8.3 What would falsify this proposal

- If Ambient receipts cannot be verified by a third party without operator cooperation, the trust-boundary argument collapses and PoAR is ceremony.
- If review-grade diffs routinely exceed the network model's effective context, the protocol needs a chunking construction that reintroduces prompt-integrity questions — at that point the design should be revisited, not patched.

## 9. Conclusion

Inside one PFTerminal instance, the host already sees everything; on-chain verification there is overhead without a trust problem. At the boundary — convincing a customer, an auditor, or an arbitrator who wasn't in the room — the same claims are unverifiable prose. Proof of Adversarial Review crosses exactly once, with exactly the attestation Ambient's Proof of Logits provides: a committed mandate, a consensus-verified execution, and an anchored verdict whose adversarial substance remains checkable by ordinary re-execution. It proves the fight was real, in a neutral venue, with the right target — and it is honest that no protocol can prove the punches were good ones.

---

*References: Ambient litepaper and site (ambient.xyz — Proof of Logits, verified inference, SVM-compatible PoW L1); PFTerminal spawn-orchestration and role doctrine (this repository, `codex-rs/core/src/agent/builtins/`, `codex-rs/tui/src/spawn_orchestration.rs`); RFC 8785 (JSON Canonicalization Scheme).*
