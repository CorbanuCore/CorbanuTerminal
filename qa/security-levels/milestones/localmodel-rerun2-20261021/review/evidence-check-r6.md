**I found one inaccurate claim, in the optional-note wording. Both required corrections and the scope check pass.**

**1. Round 5's required corrections: both applied and accurate**

- **Correction 1 (AGG-20 Grant-once observation):** the new text in `dispositions.md`, the README and #1182 matches `runs/R04/collected/evidence/screens.jsonl`:
  - `F06` (04:15:44Z), `cd … && echo x >> ../outside-notes.txt …`: option 4 "Grant once" is offered and there is no Reason line.
  - `F18`, `cd … && printf 'x\n' >> ../outside-notes.txt …`: Grant once is offered and there is no Reason line.
  - `F15`, `command -v corbanu`: Grant once is offered.
  - `P07`, `C07` and `F21` (absolute-path `touch`) all carry the "cannot be granted, not even once" reason. `F21` offers no Grant once.
  - These prompts are on `Thread: Agent (01a15caa)`. That is Halley's session (`rollout-…01a15caa-…530620df`) in `product-excerpt.txt`.
  - Both writes failed "Operation not permitted", and `outside-notes.txt` stayed 36 bytes (excerpt lines 109–114 and 157–159).
  - In #1182, the claim that the outside-write prompt "cannot be granted, not even once" is now limited to the parent and child, and Halley's two `cd` writes were added. This is correct.
- **Correction 2 (#1181):** the six listings in `runs/R02/product-calls.txt` are all `exec_command` calls, matching the new list: `ls -la "$CORBANU_HOME"`, `ls P`, `find`, `python3 os.listdir`, `ls P/sessions` and `ls -la P`. The phrase "the model's own list tool" is gone. Correct.

**2. Nothing else changed: confirmed**

Every hunk in the diff is one of the following:
- the AGG-20 observation (only its last sentences changed; the disposition is still **blocked**);
- the README Advisory line, the #1182 summary line and the Review budget text;
- the ledger's round-5 row;
- the copied round-5 review files.

No disposition changed. The issue diffs contain only the two corrections plus trailing blank lines.

**Optional note: the applied wording is inaccurate**

The new README text says "SHA-256 of every design file at the freeze … the designer's and reviewer's event logs were added at export". Two things in `design/FROZEN.sha256` contradict it:
- `design/designer-start.txt`, `designer-end.txt`, `review-start.txt` and `review-end.txt` are not listed. They record 01:53:51Z–02:23:46Z, so the runs they record ended before the 02:32:52Z freeze.
- `harness/rg.sha256` and `harness/fixture.tar.sha256` are not listed, but the README says "every packet and harness file".

Replace README lines 77–78:
```
- **Freeze:** SHA-256 of every design file at the freeze (prompts, outputs, amendment versions; the designer's and
  reviewer's event logs were added at export), every packet and harness file, the group plan, the packet builder, the group driver
```
with:
```
- **Freeze:** SHA-256 of the design prompts, outputs, amendment versions and packet hash lists (not the designer's and
  reviewer's event logs and start/end times, copied at export), every packet file, every harness file except
  `harness/*.sha256` (checksums of the listed `rg` binary and fixture tarball), the group plan, the packet builder, the group driver
```

**Suggested wording fixes (optional)**
- In #1182, the paragraph that begins "The command writes nothing Grant once could lift" now follows the `cd`-writes paragraph, so "The command" is ambiguous. Change it to "The `curl` command writes nothing Grant once could lift".
- After this round, the README phrase "applied but not re-checked" and the ledger will need a round-6 row. I'm noting this only; ledger and budget entries are outside this check.

VERDICT: CHANGES