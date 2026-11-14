"""S01 synthetic contract oracle. Not a native adapter or product persistence layer."""

from copy import deepcopy
from datetime import datetime
from decimal import Decimal, InvalidOperation
from fractions import Fraction


FIELDS = ("input", "read", "write", "output", "reasoning", "total")


def exact(value):
    """Require decimal text; arithmetic uses fractions without rounding."""
    if not isinstance(value, str):
        raise ValueError("money must be decimal text")
    try:
        number = Decimal(value)
    except InvalidOperation as error:
        raise ValueError("invalid decimal money") from error
    if not number.is_finite() or number < 0:
        raise ValueError("invalid nonnegative money")
    return Fraction(number)


def instant(value):
    stamp = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if stamp.tzinfo is None:
        raise ValueError("timestamp must have timezone")
    return stamp


def count(value):
    if value is not None and (type(value) is not int or value < 0):
        raise ValueError("invalid token count")
    return value


def merge_patch(previous, patch):
    """A missing/null field adds no measurement; supplied counters replace."""
    result = deepcopy(previous)
    for key, value in patch.items():
        if isinstance(value, dict):
            result[key] = merge_patch(result.get(key, {}), value)
        elif value is not None:
            result[key] = count(value)
    return result


def normalize(wire, usage):
    if wire == "responses":
        inputs = usage.get("input_tokens_details", {})
        outputs = usage.get("output_tokens_details", {})
        values = [usage.get("input_tokens"), inputs.get("cached_tokens"),
                  inputs.get("cache_write_tokens"), usage.get("output_tokens"),
                  outputs.get("reasoning_tokens"), usage.get("total_tokens")]
    elif wire == "chat":
        inputs = usage.get("prompt_tokens_details", {})
        outputs = usage.get("completion_tokens_details", {})
        # Native ChatUsage conversion assigns cache-write zero without a wire
        # field. Preserve unknown here; this is not provider-reported zero.
        values = [usage.get("prompt_tokens"), inputs.get("cached_tokens"), None,
                  usage.get("completion_tokens"), outputs.get("reasoning_tokens"),
                  usage.get("total_tokens")]
    elif wire == "anthropic":
        # Explicitly native Anthropic noncached-input dialect, not its compatible
        # providers' cumulative-input heuristic. Unknown dialects are rejected.
        parts = [count(usage.get(name)) for name in
                 ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens")]
        input_count = sum(parts) if all(v is not None for v in parts) else None
        output = usage.get("output_tokens")
        total = input_count + output if input_count is not None and output is not None else None
        values = [input_count, parts[1], parts[2], output, None, total]
    else:
        raise ValueError("unsupported wire dialect")
    result = dict(zip(FIELDS, (count(value) for value in values)))
    if wire == "anthropic":
        # Internal pricing measurement: independent of an unknown cache split.
        # This is not inclusive input and must not enter rows or token totals.
        result["_noncached_input"] = parts[0]
    if result["input"] is not None:
        known_cache = sum(result[key] for key in ("read", "write") if result[key] is not None)
        if known_cache > result["input"]:
            raise ValueError("cache exceeds inclusive input")
    if result["reasoning"] is not None and result["output"] is not None:
        if result["reasoning"] > result["output"]:
            raise ValueError("reasoning exceeds inclusive output")
    if all(result[key] is not None for key in ("input", "output", "total")):
        if result["input"] + result["output"] != result["total"]:
            raise ValueError("provider total conflicts with supported dialect")
    return result


def select_price(attempt, prices):
    at = instant(attempt["dispatched_at"]) if attempt["dispatched_at"] else None
    matches = []
    identifiers = set()
    for price in prices:
        if price["price_id"] in identifiers:
            raise ValueError("duplicate price identity")
        identifiers.add(price["price_id"])
        if not price["provenance"] or price["unit"] != "per_million_tokens":
            raise ValueError("missing provenance or unsupported price unit")
        start = instant(price["effective_from"])
        end = instant(price["effective_to"]) if price["effective_to"] else None
        instant(price["captured_at"])
        if end is not None and end <= start:
            raise ValueError("invalid effective interval")
        for rate in price["rates"].values():
            if rate is not None:
                exact(rate)
        if all(price[key] == attempt[key] for key in ("provider", "account_scope", "model")):
            if at is not None and start <= at and (end is None or at < end):
                matches.append(price)
    if len(matches) > 1:
        raise ValueError("overlapping prices")
    price = matches[0] if matches else None
    if price is not None and price["currency"] != "USD":
        raise ValueError("unsupported currency")
    return price


def estimate(usage, price, status):
    buckets = {key: usage[key] for key in ("read", "write", "output")}
    if "_noncached_input" in usage:
        buckets["input"] = usage["_noncached_input"]
    else:
        parts = [usage[key] for key in ("input", "read", "write")]
        buckets["input"] = parts[0] - parts[1] - parts[2] if all(v is not None for v in parts) else None
    rates = price["rates"] if price else {}
    known = Fraction(0)
    unknown = []
    for name, tokens in buckets.items():
        if tokens is None:
            unknown.append(name + ":usage_unknown")
        elif tokens == 0:
            continue
        elif rates.get(name) is None:
            unknown.append(name + ":price_missing")
        else:
            known += tokens * exact(rates[name]) / 1_000_000
    if status != "completed":
        unknown.append("attempt_not_complete")
    return {"known_cost": known, "cost": None if unknown else known,
            "unknown_reasons": unknown, "price_id": price["price_id"] if price else None}


def validate_threads(threads):
    for thread in threads:
        seen = set()
        while thread is not None:
            if thread in seen:
                raise ValueError("lineage cycle")
            if thread not in threads:
                raise ValueError("unknown parent")
            seen.add(thread)
            thread = threads[thread]


class Replay:
    """Stores synthetic observations for deterministic replay and JSON roundtrips."""

    def __init__(self, attempts, threads):
        validate_threads(threads)
        self.threads = deepcopy(threads)
        self.attempts = {}
        for attempt in attempts:
            key = attempt["attempt_id"]
            if key in self.attempts:
                raise ValueError("duplicate attempt identity")
            if attempt["thread_id"] not in threads:
                raise ValueError("unattributed attempt")
            self.attempts[key] = deepcopy(attempt)
        owners = {}
        for attempt in attempts:
            owner = (attempt["thread_id"], attempt["turn_id"])
            if owners.setdefault(attempt["request_id"], owner) != owner:
                raise ValueError("logical request has conflicting owner")
            seen = {attempt["attempt_id"]}
            previous = attempt["retry_of"]
            while previous is not None:
                if previous in seen or previous not in self.attempts:
                    raise ValueError("invalid retry chain")
                seen.add(previous)
                parent = self.attempts[previous]
                if any(parent[key] != attempt[key] for key in ("request_id", "thread_id", "turn_id")):
                    raise ValueError("retry changes logical request")
                previous = parent["retry_of"]
        self.events = {}

    def ingest(self, events):
        # Validate shape/counts/duplicate keys before committing the batch.
        # Cross-field semantics are checked while reconstructing rows.
        pending = deepcopy(self.events)
        for event in events:
            if event["attempt_id"] not in self.attempts:
                raise ValueError("unknown attempt")
            if type(event["revision"]) is not int or event["revision"] < 1:
                raise ValueError("invalid revision")
            if event["status"] not in {"streaming", "completed", "interrupted", "failed", "cancelled"}:
                raise ValueError("invalid status")
            if not event["provenance"]:
                raise ValueError("missing observation provenance")
            if set(event) != {"attempt_id", "revision", "status", "provenance", "usage"}:
                raise ValueError("immutable identity cannot be patched")
            merge_patch({}, event["usage"])
            key = (event["attempt_id"], event["revision"])
            if key in pending and pending[key] != event:
                raise ValueError("conflicting replay identity")
            pending[key] = deepcopy(event)
        self.events = pending

    def snapshot(self):
        return [deepcopy(self.events[key]) for key in sorted(self.events)]

    def rows(self, prices):
        rows = {}
        for key, attempt in self.attempts.items():
            raw, status = {}, "interrupted"
            for event in self.snapshot():
                if event["attempt_id"] == key:
                    raw = merge_patch(raw, event["usage"])
                    status = event["status"]
                    normalize(attempt["wire"], raw)
            usage = normalize(attempt["wire"], raw)
            billed = attempt["billed"]
            if billed is not None:
                if billed["currency"] != "USD" or not billed["line_id"] or not billed["provenance"]:
                    raise ValueError("invalid billed evidence")
            rows[key] = {**{field: usage[field] for field in FIELDS},
                         **estimate(usage, select_price(attempt, prices), status),
                         "billed": exact(billed["amount"]) if billed is not None else None}
        return rows

    def aggregate(self, root, prices):
        if root not in self.threads:
            raise ValueError("unknown root")
        descendants = {root}
        while True:
            more = {child for child, parent in self.threads.items() if parent in descendants}
            if more <= descendants:
                break
            descendants |= more
        rows = self.rows(prices)
        keys = [key for key, attempt in self.attempts.items() if attempt["thread_id"] in descendants]
        result = {"attempts": len(keys), "requests": len({self.attempts[key]["request_id"] for key in keys})}
        for field in FIELDS:
            known = sum(rows[key][field] for key in keys if rows[key][field] is not None)
            unknown = sum(rows[key][field] is None for key in keys)
            result[field] = {"known": known, "unknown": unknown, "total": None if unknown else known}
        lines = [self.attempts[key]["billed"]["line_id"] for key in keys if self.attempts[key]["billed"]]
        if len(lines) != len(set(lines)):
            raise ValueError("invoice line allocated twice")
        for field, subtotal in (("cost", "known_cost"), ("billed", "billed")):
            known = sum((rows[key][subtotal] for key in keys if rows[key][subtotal] is not None), Fraction(0))
            unknown = sum(rows[key][field] is None for key in keys)
            result["known_" + field] = known
            result["unknown_" + field + "_attempts"] = unknown
            result[field] = None if unknown else known
        return result
