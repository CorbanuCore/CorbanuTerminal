#!/usr/bin/env python3
"""reconcile.py <state.sqlite> [proxy.jsonl]

Per thread and per (provider, account): ledger attempts and usage, against the
provider-reported usage the proxy saw (when a proxy log is given). The ledger's
account is printed as recorded; the proxy side groups by the request's
`thread-id` header and the sha256 prefix of the bearer (no key values).
"""
import collections, json, sqlite3, sys

db = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
fmt = []
for t in ("draft_accounting_ledger_format", "draft_accounting_ledger_format_3", "draft_accounting_ledger_format_4"):
    try:
        fmt += [r[0] for r in db.execute(f"select version from {t}")]
    except sqlite3.OperationalError:
        pass
print("ledger format rows:", sorted(fmt) or "none")
latest = {}
for aid, rev, payload in db.execute("select attempt_id, revision, payload from draft_accounting_observations order by revision"):
    latest[aid] = json.loads(payload)["patch"]
rows, prewarm = [], []
for (payload,) in db.execute("select payload from draft_accounting_attempts"):
    a = json.loads(payload)
    acct = a.get("account")
    u = latest.get(a["attempt_id"])
    if u is not None and a.get("dialect") == "NativeAnthropic":
        # Anthropic reports input excluding cache reads/writes; normalise to inclusive input.
        u = dict(u, input=(u.get("input") or 0) + (u.get("read") or 0) + (u.get("write") or 0))
        u["total"] = u["input"] + (u.get("output") or 0)
    if str(a.get("turn", "")).startswith("prewarm:"):
        prewarm.append((a["thread_id"], u))
        if "--exclude-prewarm" in sys.argv:
            continue
    rows.append((a["thread_id"], a["provider"], a["model"], acct["name"] if acct else "(none)",
                 acct["fingerprint"] if acct else "", u))
by_thread = collections.defaultdict(lambda: [0, 0, 0, 0, 0, set()])
by_acct = collections.defaultdict(lambda: [0, 0, 0, 0, 0, 0])
for th, prov, model, name, fp, u in rows:
    k = (prov, name)
    by_acct[k][0] += 1
    t = by_thread[th]; t[0] += 1; t[5].add((prov, name, fp))
    if u is None:
        by_acct[k][5] += 1
        continue
    for i, f in enumerate(("input", "read", "output", "total"), 1):
        by_acct[k][i] += u.get(f) or 0
        t[i] += u.get(f) or 0
if prewarm:
    print(f"prewarm attempts (websocket warm-up, not in rollout turns): {[(t[-6:], (u or {}).get('total')) for t, u in prewarm]}"
          + (" -- EXCLUDED below" if "--exclude-prewarm" in sys.argv else ""))
print("\nLEDGER per thread: attempts input cached output total | (provider, account, fingerprint)")
for th, t in sorted(by_thread.items()):
    print(f"  {th} {t[0]:>3} {t[1]:>7} {t[2]:>7} {t[3]:>6} {t[4]:>7} | {sorted(t[5])}")
print("\nLEDGER per (provider, account): attempts input cached output total no-usage-attempts")
for k, v in sorted(by_acct.items()):
    print(f"  {k[0]:<10} {k[1]:<8} {v[0]:>3} {v[1]:>7} {v[2]:>7} {v[3]:>6} {v[4]:>7} {v[5]}")
print("  (/cost 'tokens' = input + output, input including cached)")
for k, v in sorted(by_acct.items()):
    print(f"  /cost-equivalent {k[0]} {k[1]}: {v[1] + v[3]:,} tokens")

if len(sys.argv) > 2 and sys.argv[2] != "-":
    print("\nPROXY (provider-reported) per thread: requests status-counts prompt cached completion total | bearer-hash")
    pt = collections.defaultdict(lambda: [0, collections.Counter(), 0, 0, 0, 0, set()])
    for line in open(sys.argv[2]):
        r = json.loads(line)
        if not r.get("path", "").endswith("/chat/completions"):
            continue
        th = r["headers"].get("thread-id") or r["headers"].get("Thread-Id") or "?"
        p = pt[th]; p[0] += 1; p[1][str(r["status"]) + ("/client-aborted" if r.get("client_aborted") else "")] += 1; p[6].add(r["auth_h"])
        for u in r.get("usage") or []:
            p[2] += u.get("prompt_tokens", 0); p[3] += (u.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
            p[4] += u.get("completion_tokens", 0); p[5] += u.get("total_tokens", 0)
    ok = True
    for th, p in sorted(pt.items()):
        l = by_thread.get(th)
        match = l is not None and (l[0], l[1], l[2], l[3], l[4]) == (p[0], p[2], p[3], p[4], p[5])
        tok = l is not None and (l[1], l[2], l[3], l[4]) == (p[2], p[3], p[4], p[5])
        ok &= match
        print(f"  {th} {p[0]:>3} {dict(p[1])} {p[2]:>7} {p[3]:>7} {p[4]:>6} {p[5]:>7} | {sorted(p[6])} | ledger "
              f"{'MATCH' if match else ('TOKENS MATCH, attempts ' + str(l[0]) if tok else 'DIFF ' + str(l[:5] if l else None))}")
    missing = set(by_thread) - set(pt)
    print("  ledger threads with no proxied request:", sorted(missing) or "none")
    print("PROXY vs LEDGER exact per thread:", "ALL MATCH" if ok else "MISMATCH")

if len(sys.argv) > 3 and not sys.argv[3].startswith("--"):
    import glob, os
    print("\nROLLOUT (provider-reported usage as recorded per thread: last cumulative token_count) vs LEDGER")
    ok = True
    for f in sorted(glob.glob(os.path.join(sys.argv[3], "**", "rollout-*.jsonl"), recursive=True)):
        th = f[-42:-6]
        last = None
        for line in open(f):
            d = json.loads(line)
            p = d.get("payload") or {}
            if d.get("type") == "event_msg" and p.get("type") == "token_count" and p.get("info"):
                last = p["info"]["total_token_usage"]
        l = by_thread.get(th)
        r = (last["input_tokens"], last["cached_input_tokens"], last["output_tokens"], last["total_tokens"]) if last else (0, 0, 0, 0)
        match = l is not None and (l[1], l[2], l[3], l[4]) == r
        ok &= match
        print(f"  {th} rollout in={r[0]} cached={r[1]} out={r[2]} total={r[3]} | ledger attempts={l[0] if l else '-'} "
              f"{sorted(l[5]) if l else ''} {'MATCH' if match else 'DIFF ' + str(l[:5] if l else None)}")
    print("ROLLOUT vs LEDGER exact per thread:", "ALL MATCH" if ok else "MISMATCH")
