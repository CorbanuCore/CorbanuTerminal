"""utm-par3 (D139/D140/D141): build executor packet v4d from utm-par2's packet v4c.

usage: make_packet_v4d.py SRC_V4C DST AMENDMENTS_V9 CANDIDATE_SHA256
- adds amendments-v9.md (D140, frozen in its record) and names it in the navigation (navigation.md and every prompt's
  copy of it), with its SHA-256 in the prompts;
- replaces the candidate SHA-256 in the prompts (v4c: 2f9d05c51e; v4d: the installed build 9750861c2a);
- D114 (tester-packet-rules.md): the Environment bullet on checking every 30 seconds becomes the rule's exact tester
  text; the "generous timeout" line points to it; executor-prompt.md (and each prompt's copy of it) gets the D114 lines
  of the current generic executor prompt (qa/code-blind-functional/executor-prompt.md);
- T17's group note names amendments-v9.md.
Everything else is byte-identical to SRC (checked).
"""
import hashlib
import pathlib
import shutil
import sys

OLD_SHA = "a4f9804b3d689650f5176b70c55d5388a55f424a17f3eb0c35be9262dd11c494"
NAV_OLD = ("the owner's latest decisions) and in `/opt/agms/packet/amendments-v6.md` and `/opt/agms/packet/amendments-v7.md`\n"
           "(later wording amendments). Where they differ, `amendments-v7.md` and `amendments-v6.md` govern, then\n")
NAV_NEW = ("the owner's latest decisions), in `/opt/agms/packet/amendments-v6.md` and `/opt/agms/packet/amendments-v7.md`\n"
           "(later wording amendments) and in `/opt/agms/packet/amendments-v9.md` (how MOD-21's task is sent). Where they\n"
           "differ, `amendments-v9.md` governs, then `amendments-v7.md` and `amendments-v6.md`, then\n")
EXEC_ANCHOR = "state/profile and evidence. Missing completion or a timeout is not a pass.\n"
EXEC_D114 = ("While a turn is running, check the screen about every 30 seconds rather than in\n"
             "one long wait; after each check, answer any prompt that has appeared as the case\n"
             "says, and note the time against the case's time limits (D114).\n")
WAIT_OLD = "  - Model replies can take 10-60 seconds; use `wait` with a generous timeout rather than assuming.\n"
WAIT_NEW = ("  - Model replies can take 10-60 seconds; do not assume a reply has come: wait for it, checking as the next point\n"
            "    says.\n")
CHECK30_OLD = ("  - While a product turn is running, look at the screen about every 30 seconds (for example\n"
               "    `agms-tui wait REGEX 30`, and `agms-tui screen LABEL` if it has not appeared yet), so that a prompt or a\n"
               "    question that appears is answered at once. Do not wait blind for several minutes.\n")
CHECK30_NEW = ("  - While a turn is running, check the screen about every 30 seconds: use `agms-tui wait REGEX 30` (or\n"
               "    `agms-tui screen LABEL`) again and again, not one long wait. After each check, answer any prompt that has\n"
               "    appeared, as the case says, and note the time against the case's time limits. A long `wait` sees only the\n"
               "    text it waits for, so it can miss a prompt or run past a time limit.\n")
T17_NOTE_OLD = ("- Note for this group: MOD-21 runs per amendments-v3.md R-1: the product's model is GLM 5.2; launch with "
                "`agms-tui start --model glm-5.2` and repeat `--model glm-5.2` at any relaunch.\n")
T17_NOTE_NEW = T17_NOTE_OLD.rstrip("\n") + " The task is sent from Spawn status, as `amendments-v9.md` says.\n"


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def main():
    src, dst, v9, new_sha = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3], sys.argv[4]
    if dst.exists():
        raise SystemExit(f"{dst} exists")
    shutil.copytree(src, dst)
    shutil.copyfile(v9, dst / "amendments-v9.md")
    e = dst / "executor-prompt.md"
    t = e.read_text()
    assert t.count(EXEC_ANCHOR) == 1
    e.write_text(t.replace(EXEC_ANCHOR, EXEC_ANCHOR + EXEC_D114))
    n = dst / "navigation.md"
    t = n.read_text()
    assert t.count(NAV_OLD) == 1
    nav = t.replace(NAV_OLD, NAV_NEW)
    n.write_text(nav)
    v7line = "Amendment v7 file SHA-256: `%s`." % sha(dst / "amendments-v7.md")
    for p in sorted((dst / "prompts").glob("*.md")):
        t = p.read_text()
        for old in (NAV_OLD, OLD_SHA, EXEC_ANCHOR, WAIT_OLD, CHECK30_OLD, v7line):
            assert t.count(old) == 1, (p, old[:40])
        t = (t.replace(NAV_OLD, NAV_NEW).replace(OLD_SHA, new_sha).replace(EXEC_ANCHOR, EXEC_ANCHOR + EXEC_D114)
             .replace(WAIT_OLD, WAIT_NEW).replace(CHECK30_OLD, CHECK30_NEW)
             .replace(v7line, v7line + " Amendment v9 file SHA-256: `%s`." % sha(dst / "amendments-v9.md")))
        if p.name == "T17.md":
            assert t.count(T17_NOTE_OLD) == 1
            t = t.replace(T17_NOTE_OLD, T17_NOTE_NEW)
        # The prompt's executor and navigation sections stay identical to the packet's two files.
        assert t.startswith(e.read_text().rstrip("\n") + "\n\n" + nav.rstrip("\n") + "\n"), p
        p.write_text(t)
    for f in src.rglob("*"):
        rel = f.relative_to(src)
        if f.is_file() and "prompts" not in rel.parts and rel.name not in ("executor-prompt.md", "navigation.md"):
            assert sha(f) == sha(dst / rel), f
    print("packet", dst, "from", src)


main()
