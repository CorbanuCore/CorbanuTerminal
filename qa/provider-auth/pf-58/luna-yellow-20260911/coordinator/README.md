# Coordinator snapshot

These are the pilot's coordinator scripts, preserved for reproducibility, not
inputs supplied to the code-blind executors. They contain machine-specific
paths and require the private test profile and matching package; no credential
contents are included. Do not run them against a normal user profile.

The active executable copies remain in the external-drive pilot directory
recorded in `../harness-repairs.md`. This snapshot is not a canonical testing
framework or an instruction to recreate live login flows.

`python3 -m unittest test_collection.py` runs the five read-only collector
checks without launching models, reading credentials, or operating applications.
