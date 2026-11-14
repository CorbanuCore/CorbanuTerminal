#!/usr/bin/env python3
"""Leak scan for the S06 slice-B re-acceptance (copied from the 2026-10-11 acceptance, needles updated). Real keys are read from REAL_<n> env vars set by the caller from the
vault helper (never printed); synthetic canaries are listed below. Scans every file under the given roots
and prints, per needle class, the files that contain it (paths relative to $S or the evidence dir).

usage: REAL_1=... REAL_2=... leakscan.py <root> [<root> ...]
"""
import os, sys

S = os.environ["S"]
EV = os.environ.get("EV", "")
real = {k: v for k, v in os.environ.items() if k.startswith("REAL_") and v}
for k, v in list(real.items()):
    for name, off in (("frag@8", 8), ("frag@mid", len(v) // 2 - 6), ("frag@end", len(v) - 12)):
        real[f"{k} {name} (12 chars)"] = v[off:off + 12]
synthetic = {
    "named/default OpenAI API-key canaries (sk-canary-*, sk-fake-*, sk-tuiadd-*)": (b"sk-canary-", b"sk-fake-", b"sk-tuiadd-"),
    "env decoys (sk-envdecoy-, sk-codexenv-, CODEX_ACCESS_TOKEN JWT header eyJhbGciOiAibm9uZSJ9)": (b"sk-envdecoy-", b"sk-codexenv-", b"eyJhbGciOiAibm9uZSJ9"),
    "ChatGPT refresh tokens of NAMED accounts (rt.<user>.<gen>)": tuple(b"rt.%s." % u for u in (
        b"work", b"play", b"bad", b"named", b"namedo@ws=other", b"tuiwork", b"tuiwork2", b"devz", b"fpx", b"fpy", b"play2",
        b"work~2", b"deflt~2", b"play2~2", b"tuiwork~2", b"dmnamed", b"r1", b"r2")),
    "ChatGPT refresh tokens of DEFAULT logins (rt.deflt./rt.legacy./rt.tuidefault.)": (b"rt.deflt.", b"rt.legacy.", b"rt.tuidefault."),
    "mock JWT header (eyJhbGciOiAibm9uZSIs), in any token": b"eyJhbGciOiAibm9uZSIs",
    "fake AWS secrets / bearer env canary (fakesecret-, fakebearer-env-)": (b"fakesecret-", b"fakebearer-env-"),
}


def rel(p):
    return p.replace(S, "$S").replace(EV, "$EV") if EV else p.replace(S, "$S")


def main():
    hits = {}
    files = 0
    for root in sys.argv[1:]:
        for d, _, names in os.walk(root):
            for n in names:
                p = os.path.join(d, n)
                try:
                    data = open(p, "rb").read()
                except OSError:
                    continue
                files += 1
                for label, v in real.items():
                    if v.encode() in data:
                        hits.setdefault("REAL key " + label, []).append(rel(p))
                for label, needles in synthetic.items():
                    needles = needles if isinstance(needles, tuple) else (needles,)
                    if any(x in data for x in needles):
                        hits.setdefault(label, []).append(rel(p))
    print(f"scanned {files} files under {', '.join(rel(r) for r in sys.argv[1:])}")
    for k in sorted(real):
        print(f"\n[REAL key {k}] {len(hits.get('REAL key ' + k, []))} files")
        for p in sorted(hits.get("REAL key " + k, [])):
            print("   ", p)
    for label in synthetic:
        print(f"\n[{label}] {len(hits.get(label, []))} files")
        for p in sorted(hits.get(label, [])):
            print("   ", p)


main()
