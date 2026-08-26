# GLM 5.3 Flash qualification

Date: 2026-08-26

Candidate: `5da7926796` (`feat(models): enable GLM 5.3 Flash on Z.AI plan`)

## Scope and authority

Change class: bounded fix to the already-shipping multi-provider model catalog.

Product citation: `docs/corbanu-product-spec.md`, **Shipping MVP — LIVE**:
“Multi-provider inference” includes Z.AI.

The catalog identity and capability decisions were checked against the official
[Z.AI GLM-5.3-Flash guide](https://docs.z.ai/guides/vlm/glm-5.3-flash),
[Z.AI Coding Plan overview](https://docs.z.ai/devpack/overview), and
[Z.AI open-weights card](https://huggingface.co/zai-org/GLM-5.3-Flash). The
enabled model code is `glm-5.3-flash`; its catalog advertises the documented
1,000,000-token context, native image input, required thinking, `low`/`high`/
`max` reasoning levels with `max` as the recommended default, and three times
the GLM-5.3 plan quota.

## Automated evidence

- `just test -p codex-model-provider-info`: 57 passed.
- `just test -p codex-models-manager`: 61 passed.
- `just test -p codex-tui model_selection_popup_zai_provider_includes_glm_5_3_flash_snapshot`:
  1 passed; the snapshot shows the model in the Z.AI plan tab and the test
  verifies the selected provider/model pair.
- `cargo build -p codex-cli --bin corbanu-debug`: passed.
- `python3 docs/plans/check.py`: passed; 2/2 active plan slots.
- `python3 docs/sprints/check.py`: passed; 21 current and 82 archived sprints.
- `git diff --check`: passed.
- MkDocs was not installed on the qualification host, so a local site build was
  not available.

## Interactive evidence

The committed candidate was built and the machine-local `corbanu-debug`
launcher was pointed at that binary. A private tmux server launched the real
application at 120x40 with:

```text
RUST_LOG=trace corbanu-debug --yolo --no-alt-screen -c log_dir=<temporary-log-directory>
```

The run used literal input and Enter as separate tmux events. In `/model`, the
Z.AI Coding Plan tab listed `Z.AI GLM 5.3 Flash`; selecting it and its advanced
reasoning option produced `Model changed to Z.AI GLM 5.3 Flash via Z.AI max`.
A real plan-backed request, `Reply with exactly: GLM53_FLASH_OK`, returned
`GLM53_FLASH_OK`. `/status` reported:

- model: `Z.AI GLM 5.3 Flash`;
- provider: `Z.AI - https://api.z.ai/api/coding/paas/v4`;
- context capacity: 950K usable tokens after the catalog safety margin;
- permissions: Full Access, matching `--yolo`.

The terminal status line measured 2.9 output tokens/second for this deliberately
tiny max-reasoning response. This is a connectivity and routing proof, not a
throughput benchmark. The session exited cleanly through `/exit`.

No credential value, provider authorization header, raw trace log, account
identifier, or session identifier is included in this evidence record. The
temporary trace directory was excluded from version control and removed after
the run.

Host tmux `history-limit` was already configured to `10000000` and was left at
that value.
