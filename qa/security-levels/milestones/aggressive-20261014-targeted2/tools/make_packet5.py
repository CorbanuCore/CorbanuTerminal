"""Build the second targeted re-run's executor packet (v6): the first targeted run's packet v5 verbatim (frozen cases,
amendments parts 1-5, new cases, screens) plus part 6 (amendment 5 of 2026-10-14), new-screens-5 and drivers/.

usage: python3 tools/make_packet5.py RUN_DIR V5_PACKET_DIR OUT_DIR CANDIDATE_SHA VERSION
"""
import hashlib, json, pathlib, shutil, sys

run, v5, out = (pathlib.Path(a) for a in sys.argv[1:4])
CANDIDATE, VERSION = sys.argv[4], sys.argv[5]
amend = json.loads((run / "design5/amendments5-final.json").read_text())
GROUPS = json.loads((run / "tools/groups5.json").read_text())
NOTES = json.loads((run / "tools/standing-notes5.json").read_text())

out.mkdir(parents=True, exist_ok=True)
shutil.copy2(v5 / "design-cases.md", out / "design-cases.md")
for sub in ("screens", "new-screens", "new-screens-3", "new-screens-4"):
    shutil.copytree(v5 / sub, out / sub, dirs_exist_ok=True)
shutil.copytree(run / "design5/new-screens-5", out / "new-screens-5", dirs_exist_ok=True)
shutil.copytree(run / "design5/drivers", out / "drivers", dirs_exist_ok=True)

def cell(v):
    return [f"  - {v}"] if isinstance(v, str) else [f"  - {e}" for e in v]

L = [(v5 / "amendments-and-new-cases.md").read_text().rstrip("\n"), "", "## Part 6: amendments of 2026-10-14 (second round)", "",
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

executor_prompt = (v5 / "executor-prompt.md").read_text()
navigation = (v5 / "navigation.md").read_text()
navigation = navigation.replace(
    "(amendments to some frozen cases, in three rounds, and new cases\n`AGG-R..` and `AGG-T..`). For an amended case, the latest amendment governs where it differs from earlier text: the\n2026-10-14 amendment (part 5) over the 2026-10-13 one (part 3), and that over the 2026-10-11 one (part 1);",
    "(amendments to some frozen cases, in four rounds, and new cases\n`AGG-R..` and `AGG-T..`). For an amended case, the latest amendment governs where it differs from earlier text: the\nsecond 2026-10-14 amendment (part 6) over the first (part 5), that over the 2026-10-13 one (part 3), and that over\nthe 2026-10-11 one (part 1);")
navigation = navigation.replace("`new-screens/`,\n`new-screens-3/` and `new-screens-4/` hold screens of later candidates.",
                                "`new-screens/`,\n`new-screens-3/`, `new-screens-4/` and `new-screens-5/` (this candidate) hold screens of later candidates.")
assert "part 6" in navigation and "new-screens-5" in navigation, "navigation text not updated"
navigation += """
Where a case's part 6 amendment says its actions are performed by a keystroke driver, start that driver with
`agms-drive start <case id>` (for example `agms-drive start AGG-R14`) and follow it with `agms-drive wait <case id> 50`,
repeated until it reports ENDED. Press no keys in the product and do not start or stop the product while the driver
runs. The driver's log (`agms-drive log <case id>`) shows every step with its UTC time and the screens it waited for;
everything it does is also in your evidence files like your own actions. After it has ended, do the case's own-shell
steps, read the screens it recorded, and judge every expected result yourself. If it stopped early, do not continue
the case by hand: judge what the recorded screens and files decide, and record the rest as blocked with the step
where it stopped.
"""
(out / "navigation.md").write_text(navigation)
(out / "executor-prompt.md").write_text(executor_prompt)
(out / "prompts").mkdir(exist_ok=True)
v5_prompt = (v5 / "prompts/Q01.md").read_text()
env_start = v5_prompt.index("## Environment")
tail = v5_prompt[env_start:]
tail = tail.replace("  - `agms-tui text '...'` types text", "  - `agms-drive` plays a frozen keystroke driver (run `agms-drive` for its usage); only where a case's amendment\n    says so.\n  - `agms-tui text '...'` types text")
assert "agms-drive" in tail
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
