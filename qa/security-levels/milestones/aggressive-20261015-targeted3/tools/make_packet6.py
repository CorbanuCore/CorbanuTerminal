"""Build the third targeted re-run's executor packet (v7): the second targeted run's packet v6 verbatim (frozen cases,
amendments parts 1-6, new cases, screens) plus part 7 (amendment 6 of 2026-10-15), new-screens-6 and drivers/ (AGG-R03
replaced by amendment 6's driver; AGG-R14 byte-for-byte unchanged).
usage: python3 tools/make_packet6.py RUN_DIR V6_PACKET_DIR OUT_DIR CANDIDATE_SHA VERSION
"""
import hashlib, json, pathlib, shutil, sys
run, v6, out = (pathlib.Path(a) for a in sys.argv[1:4])
CANDIDATE, VERSION = sys.argv[4], sys.argv[5]
amend = json.loads((run / "design6/amendments6-final.json").read_text())
GROUPS = json.loads((run / "tools/groups6.json").read_text())
NOTES = json.loads((run / "tools/standing-notes6.json").read_text())
out.mkdir(parents=True, exist_ok=True)
shutil.copy2(v6 / "design-cases.md", out / "design-cases.md")
for sub in ("screens", "new-screens", "new-screens-3", "new-screens-4", "new-screens-5"):
    shutil.copytree(v6 / sub, out / sub, dirs_exist_ok=True)
shutil.copytree(run / "design6/new-screens-6", out / "new-screens-6", dirs_exist_ok=True)
(out / "drivers").mkdir(exist_ok=True)
shutil.copy2(v6 / "drivers/AGG-R14.json", out / "drivers/AGG-R14.json")
shutil.copy2(run / "design6/drivers/AGG-R03.json", out / "drivers/AGG-R03.json")
assert (out / "drivers/AGG-R14.json").read_bytes() == (v6 / "drivers/AGG-R14.json").read_bytes()
def cell(v):
    return [f"  - {v}"] if isinstance(v, str) else [f"  - {e}" for e in v]
L = [(v6 / "amendments-and-new-cases.md").read_text().rstrip("\n"), "", "## Part 7: amendments of 2026-10-15", "",
     amend["packet_intro"], ""]
for r in amend["amendments"]:
    L += [f"### {r['case']} (amended; decisions {'; '.join(r['decisions'])})", ""]
    for k, lab in (("starting_conditions", "Starting conditions"), ("start_and_actions", "Starting conditions and actions"),
                   ("actions", "Actions"), ("expected", "Expected")):
        if k in r["new"]:
            L += [f"- {lab}:"] + cell(r["new"][k])
    L.append("")
(out / "amendments-and-new-cases.md").write_text("\n".join(L) + "\n")
frozen_sha = hashlib.sha256((out / "design-cases.md").read_bytes()).hexdigest()
amend_sha = hashlib.sha256((out / "amendments-and-new-cases.md").read_bytes()).hexdigest()
drivers = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((out / "drivers").glob("*.json"))}
executor_prompt = (v6 / "executor-prompt.md").read_text()
navigation = (v6 / "navigation.md").read_text()
def rep(s, a, b):
    assert a in s, a[:60]
    return s.replace(a, b)
navigation = rep(navigation,
    "(amendments to some frozen cases, in four rounds, and new cases\n`AGG-R..` and `AGG-T..`). For an amended case, the latest amendment governs where it differs from earlier text: the\nsecond 2026-10-14 amendment (part 6) over the first (part 5), that over the 2026-10-13 one (part 3), and that over\nthe 2026-10-11 one (part 1);",
    "(amendments to some frozen cases, in five rounds, and new cases\n`AGG-R..` and `AGG-T..`). For an amended case, the latest amendment governs where it differs from earlier text: the\n2026-10-15 amendment (part 7) over the second 2026-10-14 one (part 6), that over the first (part 5), that over the\n2026-10-13 one (part 3), and that over the 2026-10-11 one (part 1);")
navigation = rep(navigation, "`new-screens-4/` and `new-screens-5/` (this candidate) hold screens of later candidates.",
                 "`new-screens-4/`, `new-screens-5/` and `new-screens-6/` (this candidate) hold screens of later candidates.")
navigation = rep(navigation, "Where a case's part 6 amendment says its actions are performed by a keystroke driver,",
                 "Where a case's part 6 or part 7 amendment says its actions are performed by a keystroke driver,")
navigation += """
A case may use a second Corbanu Terminal on the same product home: `agms-tui --term 2 ...` drives it exactly like the
first (`agms-tui --term 2 start`, `screen`, `wait`, ...), and `agms-tui --term 2 stop` ends only the second. Its raw
output is in `evidence/pty-2.raw`; its actions and screens are in the same evidence files, marked `"term": 2`.
"""
(out / "navigation.md").write_text(navigation)
(out / "executor-prompt.md").write_text(executor_prompt)
(out / "prompts").mkdir(exist_ok=True)
v6_prompt = (v6 / "prompts/V03.md").read_text()
tail = v6_prompt[v6_prompt.index("## Environment"):]
tail = rep(tail, "  - `agms-drive` plays a frozen keystroke driver",
           "  - `agms-tui --term 2 ...` drives a second Corbanu Terminal on the same product home (only where a case says so).\n  - `agms-drive` plays a frozen keystroke driver")
for group, ids in GROUPS.items():
    note = NOTES.get(group, "")
    note = f"\n- Note for this group: {note}" if note else ""
    drv = [f"`drivers/{c}.json` SHA-256 `{drivers[c + '.json']}`" for c in ids if c + ".json" in drivers]
    drv = ("\n- Keystroke driver for this group: " + ", ".join(drv) + ".") if drv else ""
    prompt = f"""{executor_prompt}
{navigation}
## Assignment: group {group}
- Cases, in order: {', '.join(ids)}.
- Candidate: Corbanu Terminal {VERSION} package, read-only at `/opt/agms/pkg`; the program is
  `/opt/agms/pkg/bin/corbanu`, expected SHA-256 `{CANDIDATE}`.
- Frozen cases file SHA-256: `{frozen_sha}`. Amendments and new cases file SHA-256: `{amend_sha}`.{drv}
- Coordinator isolation preflight for this run: `$AGMS_RUN/evidence/preflight-summary.json` and the two
  `preflight-*.json` files beside it. Before anything else, run
  `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` yourself and check that it reports
  `"ok": true`. If it does not, stop and report every case as blocked.{note}
{tail}"""
    (out / "prompts" / f"{group}.md").write_text(prompt)
print(frozen_sha, amend_sha, json.dumps(drivers), len(GROUPS))
