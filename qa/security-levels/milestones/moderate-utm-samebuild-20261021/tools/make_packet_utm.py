"""utm-par2: build an executor packet from the Tart parity packet v4p.

usage: make_packet_utm.py SRC DST [--v6v7 V6 V7 SHA256]
- Always: every file byte-identical to SRC except the group prompts, which get one added line after
  "Model replies can take 10-60 seconds ..." (the brief's 30-second check, the qwen-rerun2 lesson).
- --v6v7 (D94/D95 re-run on Tart with the current build): also adds amendments-v6.md and amendments-v7.md, names
  them in the prompts' navigation and SHA-256 lines, adds a scope sentence about amendments-v5, and replaces the
  candidate SHA-256 in the prompts.
"""
import hashlib
import pathlib
import shutil
import sys

ANCHOR = "  - Model replies can take 10-60 seconds; use `wait` with a generous timeout rather than assuming.\n"
CHECK30 = ("  - While a product turn is running, look at the screen about every 30 seconds (for example\n"
           "    `agms-tui wait REGEX 30`, and `agms-tui screen LABEL` if it has not appeared yet), so that a prompt or a\n"
           "    question that appears is answered at once. Do not wait blind for several minutes.\n")
OLD_SHA = "15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6"
NAV_OLD = ("(the first re-run's amendment) and in `/opt/agms/packet/amendments-v4.md` (the second re-run's amendment, written after\n"
           "the owner's latest decisions). Where they differ, `amendments-v4.md` governs, then `amendments-v3.md`, then\n"
           "`amendments.md`, then the frozen text;")
NAV_NEW = ("(the first re-run's amendment), in `/opt/agms/packet/amendments-v4.md` (the second re-run's amendment, written after\n"
           "the owner's latest decisions) and in `/opt/agms/packet/amendments-v6.md` and `/opt/agms/packet/amendments-v7.md`\n"
           "(later wording amendments). Where they differ, `amendments-v7.md` and `amendments-v6.md` govern, then\n"
           "`amendments-v4.md`, then `amendments-v3.md`, then `amendments.md`, then the frozen text;")
V5_SCOPE = (" `amendments-v5.md`, which `amendments-v6.md` and `amendments-v7.md` mention, belongs to a separate local-model\n"
            "re-run and is not part of this packet; it does not apply here (MOD-21 runs per `amendments-v3.md` R-1, as the group\n"
            "note says).")
SCOPE_ANCHOR = "MOD-06's git-commit part stays not run, as the MOD-06 amendment in `amendments-v3.md` says."


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def main():
    src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    v6v7 = "--v6v7" in sys.argv
    if dst.exists():
        raise SystemExit(f"{dst} exists")
    shutil.copytree(src, dst)
    if v6v7:
        i = sys.argv.index("--v6v7")
        v6, v7, new_sha = sys.argv[i + 1], sys.argv[i + 2], sys.argv[i + 3]
        shutil.copyfile(v6, dst / "amendments-v6.md")
        shutil.copyfile(v7, dst / "amendments-v7.md")
    for p in sorted((dst / "prompts").glob("*.md")):
        t = p.read_text()
        assert t.count(ANCHOR) == 1, p
        t = t.replace(ANCHOR, ANCHOR + CHECK30)
        if v6v7:
            assert t.count(NAV_OLD) == 1 and t.count(SCOPE_ANCHOR) == 1 and t.count(OLD_SHA) == 1, p
            t = t.replace(NAV_OLD, NAV_NEW).replace(SCOPE_ANCHOR, SCOPE_ANCHOR + V5_SCOPE).replace(OLD_SHA, new_sha)
            v4line = "Amendment v4 file SHA-256: `%s`." % sha(dst / "amendments-v4.md")
            assert t.count(v4line) == 1, p
            t = t.replace(v4line, v4line + " Amendment v6 file SHA-256: `%s`. Amendment v7 file SHA-256: `%s`."
                          % (sha(dst / "amendments-v6.md"), sha(dst / "amendments-v7.md")))
        p.write_text(t)
    # Everything outside prompts/ (and the two added amendments) is byte-identical to SRC.
    for f in src.rglob("*"):
        if f.is_file() and "prompts" not in f.relative_to(src).parts:
            assert sha(f) == sha(dst / f.relative_to(src)), f
    print("packet", dst, "from", src, "v6v7" if v6v7 else "")


main()
