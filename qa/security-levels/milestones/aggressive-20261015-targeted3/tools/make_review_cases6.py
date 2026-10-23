"""Build design6/review-packet/cases.md: effective text (verbatim sections) of the cases amendment 6 touches."""
import re, sys, pathlib
v6 = pathlib.Path(sys.argv[1]); out = pathlib.Path(sys.argv[2])
CASES = ["AGG-04", "AGG-37", "AGG-T01", "AGG-T02", "AGG-R03", "AGG-R14"]
def sections(text):
    part, cur, res = "frozen cases (2026-10-10)", None, []
    for line in text.splitlines():
        if line.startswith("## "):
            if cur: res.append(cur); cur = None
            part = line[3:].strip()
            if part.startswith("Coordinator answers"):
                cur = ("COORD", part, [line]); continue
        elif line.startswith("### "):
            if cur: res.append(cur)
            m = re.match(r"### (AGG-[A-Z0-9]+)\b", line)
            cur = (m.group(1) if m else None, part, [line]); continue
        if cur: cur[2].append(line)
    if cur: res.append(cur)
    return res
frozen = sections((v6 / "design-cases.md").read_text())
amend = sections((v6 / "amendments-and-new-cases.md").read_text())
hdr = pathlib.Path(sys.argv[3]).read_text().split("## AGG-04")[0]
L = ["# Effective text of the cases amendment 6 touches (verbatim)", "",
     "Verbatim from the second targeted run's executor packet v6 (`design-cases.md`, frozen 2026-10-10, and",
     "`amendments-and-new-cases.md`, parts 1-6). For an amended case the latest part governs where it differs.",
     "Shorthand used by the cases:", ""]
L += [l for l in hdr.splitlines()[3:]]
for c in CASES:
    L += ["", f"## {c}", ""]
    for cid, part, body in frozen + amend:
        if cid == c:
            L += [f"From: {part}", ""] + body + [""]
L += ["", "## Coordinator answers and test-environment facts (verbatim)", ""]
for cid, part, body in amend:
    if cid == "COORD":
        L += body + [""]
out.write_text("\n".join(L).rstrip() + "\n")
print(out, len(L))
