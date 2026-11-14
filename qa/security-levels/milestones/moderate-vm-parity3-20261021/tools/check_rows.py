"""utm-par3 integrator check of MOD-22's v6 comparison: the approval, sandbox and network rows of the last 'baseline'
screen and of the final relaunch's 'after-relaunch' screens (those recorded from the last 'after-relaunch-picker' on),
text compared after joining wrapped lines.   usage: check_rows.py RUN_DIR..."""
import json, re, sys, tarfile


def rows(txt):
    txt = re.sub(r"\n\s+", " ", txt)
    out = {}
    for k in ("Approvals", "Sandbox", "Network"):
        m = re.search(r"\[set\] " + k + r":(.*?)\[resolved", txt, re.S)
        out[k] = m.group(1).strip() if m else None
    return out


for run in sys.argv[1:]:
    with tarfile.open(run + "/collected.tar") as t:
        S = [json.loads(l) for l in t.extractfile("evidence/screens.jsonl").read().decode().splitlines()]
    b = [d for d in S if d["label"] in ("baseline", "mod22-valid-baseline")][-1]
    pk = [d for d in S if d["label"] == "after-relaunch-picker"][-1]["utc"]
    fin = [d for d in S if re.fullmatch(r"after-relaunch(-\d)?", d["label"]) and d["utc"] >= pk]
    B, A = rows(b["screen"]), rows("\n".join(d["screen"] for d in fin))
    print(json.dumps({"run": run.rstrip("/").split("/")[-1], "baseline": [b["label"], b["utc"]],
                      "after_relaunch": [[d["label"], d["utc"]] for d in fin],
                      "match": {k: B[k] is not None and B[k] == A[k] for k in B}}))
