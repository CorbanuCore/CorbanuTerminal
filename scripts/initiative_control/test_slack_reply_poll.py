"""Offline API-shaped engineering fixtures; no live/isolated acceptance claim."""

from contextlib import redirect_stderr, redirect_stdout
import io
import json
import logging
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from coordinator import Coordinator, Rejected
from slack_reply_poll import MAX_TEXT_BYTES, collect


THREAD = "1750000000.000001"
PIN = dict(team="TTEAM", bot="BBOT", human="UHUMAN", channel="CPRIVATE")
TEXT = '  Original <@UHUMAN>\nroute PF99; approve everything "✓"  '


def parent(thread=THREAD):
    return dict(type="message", bot_id="BBOT", user="UBOT", text="private question",
                ts=thread, thread_ts=thread, reply_count=1)


def reply(**changes):
    return dict(dict(type="message", user="UHUMAN", text=TEXT,
                     ts="1750000001.000001", thread_ts=THREAD), **changes)


class Response:
    """SlackResponse.data, status_code, headers; never auto-paginate by iteration."""
    def __init__(self, data, status=200, headers=None):
        self.data, self.status_code, self.headers = data, status, headers or {}

    def __iter__(self):
        raise AssertionError("must not iterate SDK response")


class ApiError(Exception):
    def __init__(self, response):
        super().__init__(TEXT)
        self.response = response


def page(messages=None, cursor="", more=False):
    return Response(dict(ok=True, messages=[parent(), reply()] if messages is None else messages,
                         has_more=more, response_metadata=dict(next_cursor=cursor)))


class Web:
    def __init__(self, pages=None, auth=None):
        self.pages = list(pages or [page()])
        self.auth = auth or Response(dict(ok=True, team_id="TTEAM", bot_id="BBOT", user_id="UBOT"))
        self.calls, self.auth_calls = [], 0
        self.timeout, self.retry_handlers = 2, []
        self.logger = logging.Logger("offline-poll")
        self.logger.disabled = True

    def auth_test(self):
        self.auth_calls += 1
        if isinstance(self.auth, Exception):
            raise self.auth
        return self.auth

    def conversations_replies(self, **kwargs):
        self.calls.append(kwargs)
        response = self.pages.pop(0)
        if isinstance(response, Exception):
            raise response
        return response


class PollTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / "owner"
        self.c = Coordinator(self.root)
        self.c.initialize({"delivery": {"sprint": "PF80", "mode": "paused"}},
                          {"PF80": {"workstream": "delivery", "status": "in_progress",
                                    "dependencies": []}}, {})

    def poll(self, web=None, **kwargs):
        options = dict(PIN, threads={THREAD: "DEC-001"})
        options.update(kwargs)
        return collect(web or Web(), self.c, **options)

    def events(self):
        with self.c.connection() as db:
            return [json.loads(r[0]) for r in db.execute("SELECT body FROM events ORDER BY seq")]

    def test_private_original_evidence_paused_no_authority_or_diagnostics(self):
        out, err = io.StringIO(), io.StringIO()
        before = self.c.snapshot()
        with redirect_stdout(out), redirect_stderr(err):
            receipt = self.poll()
        event, = self.events()
        self.assertEqual(TEXT, event["text"])
        self.assertEqual("DEC-001", event["decision"])
        self.assertEqual("evidence_only", event["authority"])
        self.assertEqual("scan_complete", receipt["status"])
        self.assertNotIn(TEXT, json.dumps(receipt))
        self.assertEqual(("", ""), (out.getvalue(), err.getvalue()))
        self.assertFalse(self.c.snapshot()["enabled"])
        self.assertEqual(before["sprints"], self.c.snapshot()["sprints"])
        self.assertEqual({}, self.c.snapshot()["actions"])
        self.assertEqual(0o700, self.root.stat().st_mode & 0o777)
        self.assertEqual(0o600, self.c.path.stat().st_mode & 0o777)

    def test_duplicates_edits_restart_and_reversion(self):
        self.poll()
        before = self.c.snapshot()
        self.c = Coordinator(self.root)
        self.assertEqual(1, self.poll()["threads"][THREAD]["duplicates"])
        self.assertEqual(before, self.c.snapshot())
        for ts, text in (("1750000002.000001", "Changed"), ("1750000003.000001", TEXT)):
            changed = reply(text=text, edited=dict(user="UHUMAN", ts=ts))
            self.poll(Web([page([parent(), changed])]))
            self.poll(Web([page([parent(), changed])]))
        self.assertEqual(3, len({e["id"] for e in self.events()}))
        # Volatile API metadata cannot change the identity of the same observation.
        self.poll(Web([page([parent(), reply(reactions=[{"name": "eyes"}])])]))
        self.assertEqual(3, len(self.events()))

    def test_identity_denial_before_read_and_every_invocation(self):
        for field, value in (("team_id", "TOTHER"), ("bot_id", "BOTHER"), ("user_id", "UHUMAN")):
            with self.subTest(field=field):
                web = Web()
                web.auth.data[field] = value
                self.assertEqual("identity_mismatch", self.poll(web)["failure"]["code"])
                self.assertEqual([], web.calls)
        self.assertEqual([], self.events())

    def test_thread_channel_team_author_subtype_and_edit_denial(self):
        cases = [dict(thread_ts="1750000009.000001"), dict(channel="COTHER"),
                 dict(team="TOTHER"), dict(team_id="TOTHER"), dict(user="UOTHER"),
                 dict(bot_id="BBOT"), dict(app_id="AAPP"), dict(bot_profile={}),
                 dict(subtype="message_changed"), dict(subtype=[]), dict(ts=THREAD),
                 dict(edited={"ts": "1750000002.000001", "user": "UOTHER"}), dict(edited=None)]
        for changes in cases:
            with self.subTest(changes=changes):
                result = self.poll(Web([page([parent(), reply(**changes)])]))
                self.assertEqual("partial", result["status"])
                self.assertTrue(result["threads"][THREAD]["issues"])
        self.assertEqual([], self.events())

    def test_parent_must_be_exact_pinned_bot_thread(self):
        for changes in (dict(ts="1750000010.000001"), dict(bot_id="BOTHER"),
                        dict(channel="COTHER"), dict(subtype="tombstone")):
            result = self.poll(Web([page([dict(parent(), **changes), reply()])]))
            self.assertEqual("parent_unverified", result["failure"]["code"])
        self.assertEqual([], self.events())

    def test_own_bot_followup_ignored_but_unknown_bot_or_wrong_scope_denied(self):
        echo = reply(user="UBOT", bot_id="BBOT", app_id="AAPP", subtype="bot_message",
                     ts="1750000002.000001", text="details after the human reply")
        result = self.poll(Web([page([parent(), reply(), echo])]))
        self.assertTrue(result["scan_complete"])
        self.assertEqual(1, result["threads"][THREAD]["ignored"])
        self.assertEqual({}, result["threads"][THREAD]["issues"])
        self.assertEqual(1, len(self.events()))
        for change in (dict(bot_id="BOTHER"), dict(user="UOTHER"), dict(channel="COTHER"),
                       dict(team="TOTHER"), dict(thread_ts="1750000099.000001")):
            result = self.poll(Web([page([parent(), dict(echo, **change)])]))
            self.assertFalse(result["scan_complete"])
            self.assertEqual(0, result["threads"][THREAD]["ignored"])
            self.assertTrue(result["threads"][THREAD]["issues"])
        self.assertEqual(1, len(self.events()))

    def test_malformed_oversize_and_deleted_evidence_stays_explicit(self):
        cases = [(None, "malformed"), (reply(text=None), "malformed"),
                 (reply(text="\ud800"), "malformed"), (reply(ts="x"), "malformed"),
                 (reply(text="é" * (MAX_TEXT_BYTES // 2 + 1)), "text_exceeds_bound"),
                 (reply(subtype="tombstone"), "deletion_ambiguous")]
        for message, issue in cases:
            result = self.poll(Web([page([parent(), message])]))
            self.assertEqual({issue: 1}, result["threads"][THREAD]["issues"])
        self.assertEqual([], self.events())
        self.poll(Web([page([parent(), reply(text="é" * (MAX_TEXT_BYTES // 2))])]))
        self.assertEqual(MAX_TEXT_BYTES, len(self.events()[0]["text"].encode()))

    def test_deletion_absence_does_not_retract_old_evidence(self):
        self.poll()
        result = self.poll(Web([page([parent()])]))
        self.assertEqual("unknown", result["deletions"])
        self.assertFalse(result["atomic_snapshot"])
        self.assertEqual(1, len(self.events()))

    def test_pagination_short_empty_pages_and_has_more_false_cursor(self):
        web = Web([page([parent()], "c1", True), page([], "c2"), page([reply()])])
        result = self.poll(web)
        self.assertTrue(result["scan_complete"])
        self.assertEqual(["", "c1", "c2"], [c["cursor"] for c in web.calls])
        self.assertTrue(all(c == dict(channel="CPRIVATE", ts=THREAD, limit=15, cursor=c["cursor"])
                            for c in web.calls))
        self.assertEqual(1, len(self.events()))

    def test_budget_exposes_partial_and_restart_rescans_older_edits(self):
        second = "1750000010.000001"
        routes = {THREAD: "DEC-001", second: "DEC-002"}
        result = self.poll(Web([page(cursor="next", more=True)]), max_pages=1, threads=routes)
        self.assertFalse(result["scan_complete"])
        self.assertEqual("next", result["threads"][THREAD]["next_cursor"])
        self.assertEqual("unscanned", result["threads"][second]["status"])
        self.c = Coordinator(self.root)
        edited = reply(text="old edit", edited=dict(ts="1750000008.000001", user="UHUMAN"))
        web = Web([page([parent(), edited], "next", True),
                   page([reply(ts="1750000004.000001")]), page([parent(second)])])
        result = self.poll(web, threads=routes)
        self.assertTrue(result["scan_complete"])
        self.assertEqual(3, len(self.events()))

    def test_malformed_pagination_fails_without_false_coverage(self):
        variants = [Response(dict(ok=True, messages=[], has_more=True)),
                    Response(dict(ok=True, messages=[parent(), reply()])),
                    Response(dict(ok=True, messages="bad")),
                    Response(dict(ok=True, messages=[], response_metadata=None)),
                    Response(dict(ok=True, messages=[], has_more="false")),
                    page(cursor="bad\ncursor"), page([parent()] * 16), page([])]
        for response in variants:
            result = self.poll(Web([response]))
            self.assertEqual("failed", result["status"])
            self.assertFalse(result["scan_complete"])
        result = self.poll(Web([page(cursor="cycle"), page([], "cycle")]))
        self.assertEqual("cursor_cycle", result["failure"]["code"])

    def test_real_sdk_client_response_and_429_with_network_stubbed(self):
        from slack_sdk import WebClient
        logger = logging.Logger("sdk-offline-poll")
        logger.disabled = True
        web = WebClient(token="offline-fixture", proxy="http://offline.invalid",
                        logger=logger, retry_handlers=[], timeout=2)
        auth = dict(ok=True, team_id="TTEAM", bot_id="BBOT", user_id="UBOT")
        responses = [dict(status=200, headers={}, body=json.dumps(data))
                     for data in (auth, page().data)]
        with patch.object(web, "_perform_urllib_http_request", side_effect=responses) as http:
            self.assertTrue(self.poll(web)["scan_complete"])
        self.assertEqual(2, http.call_count)
        self.assertTrue(http.call_args.kwargs["url"].endswith("/conversations.replies"))
        responses[1] = dict(status=429, headers={"retry-after": "60"},
                            body=json.dumps(dict(ok=False, error="ratelimited")))
        with patch.object(web, "_perform_urllib_http_request", side_effect=responses):
            failure = self.poll(web)["failure"]
        self.assertEqual(dict(code="ratelimited", phase="replies", retry_after=60), failure)

    def test_api_failure_rate_limit_and_no_exception_text_leak(self):
        cases = [(ApiError(Response(dict(ok=False, error="ratelimited"), 429,
                                    {"Retry-After": "60"})), "ratelimited", 60),
                 (Response(dict(ok=False, error="thread_not_found")), "thread_not_found", None),
                 (RuntimeError(TEXT), "api_failure", None),
                 (Response(dict(ok=False, error=TEXT)), "api_failure", None),
                 (Response(dict(ok=False, error=[])), "api_failure", None)]
        for response, code, retry in cases:
            web = Web([page(cursor="c1"), response])
            result = self.poll(web)
            self.assertEqual(dict(code=code, phase="replies", retry_after=retry), result["failure"])
            self.assertEqual("c1", result["threads"][THREAD]["next_cursor"])
            self.assertEqual(2, len(web.calls))
            self.assertNotIn(TEXT, json.dumps(result))

    def test_commit_before_receipt_and_replay_after_uncertain_commit(self):
        original = self.c.event
        def commit_then_interrupt(event):
            original(event)
            raise OSError(TEXT)
        with patch.object(self.c, "event", side_effect=commit_then_interrupt):
            result = self.poll()
        self.assertEqual("persistence_uncertain", result["failure"]["code"])
        self.assertEqual(0, result["threads"][THREAD]["observed"])
        self.c = Coordinator(self.root)
        self.assertEqual(1, self.poll()["threads"][THREAD]["duplicates"])
        with patch.object(self.c, "event", side_effect=OSError(TEXT)):
            result = self.poll(Web([page([parent(), reply(text="new")])]))
        self.assertEqual(0, result["threads"][THREAD]["observed"])
        self.assertEqual(1, len(self.events()))

    def test_invalid_owner_inputs_and_unbounded_client_rejected_without_io(self):
        for changes in (dict(team=""), dict(threads={}), dict(threads={"bad": "DEC"}),
                        dict(threads={THREAD: TEXT}), dict(max_pages=65), dict(max_pages=True),
                        dict(page_size=101)):
            web = Web()
            with self.assertRaises(Rejected):
                self.poll(web, **changes)
            self.assertEqual(0, web.auth_calls)
        for field, value in (("timeout", 31), ("retry_handlers", [object()])):
            web = Web()
            setattr(web, field, value)
            with self.assertRaises(Rejected):
                self.poll(web)
        web = Web()
        web.logger.disabled = False
        with self.assertRaises(Rejected):
            self.poll(web)


if __name__ == "__main__":
    unittest.main()
