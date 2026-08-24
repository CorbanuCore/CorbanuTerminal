You are implementing the LogTriage benchmark task.

Work in this repository only. Do not remove tests. Do not bypass the verifier.

Goal:
Implement the `logtriage` package so it can parse mixed-format log lines,
deduplicate entries, summarize severity/service/message rollups, and query time
windows.

Core API:

```python
from logtriage import parse_logs, summarize_logs, query_window

parsed = parse_logs(lines)
summary = summarize_logs(lines, start=None, end=None)
entries = query_window(lines, start, end, severity=None, service=None)
```

`lines` is an iterable of strings. Blank lines are ignored.

Supported input formats:

1. JSON objects:
   `{"ts":"2026-07-02T10:00:00Z","level":"info","service":"api","id":"a1","message":"started"}`
2. Key/value lines:
   `ts=2026-07-02T10:00:00Z level=warn service=api id=a2 msg="cache warm"`
3. Pipe lines:
   `2026-07-02T10:00:00Z | ERROR | worker | id=w1 | job failed`

Output from `parse_logs`:

```python
{
  "entries": [
    {
      "ts": "2026-07-02T10:00:00Z",
      "severity": "ERROR",
      "service": "worker",
      "id": "w1",
      "message": "job failed"
    }
  ],
  "duplicate_count": 0,
  "diagnostics": []
}
```

Rules:
- Sort parsed entries by timestamp, then by input order.
- Normalize severities to `DEBUG`, `INFO`, `WARNING`, `ERROR`, or `CRITICAL`.
- Aliases: `warn`/`warning` -> `WARNING`; `err`/`error` -> `ERROR`;
  `fatal`/`crit`/`critical` -> `CRITICAL`.
- Required fields after parsing: timestamp, severity, service, and message.
- `id` is optional. If present, dedupe by id. If absent, dedupe by
  `(ts, severity, service, message)`.
- Keep the first occurrence of a duplicate and increment `duplicate_count`.
- Invalid JSON, malformed key/value, malformed pipe lines, unknown severity, or
  missing required fields must add a diagnostic with a stable `code` and never
  crash parsing.
- `query_window` returns entries in `[start, end)` and applies optional severity
  and service filters after normalization.
- `summarize_logs` summarizes the same window behavior as `query_window` and
  returns:

```python
{
  "total": 3,
  "by_severity": {"ERROR": 2},
  "by_service": {"api": 2},
  "top_messages": [{"message": "failed", "count": 2}],
  "first_ts": "2026-07-02T10:00:00Z",
  "last_ts": "2026-07-02T10:02:00Z",
  "duplicate_count": 1,
  "diagnostics": []
}
```

`top_messages` is sorted by descending count, then message text ascending.

Definition of done:
- `python3 -m unittest discover -s tests` passes.
- The benchmark harness's external verifier passes.
- Keep code readable and scoped. Do not add network dependencies.
