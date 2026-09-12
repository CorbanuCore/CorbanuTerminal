import copy
import unittest
from html.parser import HTMLParser

import attention


class Links(HTMLParser):
    def __init__(self, page):
        super().__init__()
        self.hrefs = []
        self.feed(page)

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            self.hrefs.append(dict(attrs)["href"])


class AttentionTests(unittest.TestCase):
    def setUp(self):
        self.sprints = [dict(sprint_id="PF-80-S01", path="docs/sprints/current/delivery.md"),
                        dict(sprint_id="PF-76-S01", path="docs/sprints/current/provider.md")]
        self.documents = {s["path"]: "published" for s in self.sprints}
        self.documents[attention.HISTORY] = "history"

    def render(self, items):
        return attention.render(items, self.sprints, self.documents)

    def test_legacy_links_history_not_provider_and_explains_impact(self):
        page = self.render([attention.issue(attention.LEGACY, 10)])
        self.assertIn(attention.document_url(attention.HISTORY), Links(page).hrefs)
        self.assertNotIn(attention.document_url(self.sprints[1]["path"]), Links(page).hrefs)
        self.assertIn(attention.document_url(self.sprints[0]["path"]), Links(page).hrefs)
        for phrase in ("Manager bookkeeping", "No decision needed from you", "Corbanu manager", "Inventory historical", "does not block current", "10 source warnings"):
            self.assertIn(phrase, page)
        self.assertNotIn("Question for you", page)
        self.assertNotIn("Manager attention required", page)

    def test_missing_legacy_document_never_links_modern_provider(self):
        del self.documents[attention.HISTORY]
        page = self.render([attention.issue(attention.LEGACY)])
        self.assertIn('href="#attention-context-PF-76-S01"', page)
        self.assertNotIn(attention.document_url(self.sprints[1]["path"]), page)

    def test_known_unknown_and_ambiguous_references_link_honestly(self):
        item = attention.issue("PF-80-S01 requires PF-99-S99")
        page = self.render([item])
        self.assertIn(attention.document_url(self.sprints[0]["path"]), page)
        self.assertIn('id="attention-context-PF-99-S99"', page)
        self.assertIn("No unique published context", page)
        self.sprints.append(dict(sprint_id="PF-80-S01", path="docs/duplicate.md"))
        self.documents["docs/duplicate.md"] = "duplicate"
        self.assertIn('href="#attention-context-PF-80-S01"', self.render([item]))

    def test_modern_pf76_reference_is_not_implicitly_historical(self):
        page = self.render([attention.issue("PF-76-S01 validation failed")])
        self.assertIn(attention.document_url(self.sprints[1]["path"]), page)
        self.assertNotIn(attention.document_url(attention.HISTORY), page)

    def test_explicit_question_is_read_only_and_all_its_refs_are_linked(self):
        item = attention.issue("PF-80-S01 test decision")
        item.update(owner="Travis", background="Synthetic decision fixture, not a live request.",
                    impact="Waiting delays the synthetic pilot, not unrelated work.",
                    next_step="Manager records the answer before allocating approved work.",
                    question="For PF-80-S01, should the synthetic pilot have five testers or ten? Five is recommended to bound triage time.")
        page = self.render([item])
        self.assertIn("Needs your decision", page)
        self.assertIn("Question for you", page)
        self.assertIn("five testers or ten?", page)
        self.assertIn("Slack reply routing is not connected yet", page)
        self.assertEqual(Links(page).hrefs.count(attention.document_url(self.sprints[0]["path"])), 2)
        self.assertNotIn("<form", page)
        self.assertNotIn("<input", page)

    def test_all_fields_escape_untrusted_markup(self):
        item = attention.issue('<script>bad</script> PF-99-S99')
        for field in ("owner", "kind", "background", "impact", "next_step", "question"):
            item[field] = '<img src=x onerror="bad()"> PF-99-S99'
        page = self.render([item])
        self.assertNotIn("<img", page)
        self.assertNotIn("<script>", page)
        self.assertIn("&lt;img", page)
        self.assertTrue(all(h.startswith("#attention") for h in Links(page).hrefs))

    def test_grouping_is_stable_and_never_mutates_inputs_or_suppresses_other_notices(self):
        data = dict(problems=[attention.LEGACY] * 10 + ["Validation PF-80-S01"],
                    sprints={"sprints": self.sprints}, documents=self.documents)
        original = copy.deepcopy(data)
        page = attention.notices(data)
        self.assertEqual(page.count('<details class="attention-item"'), 2)
        self.assertNotIn('<details class="attention-item" open', page)
        self.assertEqual(data, original)
        self.assertEqual(page, attention.notices(data))
        self.assertEqual(attention.render([], [], {}), "")

    def test_rejected_and_unreadable_reports_have_specific_explanations(self):
        page = self.render([attention.issue("A worker report was rejected; manager inspection required."),
                            attention.issue("Writeback status is unreadable; manager inspection required.")])
        self.assertIn("not proof that the worker stopped", page)
        self.assertIn("Delivery status is unknown", page)
        self.assertIn("do not retry an uncertain external post", page)


if __name__ == "__main__":
    unittest.main()
