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
