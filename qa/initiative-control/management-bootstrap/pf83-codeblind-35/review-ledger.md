# PF-83 functional review ledger (campaign 35)

## Prior usage

- 4 of 5 reviews used.
- 1 evidence check reserved.
- Source: `integration-handoff.md`, as cited in the frozen allocation (`normalized/allocation.md`). This is historical usage and is not reset here.

## This campaign

| # | Pass | Agent / thread | Model | Isolation | Result |
|---|---|---|---|---|---|
| 5 | Evidence check r1 | `01a10edf-cf04-7fa0-a62f-3dadab26b556` | claude-opus-5-5-plan, high | Instruction-only, read-only sandbox, record directory only | FAIL: 7 evidence defects. All are corrected in `dispositions.md` and `common/`. |
| 6 | Evidence check r2 (corrective re-check) | `01a10eed-df98-7351-9a06-e6795763dc3f` | claude-opus-5-5-plan, high | Same | FAIL: r1 defects 1–6 fixed, 7 partly; four new representation defects (A–D), all corrected; F10 moved to blocked. |
| 7 | Evidence check r3 (corrective re-check) | `01a10ef8-b3f1-7042-842a-74ff7d3e9190` | claude-opus-5-5-plan, high | Same | **PASS**, six non-blocking follow-ups; 2, 4, 5, 6 applied afterwards as wording/receipt additions. |

## Extension

The checker limit is 5. Passes 6 and 7 exceed it.

- **Who recorded it:** the integrator-side worker, under the September 12 root delegation, which lets the integrator authorize additional allowance.
- **Why:** r1 and r2 failed on correctable evidence and representation defects. r2 and r3 check those corrections. They are not new reviews of the product.
- **Status:** the manager confirms or rejects this extension. It does not turn any failed or blocked case into a pass.

## Not counted here

- **Executor sessions:** separately costed test work. 14 attempts plus 4 aborted, all GLM 5.2.

## Reviewer launch (all three passes)

`~/.local/bin/corbanu exec --json --skip-git-repo-check -s read-only -m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high" -C <record dir> -o review/evidence-check-rN.md "$(cat reviewer-prompt[-rN].md)"`.

- Each pass ran on the host, in a fresh context, with a read-only sandbox.
- Each pass's event log binds its thread ID. The model name comes from this launch command; the event log does not echo it.
