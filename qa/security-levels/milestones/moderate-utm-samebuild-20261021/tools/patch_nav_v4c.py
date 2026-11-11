"""utm-par2 (D94/D95 packet v4c): give navigation.md the same navigation and amendments-v5 scope changes that
make_packet_utm.py --v6v7 made in the prompts' copy of it, so the two copies stay identical.  usage: patch_nav_v4c.py PACKET"""
import pathlib
import sys

sys.argv = [sys.argv[0], "-", "-"] + sys.argv[1:]
ns = {}
exec(compile(pathlib.Path(__file__).with_name("make_packet_utm.py").read_text().replace("\nmain()\n", "\n"), "m", "exec"), ns)
pkt = pathlib.Path(sys.argv[3])
nav = pkt / "navigation.md"
t = nav.read_text()
assert t.count(ns["NAV_OLD"]) == 1 and t.count(ns["SCOPE_ANCHOR"]) == 1
t = t.replace(ns["NAV_OLD"], ns["NAV_NEW"]).replace(ns["SCOPE_ANCHOR"], ns["SCOPE_ANCHOR"] + ns["V5_SCOPE"])
nav.write_text(t)
for p in sorted((pkt / "prompts").glob("*.md")):
    body = p.read_text()
    section = body.split("# Neutral navigation\n", 1)[1].split("## Assignment", 1)[0]
    assert "# Neutral navigation\n" + section.rstrip("\n") + "\n" == t, p
print("navigation.md patched and identical to every prompt's navigation section")
