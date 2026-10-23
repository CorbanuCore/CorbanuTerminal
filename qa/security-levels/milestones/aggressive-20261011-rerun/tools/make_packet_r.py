"""Build the re-run executor packet (v3): frozen cases verbatim, amendments + new cases, navigation, prompts.

usage: python3 tools/make_packet_r.py RUN_DIR FIRST_RECORD_DIR OUT_DIR CANDIDATE_SHA
"""
import hashlib
import json
import pathlib
import shutil
import sys

run, first, out, CANDIDATE = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), sys.argv[4]
frozen = json.loads((first / "design.json").read_text())
old_ids = [c["id"] for c in frozen["cases"]]
amend = json.loads((run / "design-r/amendments.json").read_text())
new = json.loads((run / "design-r/new-cases.json").read_text())
GROUPS = json.loads((run / "tools/groups.json").read_text())
NOTES = json.loads((run / "tools/standing-notes.json").read_text())
all_ids = old_ids + [c["id"] for c in new["cases"]]
executed = [i for i in all_ids if i != "AGG-R06"]  # AGG-R06 needs Claude Code, not installed in the guest
assert sorted(sum(GROUPS.values(), [])) == sorted(executed), "every executed case in exactly one group"

out.mkdir(parents=True, exist_ok=True)
shutil.copy2(first / "executor-packet/v2/design-cases.md", out / "design-cases.md")
for sub, src in (("screens", first / "design/packet/screens"), ("new-screens", run / "design-r/packet/new-screens")):
    (out / sub).mkdir(exist_ok=True)
    for txt in sorted(src.glob("*.txt")):
        shutil.copy2(txt, out / sub / txt.name)

def j(x):
    return x if isinstance(x, str) else None

L = ["# Re-run: amendments to the frozen cases, and new cases", "",
     "The frozen cases in `design-cases.md` are unchanged. The product owner's decisions of 2026-10-11 changed what some",
     "of them expect; for those cases the amendment below governs where it differs from the frozen text. New cases",
     "cover behaviour the decisions added. Screen copies of the new candidate are in `new-screens/`.", "",
     "## Amendments", ""]
for r in amend["amendments"]:
    if r.get("no_change"):
        continue
    L += [f"### {r['case']} (amended; decisions {', '.join(r['decisions'])})", ""]
    for k, lab in (("starting_conditions", "Starting conditions"), ("actions", "Actions"), ("expected", "Expected")):
        if k in r["new"]:
            v = r["new"][k]
            L += [f"- {lab}:"] + ([f"  - {v}"] if isinstance(v, str) else [f"  - {e}" for e in v])
    L.append("")
L += ["## New cases", "", "Shorthand S0/S1 as in `design-cases.md`.", ""]
for c in new["cases"]:
    L += [f"### {c['id']} ({c['priority']})", "", f"- Starting conditions: {c['starting_conditions']}", "- Actions:"]
    L += [f"  {n}. {a}" for n, a in enumerate(c["actions"], 1)]
    L += ["- Expected:"] + [f"  - {e}" for e in c["expected"]] + [""]
qa = (run / "design-r/questions-and-answers.md").read_text()
L += ["## Coordinator answers and test-environment facts", "", qa.split("\n", 2)[2]]
(out / "amendments-and-new-cases.md").write_text("\n".join(L))
frozen_sha = hashlib.sha256((out / "design-cases.md").read_bytes()).hexdigest()
amend_sha = hashlib.sha256((out / "amendments-and-new-cases.md").read_bytes()).hexdigest()

executor_prompt = (first.parents[3] / "qa/code-blind-functional/executor-prompt.md").read_text()
navigation = """# Neutral navigation

Read your assigned case headings in `/opt/agms/packet/design-cases.md` (frozen cases) and in
`/opt/agms/packet/amendments-and-new-cases.md` (amendments to some frozen cases, and new cases `AGG-R..`). For an
amended case, the amendment governs where it differs from the frozen text; everything it does not mention stays
as frozen. Cases that mention a screen by number (for example "screen 03" or "E1") mean the design-time screenshot
whose text copy is in `/opt/agms/packet/screens/`; `/opt/agms/packet/new-screens/` holds screens of this candidate.
Use their starting conditions, actions and expected results; preserve any ambiguity. Run only the cases assigned
to you, in the order given.

Send prompt text and Enter as separate actions. Observe the live screen before choosing keys; use the menus,
choices and keys actually displayed. Use only the synthetic fixture in your run directory. Record offered
cancellation, recovery and continuation paths without inventing missing controls.

Stop a case and record the exact observation on a credential or system password prompt, unexpected profile
access, missing required control, lost observation or exhausted budget. When the product's own model declines a
request, record its words verbatim; you may resubmit the same case wording unchanged once, and label it.
"""
(out / "navigation.md").write_text(navigation)
(out / "executor-prompt.md").write_text(executor_prompt)
(out / "prompts").mkdir(exist_ok=True)
for group, ids in GROUPS.items():
    note = NOTES.get(group, "")
    note = f"\n- Note for this group: {note}" if note else ""
    prompt = f"""{executor_prompt}
{navigation}
## Assignment: group {group}

- Cases, in order: {', '.join(ids)}.
- Candidate: Corbanu Terminal 0.1.48 package, read-only at `/opt/agms/pkg`; the program is
  `/opt/agms/pkg/bin/corbanu`, expected SHA-256 `{CANDIDATE}`.
- Frozen cases file SHA-256: `{frozen_sha}`. Amendments and new cases file SHA-256: `{amend_sha}`.
- Coordinator isolation preflight for this run: `$AGMS_RUN/evidence/preflight-summary.json` and the two
  `preflight-*.json` files beside it. Before anything else, run
  `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` yourself and check that it reports
  `"ok": true`. If it does not, stop and report every case as blocked.{note}

## Environment

- You run as an unprivileged account in an isolated macOS virtual machine. Your writable area is `$AGMS_RUN`
  (shell variable). Write nothing outside it, except where a case itself has you act as the person on the test
  data (for example creating files in the project from your own shell).
- Test data: `$AGMS_RUN/fixture/workspace` is the `ledger-notes` git project (README.md, src/notes.py, notes.txt,
  .env with a synthetic token, .gitignore); `$AGMS_RUN/fixture/outside-notes.txt` is just outside it. The product
  starts in `$AGMS_RUN/fixture/workspace`. The product's home (its settings, history, security state) is
  `$AGMS_RUN/product-home`; its `config.toml` switches the security features on and starts at Permissive. The
  product's `~` is your account's home, which is wiped after the run.
- Drive the product only through `agms-tui` (run `agms-tui help`). It runs the product in a private terminal and
  records every action and screen:
  - `agms-tui start` launches the product (route `zai`, model `glm-5.3-flash`). Options: `--route zai-anthropic`,
    `--route zai-plain-http` (same models; provider address given as plain HTTP), `--model glm-5.2` (only where a
    case names it), `--env NAME=value` (adds a synthetic environment variable; the value is made of letters,
    digits, `.`, `_` or `-`). Anything after the options goes to the product, for example
    `agms-tui start resume --last` or `agms-tui start -c 'key="value"'`.
  - `agms-tui text '...'` types text (no Enter); `agms-tui key Enter` presses keys (tmux names: Enter Escape Up
    Down Tab BSpace y n g i r u C-c ...). `agms-tui screen LABEL` prints the screen; `agms-tui wait REGEX SECONDS`
    waits for visible text; `agms-tui fixture LABEL` records the test data files; `agms-tui stop` ends the
    product's terminal session (for a restart, quit in the product first when the case says quit, then start).
  - Model routes provisioned: `zai`, `zai-anthropic`, `zai-plain-http`. No other route has network access.
  - Model replies can take 10-60 seconds; use `wait` with a generous timeout rather than assuming.
- Wall-clock budget: about 80 minutes; the session is terminated at 90 minutes. Keep your report current.

## Report

Write `$AGMS_RUN/evidence/result.md` (update it as you go). For each case: requested versus actual actions;
positive checkpoints with their screen labels and times; the security level the UI shows at each transition;
fixture contents before/after where relevant; every refusal or decline by the product's model, verbatim; and an
outcome for each expected result (passed, failed, blocked, not exercised or inconclusive, with the reason). End
with one line per case: `VERDICT <case id>: passed|failed|blocked` (failed if any expected result failed; blocked
if the case could not be executed). Do not claim human acceptance.
"""
    (out / "prompts" / f"{group}.md").write_text(prompt)
print(frozen_sha, amend_sha, len(GROUPS))
