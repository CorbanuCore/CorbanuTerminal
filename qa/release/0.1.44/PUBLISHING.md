# 0.1.44 publication run

Authorized release source: `9af80015fee2a981f7de4f15cadecd5c00f0d241`.

GitHub Actions: https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/35796356668

Inputs: release_version=0.1.44, publish_release=true, make_latest=true, no artifact reuse. Validation passed; all five macOS/Linux/Windows package jobs started. The workflow creates the release and marks it latest after successful packaging/smoke checks. At this record's creation, publication is **in progress**, not confirmed complete.

Final versioned focused tests on the exact source passed: **433/433**, 6412 intentionally filtered out. Includes protocol/models/provider libraries, exact new request routes and reasoning, saved-effort persistence, Z.AI preserved tool reasoning, selector snapshots, crew defaults, and carried-forward configured-credential replacement. Local log: /home/pfrpc/corbanu-debug-evidence/release44-tests.log. Installer contract 5/5 and portable skills 25/25 also passed; Bazel lock refresh and diff checks passed.

Integration PR: https://github.com/CorbanuCore/CorbanuTerminal/pull/123. Carries published 0.1.43 forward plus current main; PR122 had not been merged. This documentation-only follow-up does not change the source being built by the run above.

Disclosed cancellation-recovery failure and qualification/benchmark gaps remain as recorded in RELEASE.md. No public release success is claimed until the workflow and release assets confirm it.
