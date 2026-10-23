**Shipped**

- **PR #395** — docs-only acceptance record, **merged** into `main`.
- **Issue #389** — commented and **closed** with verdict **PASS WITH LIMITS**.

**Gate evidence**

- Built `corbanu.exe` from origin/main `b1e20a8ec6192ea16a4b3c7e730d28cb712d3707` on the real Windows 11 machine.
- Console-session proof: scheduled tasks and child processes ran in **console session 1**, not SSH/services session 0.
- Vault seeded via Credential Manager; no `ZAI_API_KEY` in the brokered run.
- GLM 5.3 Flash answered through the broker with `broker_model_auth` on.
- Positive control found **11** raw-key hits; brokered run found **0**.
- Flag off remained unchanged.
- Memory scanner was independently compiled and used both UTF-8 and UTF-16LE matching.

**Cleanup**

- Removed all Windows dirs, scripts, logs, scheduled tasks, raw key files, and the run-created Credential Manager entry.
- Left shared `C:\CorbanuQA\target` in place per brief.

**Open**

- None for this follow-up.