"""Build the Moderate targeted re-run's executor packet (v3): the frozen cases and the first run's amendments v2
verbatim, amendment v3 (reviewed), both screen sets, neutral navigation and one prompt per group.

usage: python3 tools/make_packet3.py RUN_DIR REPO_DIR OUT_DIR CANDIDATE_SHA
"""
import hashlib
import json
import pathlib
import shutil
import sys

run, repo, out = (pathlib.Path(a) for a in sys.argv[1:4])
CANDIDATE = sys.argv[4]
GROUPS = json.loads((run / "tools/groups.json").read_text())
NOTES = json.loads((run / "tools/standing-notes.json").read_text())
REPLAYS = json.loads((run / "tools/replays.json").read_text()) if (run / "tools/replays.json").exists() else {}
first = run / "design/designer-packet"
out.mkdir(parents=True, exist_ok=True)
shutil.copy2(first / "design-cases.md", out / "design-cases.md")
shutil.copy2(first / "amendments-v2.md", out / "amendments.md")
shutil.copy2(run / "design/amendments-v3-final.md", out / "amendments-v3.md")
for sub, dst in (("screens-v1", "screens"), ("screens", "screens-v3")):
    (out / dst).mkdir(exist_ok=True)
    for f in sorted((first / sub).iterdir()):
        shutil.copy2(f, out / dst / f.name)
readme = out / "screens-v3" / "README.md"
readme.write_text(readme.read_text().replace("`screens-v1/` holds the first run's design-time screens",
                                             "`screens/` holds the first run's design-time screens"))
cases_sha = hashlib.sha256((out / "design-cases.md").read_bytes()).hexdigest()
amend_sha = hashlib.sha256((out / "amendments.md").read_bytes()).hexdigest()
v3_sha = hashlib.sha256((out / "amendments-v3.md").read_bytes()).hexdigest()

executor_prompt = (repo / "qa/code-blind-functional/executor-prompt.md").read_text()
navigation = """# Neutral navigation

Read your assigned case headings in `/opt/agms/packet/design-cases.md` (frozen cases), in
`/opt/agms/packet/amendments.md` (the first run's rules for every case, named limits, amendments to some cases, one
new case MOD-34, and the coordinator's answers and test-machine facts) and in `/opt/agms/packet/amendments-v3.md`
(this re-run's amendment, written after the owner's later decisions). Where they differ, `amendments-v3.md` governs,
then `amendments.md`, then the frozen text; everything a later document does not mention stays as before. Cases that
mention a screen by number (for example "screen 13a" or "E2b") mean the first run's design-time screen whose text copy
is in `/opt/agms/packet/screens/` (`README.md` there says what each shows); `amendments-v3.md` may cite screens of the
candidate under test in `/opt/agms/packet/screens-v3/` (its own `README.md`). Use their starting conditions, actions
and expected results; preserve any ambiguity. Run only the cases assigned to you, in the order given.

Each group starts from a fresh product profile and a fresh copy of the test project: you set up every starting
condition yourself in this run ("Moderate (restarted)" and any state a case describes). To reach a command's
approval prompt, ask the product's agent to run it (for example: Run exactly this command: ...). Do not use the
product's `!` shell escape unless a case asks for it.

Send prompt text and Enter as separate actions. Observe the live screen before choosing keys; use the menus,
choices and keys actually displayed. On an approval prompt a letter in brackets only highlights its option; Enter
confirms it. Never type text into an open approval prompt. Use only the synthetic fixture in your run directory.
Record offered cancellation, recovery and continuation paths without inventing missing controls.

Stop a case and record the exact observation on a credential or system password prompt, unexpected profile
access, missing required control, lost observation or exhausted budget. When the product's own model declines a
request, record its words verbatim; you may resubmit the same case wording unchanged once, and label it.
"""
(out / "navigation.md").write_text(navigation)
(out / "executor-prompt.md").write_text(executor_prompt)
(out / "prompts").mkdir(exist_ok=True)
ASSIGN = [(g, ids, NOTES.get(g, "")) for g, ids in GROUPS.items()] + \
         [(label, r["cases"], (NOTES.get(r["group"], "") + " " if NOTES.get(r["group"]) else "") +
           "Replay assignment (fresh executor; earlier attempts are kept separately): " + r["note"])
          for label, r in REPLAYS.items()]
for group, ids, note in ASSIGN:
    note = f"\n- Note for this group: {note}" if note else ""
    prompt = f"""{executor_prompt}
{navigation}
## Assignment: group {group}

- Cases, in order: {', '.join(ids)}.
- Candidate: Corbanu Terminal 0.1.48 package, read-only at `/opt/agms/pkg`; the program is
  `/opt/agms/pkg/bin/corbanu`, expected SHA-256 `{CANDIDATE}`.
- Frozen cases file SHA-256: `{cases_sha}`. Amendments file SHA-256: `{amend_sha}`. Amendment v3 file SHA-256:
  `{v3_sha}`.
- Coordinator isolation preflight for this run: `$AGMS_RUN/evidence/preflight-summary.json` and the two
  `preflight-*.json` files beside it. Before anything else, run
  `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` yourself and check that it reports
  `"ok": true`. If it does not, stop and report every case as blocked.{note}

## Environment

- You run as an unprivileged account in an isolated macOS virtual machine. Your writable area is `$AGMS_RUN`
  (shell variable). Write nothing outside it, except where a case itself has you act as the person on the test
  data (for example creating files in the project, or in your account's home, from your own shell).
- Test data: `$AGMS_RUN/fixture/workspace` is the `ledger-notes` git project (README.md, src/notes.py, notes.txt,
  .env with a synthetic token, .gitignore); `$AGMS_RUN/fixture/outside-notes.txt` is just outside it (the project's
  `../outside-notes.txt`). The product starts in `$AGMS_RUN/fixture/workspace`. The product's home (its settings,
  history, security state) is `$AGMS_RUN/product-home`; its `config.toml` switches the security features on and starts
  at Permissive. The product's `~` is your account's home (`$HOME`), which is wiped after the run.
- Drive the product only through `agms-tui` (run `agms-tui help`). It runs the product in a private terminal and
  records every action and screen:
  - `agms-tui start` launches the product (route `zai`, model `glm-5.3-flash`). Options: `--route zai-anthropic`,
    `--route zai-plain-http` (same models; provider address given as plain HTTP; shown in the product as `zaihttp`),
    `--model glm-5.2` (only where a case names it), `--env NAME=value` (adds a synthetic environment variable; the
    value is made of letters, digits, `.`, `_` or `-`), `--mcp-notes` (adds the fixture MCP server `notes`, see
    `amendments.md`). Anything after the options goes to the product, for example `agms-tui start resume --last` or
    `agms-tui start -c 'key="value"'`.
  - `agms-tui text '...'` types text (no Enter); `agms-tui key Enter` presses keys (tmux names: Enter Escape Up
    Down Tab BSpace y n g i r u C-c ...). `agms-tui screen LABEL` prints the screen; `agms-tui wait REGEX SECONDS`
    waits for visible text; `agms-tui fixture LABEL` records the test data files (credential-like `.env` files by
    size and SHA-256 only); `agms-tui stop` ends the product's terminal session (for a restart, quit in the product
    first when the case says quit, then start). `agms-tui --window B ...` drives a second product window.
  - Model routes provisioned: `zai` (Z.AI OpenAI-style connection), `zai-anthropic` (Z.AI Claude-style connection),
    `zai-plain-http`. No other route has network access.
  - Model replies can take 10-60 seconds; use `wait` with a generous timeout rather than assuming.
  - The relay log is not visible to you; where a case's expected result depends on it, record the UTC times the case
    asks for, and the independent evidence check reads the relay afterwards.
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
print(cases_sha, amend_sha, v3_sha, len(ASSIGN))
