#!/usr/bin/env python3
"""Independent recomputation of GLM 5.2 (Z.AI) spend from provider-reported usage.

Sources of provider-reported usage (raw SSE chunks as received from api.z.ai):
  * `--err FILE`   : stderr of `corbanu exec` run with RUST_LOG=codex_api=trace
  * `--logdb FILE` : the TUI's log database (same trace line, feedback_log_body)
  * `--proxy FILE` : JSONL written by usage_proxy.py (wire capture outside the product)
Each usage chunk is keyed by the provider's response id, so duplicates across
sources are counted once.

Published Z.AI pay-as-you-go prices (docs.z.ai/guides/overview/pricing, checked
2026-10-08), USD per 1M tokens: input 1.40, cached input 0.26, output 4.40.
Billable: (prompt - cached) * input + cached * cached_input + completion * output.
Reasoning tokens are part of completion_tokens.
"""
import argparse, collections, datetime, json, re, sqlite3
from decimal import Decimal

P_IN, P_CACHED, P_OUT = Decimal("1.40"), Decimal("0.26"), Decimal("4.40")
M = Decimal(1_000_000)
LINE = re.compile(r'^(\S+Z)\s.*Chat completions SSE event: (\{.*\})\s*$')


def cost(u):
    p, c, o = u["prompt_tokens"], (u.get("prompt_tokens_details") or {}).get("cached_tokens", 0) or 0, u["completion_tokens"]
    return (Decimal(p - c) * P_IN + Decimal(c) * P_CACHED + Decimal(o) * P_OUT) / M


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--err", nargs="*", default=[])
    ap.add_argument("--logdb", nargs="*", default=[])
    ap.add_argument("--proxy", nargs="*", default=[])
    ap.add_argument("--group", nargs="*", default=[], help="label=src1,src2 groups for subtotals")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    rows = {}

    def add(src, ts, obj):
        u = obj.get("usage")
        if not u:
            return
        rid = obj.get("id") or f"{src}:{ts}"
        rows.setdefault(rid, {"id": rid, "src": src, "ts": ts, "model": obj.get("model"), "usage": u})

    for f in a.err:
        for line in open(f, errors="replace"):
            m = LINE.match(line.strip())
            if m:
                add(f.split("/")[-1].rsplit(".", 1)[0], m.group(1), json.loads(m.group(2)))
    for f in a.logdb:
        db = sqlite3.connect(f"file:{f}?mode=ro", uri=True)
        for ts, body in db.execute("select ts, feedback_log_body from logs where feedback_log_body like '%Chat completions SSE event%usage%'"):
            j = body.split("Chat completions SSE event: ", 1)[1]
            add("tui:" + f.split("/")[-3], datetime.datetime.fromtimestamp(ts, datetime.timezone.utc).isoformat(), json.loads(j))
    for f in a.proxy:
        for line in open(f):
            r = json.loads(line)
            for u in r["usage"]:
                add("proxy:" + f.split("/")[-1], datetime.datetime.fromtimestamp(r["t_end"], datetime.timezone.utc).isoformat(), {"id": (r["ids"] or [None])[0], "model": r["model"], "usage": u})

    by = collections.defaultdict(lambda: [0, 0, 0, 0, 0, Decimal(0)])
    for r in sorted(rows.values(), key=lambda r: r["ts"]):
        u = r["usage"]
        c = cost(u)
        r["usd"] = str(c)
        b = by[r["src"]]
        b[0] += 1; b[1] += u["prompt_tokens"]; b[2] += (u.get("prompt_tokens_details") or {}).get("cached_tokens", 0) or 0
        b[3] += u["completion_tokens"]; b[4] += u.get("total_tokens", 0); b[5] += c
        if not a.json:
            print(f'{r["ts"][:19]} {r["src"]:<22} {r["id"]:<36} p={u["prompt_tokens"]:>7} c={b and (u.get("prompt_tokens_details") or {}).get("cached_tokens",0):>7} o={u["completion_tokens"]:>5} tot={u.get("total_tokens"):>7} usd={c:.6f}')
    tot = [0, 0, 0, 0, 0, Decimal(0)]
    print("\nper source: requests prompt cached completion total usd")
    for s, b in sorted(by.items()):
        print(f"  {s:<24} {b[0]:>3} {b[1]:>8} {b[2]:>8} {b[3]:>6} {b[4]:>8} {b[5]:.6f}")
        tot = [x + y for x, y in zip(tot, b)]
    for g in a.group:
        name, srcs = g.split("=")
        gs = [by[s] for s in srcs.split(",")]
        print(f"  GROUP {name:<18} {sum(b[0] for b in gs):>3} requests, {sum(b[4] for b in gs)} tokens, usd={sum((b[5] for b in gs), Decimal(0)):.6f}")
    print(f"  ALL {tot[0]} requests, {tot[4]} tokens, usd={tot[5]:.6f}")


if __name__ == "__main__":
    main()
