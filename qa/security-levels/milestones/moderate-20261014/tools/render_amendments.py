"""Render the amendment table JSON as Markdown. usage: render_amendments.py IN.json OUT.md"""
import json
import sys

d = json.load(open(sys.argv[1]))
L = ["# Amendments to the Moderate design (" + sys.argv[1].rsplit("/", 1)[-1].replace(".json", "") + ")", "", d["about"], "",
     "## Rules for every case", ""]
for g in d["global"]:
    L.append(f"- **{g['id']}** ({', '.join(g['decisions'])}): {g['text']}")
L += ["", "## Named limits of this run", ""] + [f"- {x}" for x in d["named_limits"]] + ["", "## Case amendments", ""]


def lst(v):
    return [f"  - {x}" for x in v] if isinstance(v, list) else [f"  - {v}"]


for a in d["amendments"]:
    if a["kind"] == "new":
        L += [f"### {a['case']} (new, {a['priority']}; {', '.join(a['decisions'])}): {a['title']}", "",
              f"- Starting conditions: {a['starting_conditions']}", "- Actions:"]
        L += [f"  {n}. {x}" for n, x in enumerate(a["actions"], 1)]
        L += ["- Expected:"] + lst(a["expected"]) + [""]
        continue
    n = a["new"]
    L += [f"### {a['case']} (amended; {', '.join(a['decisions'])})", ""]
    if "starting_conditions" in n:
        L.append(f"- Starting conditions (replace): {n['starting_conditions']}")
    if "starting_conditions_note" in n:
        L.append(f"- Starting conditions (add): {n['starting_conditions_note']}")
    for k, v in n.get("actions_replaces", {}).items():
        L.append(f"- Action {k} (replace): {v}")
    for k, v in n.get("actions_add_after", {}).items():
        L.append(f"- New action after action {k}: {v}")
    for old, new in n.get("expected_replaces", {}).items():
        L.append(f"- Expected (replace) \"{old}\" with: {new}")
    for v in n.get("expected_add", []):
        L.append(f"- Expected (add): {v}")
    if "note" in n:
        L.append(f"- Note: {n['note']}")
    L.append("")
open(sys.argv[2], "w").write("\n".join(L))
