# PF-35-S01 classifier-corpus handoff

Owner: Raman. Branch/worktree/base and literal scope are authoritative in the
sprint front matter. Shared Cargo/Bazel/lock/navigation files are read-only and
belong to the integration owner.

Implement the remaining PF-35-S01 ledger against the frozen PF-34-S04 contract.
The product decisions are:

- synthetic-first plus commercial-safe CC0, CC BY 4.0, Apache-2.0, MIT, BSD or
  explicitly permitted sources; log exact origin, revision, hashes, license,
  permitted use, attribution and transformations;
- Qwen3.5-27B through pinned vLLM generates English synthetic records on the
  RTX host; ComfyUI is stopped during campaigns; record prompts, seeds,
  sampling, model/runtime/container/driver versions, campaign IDs and hashes;
- initial accepted targets: 250,000 training, 25,000 development/calibration
  and 150,000 separately custodied blind records; generate in versioned rounds
  and expand only from learning-curve evidence;
- independent encrypted blind custody belongs to the human product custodian;
  training agents never receive blind contents or labels;
- primary detector feasibility is DeBERTa-v3-xsmall exported as optimized INT8
  ONNX Runtime; a custom lightweight classifier is the explicit fallback;
- provisional weakest host is Intel N100, 16 GiB RAM, x86-64 Linux; measure
  tokenization plus inference at 2,048-token end-to-end inputs against the plan;
- one calibrated score plus two signed thresholds maps allow/suspicious/hostile;
  only runtime/artifact/resource failure produces unavailable;
- English is the only initially supported language; uncertain or unsupported
  language fails closed and multilingual attacks remain adversarial fixtures;
- campaign labels are provisional. A blind labeling pass checks them, every
  disagreement/low-confidence/suspicious record gets human adjudication, and
  initially 1% stratified human plus non-overlapping 1% Opus audits apply to
  high-confidence agreements. Reassess scalability after 10,000 acceptances;
- the human custodian controls an offline Ed25519 root authorizing a rotating
  release key. Immutable GitHub Release bundles install atomically, verify
  locally, retain rollback and run offline. PF-35-S01 records the evaluation
  contract but must not create or commit private keys or production weights.

Do not fabricate the RTX, N100, private evaluator or large-corpus measurements.
If access is absent, complete every independently provable implementation item,
record the exact external evidence still required and leave those checklist
items open rather than claiming sprint completion.
