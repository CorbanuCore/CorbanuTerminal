# Classifier corpus and evaluation preparation

This directory contains public, secret-free PF-35 evaluation contracts. It does
not contain model weights, generated corpus records, blind examples, labels,
signing keys, customer information, credentials, or protected financial data.

`corpus-manifest.json` pins the public source metadata, licensing rules,
synthetic-campaign contract, custody boundary, hardware target, and external
evidence that still must be supplied. `split-manifest.json` freezes grouped
split and holdout requirements without exposing evaluator-owned data. Both
manifests use exact structural allowlists plus bounded strings and arrays;
unknown nesting and wrong-typed scalar values fail closed.

Validate the preparation contract with:

```console
python3 scripts/security-classifier-eval \
  --manifest qa/security-levels/classifier/corpus-manifest.json \
  --splits qa/security-levels/classifier/split-manifest.json \
  --output /path/outside-the-repository
```

`--predictions` accepts strict-schema record-level development rows only. Every
row is bound to the checked corpus and split manifest digests. The evaluator
also emits a deterministic SHA-256 fingerprint over sorted canonical group
tuples using the method pinned in the split manifest, counts same-label sibling
records once in that group set and in per-cohort group counts, and rejects
contradictory labels or scopes. Unknown
fields, blind/train rows,
contradictory truth/scope labels, mixed identities, Boolean numbers, and
score/threshold mismatches fail closed. The report binds the exact predictions
file by evaluator ID/report-schema version, path, kind, byte count and SHA-256;
artifact and threshold
identity fields remain explicitly `unverified-declaration` until successor
release verification.

`--blind-aggregate` accepts one custodian-produced sufficient-statistics object;
it cannot contain blind rows or unknown fields. Supply the prior evaluator
output through `--development-report`; the blind report must match its
development group fingerprint and model/threshold identity. The report also binds the exact aggregate and
development-report files by
evaluator ID/report-schema version, path, kind, byte count and SHA-256.
Statistical gates use Wilson 95%
confidence bounds, including a conservative difference of Wilson bounds for
hard-negative perturbation, and distinguish measured `fail` from
missing evidence. The reported blind total must satisfy a 140,000-record floor
and ±10,000 tolerance around the approximately 150,000-record target, and the
actual observed count is included in the report. The schema reserves a detached
Ed25519 signature envelope but exposes it only as `unverified`; a successor
verifier must authenticate it. Unequal cross-split group fingerprints are
declarations, not proof of disjointness; a custodian-side group-tuple overlap
audit remains required. N100
latency/RSS, model size, signed artifact identity, the actual corpus campaign,
and private custody remain `external-evidence-required`, so this preparation
tool cannot claim full qualification from self-reported numbers.

The integration owner wired `scripts/test_security_classifier_eval.py` and the
shipped evaluator smoke into recurring multi-platform CI during integration.
JSON objects reject duplicate keys, prediction volume and byte line length are
explicitly bounded, JSONL splits only on LF, and all dynamic operator-facing
errors escape control characters.
Input files are snapshotted from one opened regular file; parsing, byte count
and SHA-256 all use those identical bytes. Reports are written atomically.

## External synthetic campaign

`generator-candidates-v1.json` pins the candidate repositories, immutable
revisions, parent provenance, licenses, architectures, quantization formats,
tokenizers and artifact hashes considered for the external generator. The
selection pins `preetpatel/Qwen3.8-27B-Uncensored-NVFP4` at its immutable
revision after the same-host result recorded in
`../sprints/PF-35-S01/generator-qualification-2026-08-31.json`; model-card
performance claims remain comparators, not campaign evidence.
`pf35_vllm_launch.sh` is the secret-free exact launch recipe for that accepted
host/runtime layout. It binds loopback only, disables request/output logging,
uses the measured O2/CUDA-graph configuration and serializes compilation to
avoid the observed parallel-build memory spike.

`pf35_bakeoff.py` sends only bounded synthetic defensive research requests to a
local OpenAI-compatible endpoint. It records refusal, exact JSON-format,
throughput, latency and request-error aggregates plus response hashes, never
response content. The selected runtime uses a strict JSON schema so item count,
labels, minimum text length and additional-property exclusion are enforced
during decoding. Keep raw output outside Git until reviewed and reduced to safe
aggregate evidence.

`campaign-config-pilot-v1.json` is retained only to reproduce the quarantined
first-pilot identity; generation refuses legacy schema 1 configurations.
`campaign-config-canary-v2.json` uses a label/scope-compatible coverage matrix,
short/medium/long artifact buckets and class-specific artifact contracts.
Attack-class text must be attacker-authentic content without research,
classification, family, decisive-signal, placeholder or fragment-mechanism
exposition. Benign and hard-negative plans carry the descriptive, quoted,
negated, historical, classifier-discussion and inert-code cases instead.

`campaign-config-canary-v3.json` is the current generation contract. It adds
label-compatible attack families, makes explicit tool redirection hostile-only,
requires family-specific operative signals, rejects human-only phishing and
narrator wrappers, requires a concrete attack-like lookalike in every hard
negative, and requires the operative verb inside non-English attack clauses.
It also rejects benign safety self-announcement, label/test-outcome leakage,
incomplete long artifacts, trailing ellipses, schema-ceiling text, contradictory
action state and high-confidence strata too small for disjoint audits. Schema 2
remains readable as immutable evidence, but only schema 3 may generate.

`pf35_campaign.py` creates deterministic balanced compatible request plans and
validates the returned provisional records. It rejects unexpected schemas,
invalid label/scope pairs, secret-like material, exact duplicates, cross-group
near-duplicates, semantic exposition, improperly framed hard negatives and
text that reaches the schema ceiling rather than ending naturally. Its
per-plan response schema fixes the item count, label, family scope, assigned
length bounds and confidence bounds during decoding. Every
disagreement, low-confidence record and `suspicious` label enters the
human-review queue. High-confidence agreements receive
deterministic, non-overlapping, per-stratum 1% human and 1% Opus audit samples.
An undersized high-confidence stratum is rejected from the provisional set
rather than overlapping reviewers or aborting the whole round.
The tool writes raw/provisional/rejected JSONL only to an operator-supplied
external directory and appends a verified SHA-256 hash-chained campaign ledger.
Prior rounds are loaded before deduplication so a new round cannot silently
repeat an earlier group.

The `quarantine` subcommand verifies every generation output against the
campaign ledger, writes a hash-bound `QUARANTINED.json` marker and appends the
decision to the ledger. A marked round cannot be adjudicated. Quarantined
records remain failure evidence only and must not enter training, import,
deduplication roots for replacement rounds, or scale-up decisions.

After reviewers return strict-schema JSONL decisions, the `adjudicate`
subcommand requires a human decision for every mandatory review and human
sample, an Opus decision for every separate Opus sample, and human resolution
of every Opus disagreement. It rejects decisions for unselected records and
inconsistent `accept`/`relabel` actions, writes final accepted and rejected
rows exclusively, and appends the decision/input/output hashes to the same
ledger. Reviewer decision rows contain exactly `record_id`, `action`
(`accept`, `reject`, or `relabel`), nullable `final_label` and
`final_family_scope`, `reviewer`, `timestamp_utc`, and `reason`.

These tools do not create or process blind rows. The human custodian runs blind
generation, encryption, grouping and later evaluation in a separate location;
the training lane receives only the sealed manifest identity, signed overlap
statement and, during S03, the allowlisted signed aggregate.
