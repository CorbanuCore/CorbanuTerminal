"""Build the PF-60-S10 executor packet from the frozen design.
usage: python3 make_packet.py W2 OUT CANDIDATE_SHA VERSION
Inputs (under W2): design/designer-output.md (frozen cases, verbatim), design/amendments-final.md (review
amendments as applied), design/answers-final.md (environment facts), design/packet/screens/,
tools/groups.json ({group: [case ids]}), tools/group-notes.json ({group: note}), tools/navigation.md,
the repository's qa/code-blind-functional/executor-prompt.md (copied to tools/executor-prompt.md).
"""
import hashlib, json, pathlib, shutil, sys

w2, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CANDIDATE, VERSION = sys.argv[3], sys.argv[4]
groups = json.loads((w2 / "tools/groups.json").read_text())
notes = json.loads((w2 / "tools/group-notes.json").read_text())
out.mkdir(parents=True, exist_ok=True)
shutil.copy2(w2 / "design/designer-output.md", out / "design-cases.md")
shutil.copy2(w2 / "design/amendments-final.md", out / "amendments.md")
shutil.copy2(w2 / "design/answers-final.md", out / "answers.md")
shutil.copytree(w2 / "design/packet/screens", out / "screens", dirs_exist_ok=True)
executor_prompt = (w2 / "tools/executor-prompt.md").read_text()
navigation = (w2 / "tools/navigation.md").read_text()
import os
if os.environ.get("NAV_NOTE"):
    navigation += pathlib.Path(os.environ["NAV_NOTE"]).read_text()
(out / "executor-prompt.md").write_text(executor_prompt)
(out / "navigation.md").write_text(navigation)
sha = {n: hashlib.sha256((out / n).read_bytes()).hexdigest() for n in ("design-cases.md", "amendments.md", "answers.md")}

ENV = """## Environment

- You run as an unprivileged account in an isolated macOS virtual machine. Your writable area is `$AGMS_RUN`
  (shell variable). Write nothing outside it, except where a case itself has you act as the person on the test
  data (for example editing a file in the project).
- Test data: `$AGMS_RUN/fixture/workspace` is a small git project (`ledger-notes`: README.md, src/notes.py,
  notes.txt, .gitignore and a .env with a synthetic value). The product starts there. The product's home (settings,
  conversations, its records) is `$AGMS_RUN/product-home`; the product's `~` is your account's home, wiped after
  the run.
- Drive the product only through `agms-tui` (run `agms-tui help` first: routes, models and every control are
  listed there). It runs the product in a private terminal and records every action and screen:
  - `agms-tui start [--route R] [--model M] [--provider-name N] [PRODUCT ARGS]` starts the product (route `zai`,
    model `glm-5.3-flash` by default), e.g. `agms-tui start --route openai --model gpt-6-luna resume --last` or
    `agms-tui start -c 'skip_reopen_warmup=true'`.
  - `agms-tui text '...'` types text (no Enter); `agms-tui key Enter` presses keys (tmux names: Enter Escape Up
    Down Left Right Tab BSpace C-c ...). `agms-tui screen LABEL` prints the screen; `agms-tui wait REGEX SECONDS`
    waits for visible text; `agms-tui fixture LABEL` records the project files; `agms-tui stop` ends the product's
    terminal session (to restart on a conversation, stop, then start with `resume --last` or `resume <id>`).
  - `agms-tui exec [--route R] [--model M] [--ephemeral] 'prompt'` runs `corbanu exec` once on the same profile
    and project; `agms-tui config 'key = value'` appends a line to the profile's config.toml;
    `agms-tui standin chatgpt-image` and `agms-tui standin no-charge on|off` control the stand-ins (see answers.md).
  - Network: only the relay for the listed routes. Real routes: `zai`, `openai`, `moonshot`. Stand-ins:
    `chatgpt`, `openrouter`, `vercel`, `corbanu`, `custom`.
- The relay and stand-in logs are not visible to you; the independent evidence check reads them afterwards. Record
  UTC times of the turns you send.
- Wall-clock budget: about 80 minutes; the session is terminated at 90 minutes. Keep your report current.

## Report

Write `$AGMS_RUN/evidence/result.md` (update it as you go). For each case: requested versus actual actions; the
`/cost` lines you judged, copied exactly, with their screen labels and UTC times; your recomputation; every refusal
or decline by the product's model, verbatim; and an outcome for each expected result (passed, failed, blocked, not
exercised or inconclusive, with the reason). End with one line per case:
`VERDICT <case id>: passed|failed|blocked` (failed if any expected result failed; blocked if the case could not be
executed). Do not claim human acceptance.
"""

(out / "prompts").mkdir(exist_ok=True)
for group, ids in groups.items():
    note = notes.get(group, "")
    note = f"\n- Note for this group: {note}" if note else ""
    prompt = f"""{executor_prompt}
{navigation}
## Assignment: group {group}
- Cases, in order: {', '.join(ids)}.
- Candidate: Corbanu Terminal {VERSION} developer build, read-only at `/opt/agms/pkg`; the program is
  `/opt/agms/pkg/bin/corbanu`, expected SHA-256 `{CANDIDATE}`.
- Frozen design SHA-256 `{sha['design-cases.md']}`; amendments `{sha['amendments.md']}`; environment answers
  `{sha['answers.md']}`.
- Coordinator isolation preflight for this run: `$AGMS_RUN/evidence/preflight-summary.json` and the two
  `preflight-*.json` files beside it. Before anything else, run
  `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` yourself and check that it reports
  `"ok": true`. If it does not, stop and report every case as blocked.{note}

{ENV}"""
    (out / "prompts" / f"{group}.md").write_text(prompt)
print(json.dumps(sha), len(groups))
