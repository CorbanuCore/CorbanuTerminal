"""Build the read-only executor packet: frozen cases, executor prompt, neutral navigation, per-case assignment."""
import hashlib
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
FROZEN = pathlib.Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/normalized/original-F01-F11.md")
FROZEN_SHA = "c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726"
EXECUTOR = ROOT / "qa/code-blind-functional/executor-prompt.md"
NAVIGATION = ROOT / "qa/initiative-control/management-bootstrap/pf83-handoff-70/navigation.md"

ASSIGNMENT = """
## Assignment: {case}

- Case: the heading `### {case}` in `/opt/pf83/packet/original-F01-F11.md`. The file's
  "Safe fixture and observation rules" section applies to your case.
- Candidate: Corbanu 0.1.42 package, read-only at `/opt/pf83/pkg`. Expected `corbanu`
  SHA-256 `e552180a2dfc5457a93881cc4822054e18a4a90e8bf86f67008238e461e758f6`.
- Coordinator isolation preflight for this run: `$PF83_RUN/evidence/preflight-summary.json`
  and the two `preflight-*.json` files beside it. Before anything else, run
  `pf83-probe all > $PF83_RUN/evidence/preflight-executor.json; echo rc=$?` yourself and
  check that `"ok": true`. If it is not, stop and report the case as blocked.

## Environment

- You run as an unprivileged account in an isolated guest. Your writable area is
  `$PF83_RUN` (shell variable). Write nothing outside it.
- Fixture: `$PF83_RUN/fixture/` contains `input.txt` (text `permission-test`) and an empty
  `events.txt`. The product starts with working directory `$PF83_RUN/fixture/workspace`.
  Create any disposable output files only inside `$PF83_RUN/fixture/`.
- Drive the product only through `pf83-tui` (run `pf83-tui help`). It runs the product in a
  private terminal and records every action and screen you take:
  - `pf83-tui start` launches the product; `pf83-tui start resume ARGS...` passes ARGS to
    the product's own `resume` command (use this for a restart).
  - Model routes provisioned here: `zai` and `zai-anthropic`, both model `glm-5.2`;
    `pf83-tui start --route zai-anthropic` selects the second at launch. No other route has
    network access or credentials.
  - `pf83-tui text '...'` types text; `pf83-tui key Enter` presses Enter. Send text and
    Enter as separate actions. `pf83-tui screen LABEL` shows the screen; `pf83-tui wait
    REGEX SECONDS` waits for visible text; `pf83-tui fixture LABEL` records fixture contents.
- Wall-clock budget: about 80 minutes; the session is terminated at 90 minutes. Write your
  report before then.

## Report

Write `$PF83_RUN/evidence/result.md` (update it as you go). For each variant you exercise:
requested versus actual actions; positive checkpoints with their screen labels and times;
at each transition the selected level, the level the UI claims is effective, and whether
probe behavior agrees; fixture contents before/after; every refusal by the product's model,
verbatim; and an outcome for each observable result in the case (passed, failed, blocked,
not exercised, or inconclusive, with the reason). End with one line:
`VERDICT {case}: passed|failed|blocked` (failed if any expected observable result failed;
blocked if the case could not be executed). Do not claim human acceptance.
"""


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(out):
    out = pathlib.Path(out)
    (out / "prompts").mkdir(parents=True)
    assert sha(FROZEN) == FROZEN_SHA, "frozen cases changed"
    (out / "original-F01-F11.md").write_bytes(FROZEN.read_bytes())
    (out / "executor-prompt.md").write_bytes(EXECUTOR.read_bytes())
    (out / "navigation.md").write_bytes(NAVIGATION.read_bytes())
    preamble = EXECUTOR.read_text() + "\n" + NAVIGATION.read_text()
    for n in range(1, 12):
        case = "F%02d" % n
        (out / "prompts" / (case + ".md")).write_text(preamble + ASSIGNMENT.format(case=case))
    for path in sorted(out.rglob("*")):
        if path.is_file():
            print(sha(path), path.relative_to(out))


if __name__ == "__main__":
    main(sys.argv[1])
