You are implementing the RateGate benchmark task.

Work in this repository only. Do not remove tests. Do not bypass the verifier.

Goal:
Implement the `rategate` package so it can evaluate per-key token-bucket and
sliding-window rate limits over request events.

Core API:

```python
from rategate import token_bucket, sliding_window

bucket = token_bucket(requests, rate=1.5, capacity=5)
window = sliding_window(requests, limit=10, window_seconds=60)
```

`requests` is an iterable of dicts. Each request has:

```python
{"id": "r1", "key": "user-1", "ts": 0, "cost": 1}
```

Rules:
- Process requests sorted by timestamp, then original input order.
- `key` is required and buckets/windows are independent per key.
- `id` is optional but should be echoed when present.
- `ts` may be numeric seconds or an ISO UTC timestamp ending in `Z`.
- Token bucket starts full at `capacity` for each new key.
- Refill amount is elapsed_seconds * `rate`, capped at `capacity`.
- `cost` defaults to 1. A request is allowed only when current tokens >= cost.
- Rejected token-bucket decisions do not consume tokens.
- Sliding window allows at most `limit` accepted requests per key in the half-open
  interval `(ts - window_seconds, ts]`; rejected requests are not counted.
- Invalid requests must add diagnostics with stable `code` values and produce a
  rejected decision when possible; they must never crash evaluation.
- Monetary precision is irrelevant; use normal numeric arithmetic and round
  `tokens_remaining` to six decimal places in output.

Output shape:

```python
{
  "decisions": [
    {
      "id": "r1",
      "key": "user-1",
      "ts": 0.0,
      "allowed": True,
      "reason": "allowed",
      "tokens_remaining": 4.0
    }
  ],
  "diagnostics": []
}
```

For sliding windows, decisions include `window_count` instead of
`tokens_remaining`.

Definition of done:
- `python3 -m unittest discover -s tests` passes.
- The benchmark harness's external verifier passes.
- Keep code readable and scoped. Do not add network dependencies.
