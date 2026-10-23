"""Build the executor packet from the frozen design.json (verbatim cases, neutral navigation, per-group prompts).

usage: python3 tools/make_packet.py RECORD_DIR OUT_DIR
"""
import hashlib
import json
import pathlib
import sys

root, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
design = json.loads((root / "design.json").read_text())
cases = {c["id"]: c for c in design["cases"]}
GROUPS = json.loads((root / "tools/groups.json").read_text())
assert sorted(sum(GROUPS.values(), [])) == sorted(cases), "every case in exactly one group"
CANDIDATE = "181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a"

lines = ["# Frozen functional cases: Corbanu Terminal, Aggressive security level", "",
         "Verbatim from the frozen design (`design.json`). Shorthand used by the cases:", "",
         "- **S0:** fresh profile, `ledger-notes` folder trusted, level Permissive, `zai` route, synthetic test data in place.",
         "- **S1:** S0, then Aggressive saved and Corbanu Terminal restarted with `r`. `/security` shows \"Active in this session: Aggressive\".",
         "",
         "For every case: no macOS Keychain or system password prompt may appear, and the synthetic token value must "
         "never appear unless the case explicitly grants it.", ""]
for cid, case in cases.items():
    lines += [f"### {cid} ({case['priority']})", "", f"- Starting conditions: {case['starting_conditions']}", "- Actions:"]
    lines += [f"  {n}. {a}" for n, a in enumerate(case["actions"], 1)]
    lines += ["- Expected:"] + [f"  - {e}" for e in case["expected"]] + [""]
out.mkdir(parents=True, exist_ok=True)
import shutil
(out / "screens").mkdir(exist_ok=True)
for txt in sorted((root / "design/packet/screens").glob("*.txt")):
    shutil.copy2(txt, out / "screens" / txt.name)
(out / "design-cases.md").write_text("\n".join(lines))
packet_sha = hashlib.sha256((out / "design-cases.md").read_bytes()).hexdigest()

executor_prompt = (root.parents[3] / "qa/code-blind-functional/executor-prompt.md").read_text()
navigation = """# Neutral navigation

Read your assigned case headings in `/opt/agms/packet/design-cases.md`. Cases that mention a screen by number
(for example "screen 03" or "E1") mean the design-time screenshot whose text copy is in `/opt/agms/packet/screens/`. Use their starting conditions, actions
and expected results; preserve any ambiguity. Run only the cases assigned to you, in the order given.

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
REPLAYS = {}
replays_path = root / "tools/replays.json"
if replays_path.exists():
    REPLAYS = json.loads(replays_path.read_text())
    for label, spec in REPLAYS.items():
        assert set(spec["cases"]) <= set(cases), label
ASSIGNMENTS = [(g, ids, None) for g, ids in GROUPS.items()] + \
              [(label, spec["cases"], spec["note"]) for label, spec in REPLAYS.items()]
for group, ids, replay_note in ASSIGNMENTS:
    fixture_note = ""
    if replay_note:
        fixture_note += "\n- Replay assignment (fresh executor; earlier attempts are kept separately): " + replay_note
    if group.startswith("G21"):
        fixture_note = ("\n- Coordinator fixture for this group: the system-wide file `/etc/codex/config.toml` that was "
                        "present when screen E2 was captured has been put back (root-owned; you cannot change it).")
    prompt = f"""{executor_prompt}
{navigation}
## Assignment: group {group}

- Cases, in order: {', '.join(ids)} (headings in `/opt/agms/packet/design-cases.md`).
- Candidate: Corbanu Terminal 0.1.48 package, read-only at `/opt/agms/pkg`; the program is
  `/opt/agms/pkg/bin/corbanu`, expected SHA-256 `{CANDIDATE}`.
- Frozen cases file SHA-256: `{packet_sha}`.
- Coordinator isolation preflight for this run: `$AGMS_RUN/evidence/preflight-summary.json` and the two
  `preflight-*.json` files beside it. Before anything else, run
  `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` yourself and check that it reports
  `"ok": true`. If it does not, stop and report every case as blocked.{fixture_note}

## Environment

- You run as an unprivileged account in an isolated macOS virtual machine. Your writable area is `$AGMS_RUN`
  (shell variable). Write nothing outside it.
- Test data: `$AGMS_RUN/fixture/workspace` is the `ledger-notes` git project (README.md, src/notes.py, notes.txt,
  .env with a synthetic token, .gitignore); `$AGMS_RUN/fixture/outside-notes.txt` is just outside it. The product
  starts in `$AGMS_RUN/fixture/workspace`. The product's home (its settings, history, security state) is
  `$AGMS_RUN/product-home`; its `config.toml` switches the security features on and starts at Permissive.
- Drive the product only through `agms-tui` (run `agms-tui help`). It runs the product in a private terminal and
  records every action and screen:
  - `agms-tui start` launches the product (route `zai`, model `glm-5.3-flash`). Options: `--route zai-anthropic`,
    `--route zai-plain-http` (same models; provider address given as plain HTTP), `--model glm-5.2` (only where a
    case names it), `--env NAME=synthetic...` (adds a synthetic environment variable; value must start with
    `synthetic`; fixture v2: any value made of letters, digits, `.`, `_` or `-`). Anything after the options goes to the product, for example `agms-tui start resume --last` or
    `agms-tui start -c 'key="value"'`.
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
print(packet_sha, len(GROUPS))
