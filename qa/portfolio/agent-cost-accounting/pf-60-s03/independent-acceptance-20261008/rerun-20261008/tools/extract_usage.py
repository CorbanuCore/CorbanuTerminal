#!/usr/bin/env python3
"""Extract provider-reported usage chunks from `corbanu exec` trace stderr (RUST_LOG=codex_api=trace)
or a TUI log database into redaction-safe JSONL: one line per provider response id with the raw
`usage` object exactly as Z.AI sent it, first/usage chunk client timestamps and time to first byte.
usage: extract_usage.py OUT.jsonl (FILE.err | FILE.sqlite) ...
"""
import json, re, sqlite3, sys, datetime

LINE = re.compile(r'^(\S+Z)\s.*Chat completions SSE event: (\{.*\})\s*$')
out = open(sys.argv[1], "w")
for f in sys.argv[2:]:
    src = f.split("/")[-1]
    first, usage, uts, lat = {}, {}, {}, {}

    def see(t, o):
        i = o.get("id")
        first.setdefault(i, t)
        if o.get("usage"):
            usage[i] = o["usage"]; uts[i] = t

    if f.endswith(".sqlite"):
        db = sqlite3.connect(f"file:{f}?mode=ro", uri=True)
        for ts, ns, body in db.execute("select ts, ts_nanos, feedback_log_body from logs where feedback_log_body like 'Chat completions SSE event:%' order by ts, ts_nanos, id"):
            t = datetime.datetime.fromtimestamp(ts, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S") + f".{ns:09d}Z"
            try:
                see(t, json.loads(body.split("SSE event: ", 1)[1]))
            except json.JSONDecodeError:
                pass  # non-JSON frames such as [DONE]
        src = "tui-log-db"
    else:
        for line in open(f, errors="replace"):
            s = line.strip()
            m = LINE.match(s)
            if m:
                see(m.group(1), json.loads(m.group(2)))
            elif '"corbanu_call_metrics"' in s:
                o = json.loads(s[s.index("{"):])
                lat[o.get("generation_id")] = o.get("ms_to_first_sse_byte")
    for i, u in usage.items():
        out.write(json.dumps({"source": src, "provider_response_id": i, "first_chunk_utc": first[i],
                              "usage_chunk_utc": uts[i], "ms_to_first_sse_byte": lat.get(i), "usage": u}) + "\n")
