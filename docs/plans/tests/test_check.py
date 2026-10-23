import importlib.util
import tempfile
import unittest
from pathlib import Path


CHECKER_PATH = Path(__file__).resolve().parents[1] / "check.py"
SPEC = importlib.util.spec_from_file_location("plan_checker", CHECKER_PATH)
assert SPEC and SPEC.loader
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


def active_plan(title):
    sections = "\n\n".join(
        f"## {section}\n\nEvidence pending."
        for section in checker.REQUIRED_ACTIVE_SECTIONS
    )
    return f"""---
title: "{title}"
status: active
change_class: product-initiative
priority: P0
owner: "Owner"
activation_authority: "Authority"
activation_basis: "Authorized product decision"
target_release: "TBD"
deadline: 2026-10-08
created: 2026-08-23
updated: 2026-08-23
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "P0 /security levels"
  requirement_excerpt: "Deterministic policy outside the model."
implementation_worktrees:
  - path: "/workspace/corbanu-security"
    branch: "feat/security"
    base_commit: "1111111111111111111111111111111111111111"
---

# {title}

{sections}
"""


class PlanCheckerTests(unittest.TestCase):
    def make_root(self, temporary):
        root = Path(temporary)
        for directory in checker.LIFECYCLE_STATUS:
            (root / directory).mkdir()
        return root

    def test_one_complete_active_plan_passes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = self.make_root(temporary)
            (root / "active" / "index.md").write_text(
                "# Active plans\n\nLifecycle navigation only.\n",
                encoding="utf-8",
            )
            (root / "active" / "security.md").write_text(
                active_plan("Security"),
                encoding="utf-8",
            )
            result = checker.check_plan_root(root)
            self.assertTrue(result["ok"], result["errors"])
            self.assertEqual(result["active_count"], 1)
            self.assertEqual(result["available_slots"], 2)

    def test_fourth_active_plan_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = self.make_root(temporary)
            for number in range(4):
                (root / "active" / f"plan-{number}.md").write_text(
                    active_plan(f"Plan {number}"),
                    encoding="utf-8",
                )
            result = checker.check_plan_root(root)
            self.assertFalse(result["ok"])
            self.assertIn(
                "active-plan limit exceeded: found 4, maximum is 3",
                result["errors"],
            )

    def test_status_must_match_lifecycle_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = self.make_root(temporary)
            text = active_plan("Misplaced").replace("status: active", "status: draft")
            (root / "active" / "misplaced.md").write_text(text, encoding="utf-8")
            result = checker.check_plan_root(root)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("does not match directory" in error for error in result["errors"])
            )

    def test_concurrency_metadata_before_sprint_allocation(self):
        three = "broker, untrusted-content, tui"
        for limit, owner, lanes, valid in (
            (1, "Owner", "", True),
            (2, "Owner", "broker, tui", True),
            (3, "Owner", three, True),
            (3, "Owner", "", False),
            (3, "Owner", "broker, tui", False),
            (3, "Owner", "broker, Broker, tui", False),
            (2, "", "broker, tui", False),
            (0, "Owner", three, False),
            (4, "Owner", three + ", extra", False),
            ("many", "Owner", three, False),
            (3, "<owner>", three, False),
        ):
            with (
                self.subTest(limit=limit, owner=owner, lanes=lanes),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root = self.make_root(temporary)
                text = active_plan("Security").replace(
                    "status: active",
                    f'status: active\nparallel_sprint_limit: {limit}\n'
                    f'parallel_lanes: "{lanes}"\nintegration_owner: "{owner}"',
                )
                (root / "active/security.md").write_text(text, encoding="utf-8")
                result = checker.check_plan_root(root)
                self.assertEqual(result["ok"], valid, result["errors"])

    def test_max_active_sprints_must_match_limit(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = self.make_root(temporary)
            text = active_plan("Security").replace(
                "status: active",
                'status: active\nparallel_sprint_limit: 3\nmax_active_sprints: 1\n'
                'parallel_lanes: "a, b, c"\nintegration_owner: "Owner"',
            )
            (root / "active/security.md").write_text(text, encoding="utf-8")
            result = checker.check_plan_root(root)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("parallel_sprint_limit must be 1-3" in e for e in result["errors"])
            )

    def test_three_active_plans_pass(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = self.make_root(temporary)
            for number in range(3):
                (root / "active" / f"plan-{number}.md").write_text(active_plan(str(number)))
            result = checker.check_plan_root(root)
            self.assertTrue(result["ok"], result["errors"])
            self.assertEqual(result["available_slots"], 0)


if __name__ == "__main__":
    unittest.main()
