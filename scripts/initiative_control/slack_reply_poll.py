"""Owner-invoked, read-only Slack observations; never decision authority.

Inject a synchronous authenticated WebClient with retries disabled, timeout <=30s
and a disabled/private logger. No credentials, CLI, scheduling or Slack journals.
Every invocation scans from the start: cursors in receipts describe incomplete
coverage, not durable watermarks or inputs that could skip earlier edits.
"""

import re

from coordinator import digest, ident, require


MAX_TEXT_BYTES = 16000
API_ERRORS = frozenset({"ratelimited", "thread_not_found", "channel_not_found",
                        "not_in_channel", "missing_scope", "invalid_auth",
                        "token_revoked", "account_inactive", "invalid_cursor",
                        "access_denied", "request_timeout", "internal_error"})


class PollFailure(Exception):
    def __init__(self, code, retry_after=None):
        super().__init__(code)
        self.code, self.retry_after = code, retry_after


def _stamp(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9]{1,16}\.[0-9]{6}", value)


def _data(response):
    # SlackResponse is not a dict and must not be iterated (iteration fetches pages).
    return response if isinstance(response, dict) else getattr(response, "data", None)


def _api_failure(response):
    data = _data(response)
    error = data.get("error") if isinstance(data, dict) else None
    status = getattr(response, "status_code", None)
    headers = getattr(response, "headers", {}) or {}
    retry = headers.get("Retry-After", headers.get("retry-after"))
    if isinstance(retry, str) and re.fullmatch(r"[0-9]{1,8}", retry):
        retry = int(retry)
    else:
        retry = None
    code = error if isinstance(error, str) and error in API_ERRORS else "api_failure"
    if status == 429:
        code = "ratelimited"
    return PollFailure(code, retry if code == "ratelimited" else None)


def _call(method, **kwargs):
    try:
        response = method(**kwargs)
    except Exception as exc:
        raise _api_failure(getattr(exc, "response", None)) from None
    data = _data(response)
    if not isinstance(data, dict) or data.get("ok") is not True:
        raise _api_failure(response)
    status = getattr(response, "status_code", 200)
    if status != 200:
        raise _api_failure(response)
    return data


def _scope(message, pin):
    return (all(message.get(key, pin[key]) == pin[key] for key in ("team", "channel"))
            and message.get("team_id", pin["team"]) == pin["team"])


def _candidate(message, pin, thread, decision, bot_user):
    if not isinstance(message, dict) or message.get("type") != "message":
        return None, "malformed"
    if not _scope(message, pin) or message.get("thread_ts") != thread:
        return None, "wrong_scope"
    if message.get("subtype") in ("message_deleted", "tombstone"):
        return None, "deletion_ambiguous"
    ts, text = message.get("ts"), message.get("text")
    if not _stamp(ts) or tuple(map(int, ts.split("."))) <= tuple(map(int, thread.split("."))):
        return None, "malformed"
    if (message.get("bot_id") == pin["bot"] and message.get("user") == bot_user
            and message.get("subtype") in (None, "bot_message", "thread_broadcast")):
        return None, None  # Authenticated outbound echo, not missing human coverage.
    if (message.get("user") != pin["human"] or "bot_id" in message
            or "bot_profile" in message or "app_id" in message):
        return None, "denied_author"
    if message.get("subtype") not in (None, "thread_broadcast"):
        return None, "unsupported_subtype"
    if not isinstance(text, str):
        return None, "malformed"
    try:
        if len(text.encode("utf-8")) > MAX_TEXT_BYTES:
            return None, "text_exceeds_bound"
    except UnicodeError:
        return None, "malformed"
    edited = message.get("edited")
    if "edited" in message and (not isinstance(edited, dict) or not _stamp(edited.get("ts"))
                                or edited.get("user") != pin["human"]):
        return None, "invalid_edit"
    body = dict(source="slack_reply_poll_v1", kind="reply_observation", authority="evidence_only",
                **pin, thread_ts=thread, decision=decision, message_ts=ts,
                edited_ts=edited["ts"] if edited else None, text=text)
    # Exact original text and revision are immutable evidence, never parsed for routing.
    return {"id": "slack-poll:" + digest(body), **body}, None


def _page(data, page_size):
    messages = data.get("messages")
    metadata = data.get("response_metadata", {})
    if (not isinstance(messages, list) or len(messages) > page_size
            or not isinstance(metadata, dict) or type(data.get("has_more", False)) is not bool
            or ("has_more" not in data and "next_cursor" not in metadata)):
        raise PollFailure("malformed_page")
    cursor = metadata.get("next_cursor", "")
    if not isinstance(cursor, str) or len(cursor) > 1024 or any(ord(c) < 33 or ord(c) > 126 for c in cursor):
        raise PollFailure("malformed_cursor")
    if data.get("has_more") and not cursor:
        raise PollFailure("missing_cursor")
    return messages, cursor


def collect(web, coordinator, *, team, bot, human, channel, threads,
            max_pages=4, page_size=15):
    """Return redacted coverage only after each accepted observation commits.

    threads is an owner-supplied exact {parent_ts: decision_id} map (1..16).
    max_pages bounds all replies calls together (1..64); page_size is 1..100.
    Failure stops this invocation without retry/sleep. Unscanned/partial threads
    require another full scan with sufficient budget or a narrower owner map.
    No receipt, including scan_complete, proves absence of unseen deletions,
    intermediate edits, or arrivals during/after these non-atomic API reads.
    """
    pin = dict(team=team, bot=bot, human=human, channel=channel)
    for key, prefix in (("team", "T"), ("bot", "B"), ("human", "U"), ("channel", "CGD")):
        require(isinstance(pin[key], str) and re.fullmatch(f"[{prefix}][A-Z0-9]{{1,79}}", pin[key]),
                "invalid owner identity pin")
    require(isinstance(threads, dict) and 1 <= len(threads) <= 16, "invalid tracked thread count")
    routes = dict(threads)
    for thread, decision in routes.items():
        require(_stamp(thread), "invalid owner thread pin")
        ident(decision)
    require(type(max_pages) is int and 1 <= max_pages <= 64, "invalid page budget")
    require(type(page_size) is int and 1 <= page_size <= 100, "invalid page size")
    require(getattr(web, "retry_handlers", None) == [], "injected client must disable retries")
    require(type(getattr(web, "timeout", None)) in (int, float) and 0 < web.timeout <= 30,
            "injected client must bound request timeout")
    logger = getattr(web, "logger", None)
    require(logger is not None and logger.disabled, "injected client logger must be disabled")
    coverage = {ts: dict(decision=decision, pages=0, observed=0, added=0, duplicates=0, ignored=0,
                        issues={}, scanned_to_end=False, next_cursor=None, status="unscanned")
                for ts, decision in routes.items()}
    result = dict(status="partial", pages=0, scan_complete=False,
                  deletions="unknown", atomic_snapshot=False, threads=coverage, failure=None)
    phase, current = "auth", None
    try:
        auth = _call(web.auth_test)
        bot_user = auth.get("user_id")
        if (auth.get("team_id") != team or auth.get("bot_id") != bot or bot_user == human
                or not isinstance(bot_user, str) or not re.fullmatch(r"U[A-Z0-9]{1,79}", bot_user)):
            raise PollFailure("identity_mismatch")
        for thread, decision in routes.items():
            current = coverage[thread]
            cursor, seen = "", set()
            while result["pages"] < max_pages:
                phase = "replies"
                current["status"] = "partial"
                data = _call(web.conversations_replies, channel=channel, ts=thread,
                             limit=page_size, cursor=cursor)
                result["pages"] += 1
                current["pages"] += 1
                messages, next_cursor = _page(data, page_size)
                if not _scope(data, pin):
                    raise PollFailure("identity_mismatch")
                if not cursor:
                    parent = messages[0] if messages else None
                    if (not isinstance(parent, dict) or parent.get("ts") != thread
                            or parent.get("thread_ts", thread) != thread
                            or parent.get("type") != "message" or not _scope(parent, pin)
                            or parent.get("bot_id") != bot
                            or parent.get("subtype") not in (None, "bot_message")):
                        raise PollFailure("parent_unverified")
                    messages = messages[1:]
                for message in messages:
                    event, issue = _candidate(message, pin, thread, decision, bot_user)
                    if issue:
                        current["issues"][issue] = current["issues"].get(issue, 0) + 1
                        continue
                    if event is None:
                        current["ignored"] += 1
                        continue
                    phase = "persist"
                    try:
                        added = coordinator.event(event)
                    except Exception:
                        # A commit followed by an exception remains uncertain; replay deduplicates.
                        raise PollFailure("persistence_uncertain") from None
                    current["observed"] += 1
                    current["added" if added else "duplicates"] += 1
                phase = "replies"
                current["next_cursor"] = next_cursor or None
                if not next_cursor:
                    current["scanned_to_end"] = True
                    current["status"] = "partial" if current["issues"] else "scan_complete"
                    break
                if next_cursor in seen:
                    raise PollFailure("cursor_cycle")
                seen.add(next_cursor)
                cursor = next_cursor
            if not current["scanned_to_end"]:
                break
        result["scan_complete"] = all(c["status"] == "scan_complete" for c in coverage.values())
        result["status"] = "scan_complete" if result["scan_complete"] else "partial"
    except PollFailure as exc:
        if current is not None:
            current["status"] = "failed"
        result["status"] = "failed"
        result["failure"] = dict(code=exc.code, phase=phase, retry_after=exc.retry_after)
    return result
