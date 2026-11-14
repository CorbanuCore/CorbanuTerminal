#!/usr/bin/env python3
"""Leak scan for the S06 acceptance. Real keys are read from REAL_<n> env vars set by the caller from the
vault helper (never printed); synthetic canaries are listed below. Scans every file under the given roots
and prints, per needle class, the files that contain it (paths relative to $S or the evidence dir).

usage: REAL_1=... REAL_2=... leakscan.py <root> [<root> ...]
"""
import os, sys

S = os.environ["S"]
EV = os.environ.get("EV", "")
real = {k: v for k, v in os.environ.items() if k.startswith("REAL_") and v}
synthetic = {
    "aws secret keys (fakesecret-*-S06CANARY)": b"fakesecret-",
    "credential_process helper stderr (PROCSTDERR-S06CANARY)": b"PROCSTDERR-S06CANARY",
    "cached SSO access token (fake-sso-access-S06CANARY)": b"fake-sso-access-S06CANARY",
    "Bedrock bearer env canary (fakebearer-env-S06CANARY)": b"fakebearer-env-S06CANARY",
    "named OpenAI API-key canaries (sk-canary-*/sk-fake-*/sk-tuiadd-*)": (b"sk-canary-", b"sk-fake-", b"sk-tuiadd-"),
    "ChatGPT refresh tokens from the mock (rt.<user>.<gen>)": (b"rt.work.", b"rt.play.", b"rt.tuiwork.", b"rt.devcode.", b"rt.r1", b"rt.r2", b"rt.bad.", b"rt.legacy.", b"rt.named."),
    "mock JWT header (eyJhbGciOiAibm9uZSIs)": b"eyJhbGciOiAibm9uZSIs",
    "env decoys (sk-envdecoy-, sk-codexenv-, fake-env-session, fake-webid-token, CODEX_ACCESS_TOKEN JWT eyJhbGciOiAibm9uZSJ9)":
        (b"sk-envdecoy-", b"sk-codexenv-", b"fake-env-session", b"fake-webid-token", b"eyJhbGciOiAibm9uZSJ9"),
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
