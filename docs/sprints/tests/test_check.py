import importlib.util
import os
import tempfile
import unittest
from pathlib import Path


CHECKER_PATH = Path(__file__).resolve().parents[1] / "check.py"
SPEC = importlib.util.spec_from_file_location("sprint_checker", CHECKER_PATH)
assert SPEC and SPEC.loader
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


def sprint_text(
    *,
    status="draft",
    feature="PF-01",
    plan_file="docs/plans/proposed/plan.md",
    worktree="UNALLOCATED",
    branch="UNALLOCATED",
    base_commit="UNALLOCATED",
):
    return f"""---
sprint_id: "PF-01-S01"
title: "One task"
status: {status}
plan_file: "{plan_file}"
plan_feature: "{feature}"
execution_order: 1
owner: "Owner"
worktree: "{worktree}"
branch: "{branch}"
base_commit: "{base_commit}"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-01-S01 — One task

## Execution mandate

- Deliver: one task

## Plan linkage

- Feature: `PF-01`

## Code boundaries

- Existing: `code.rs`

## Preconditions

- [ ] Plan active.

## Done

- [x] Sprint contract drafted.

## Remaining

- [ ] Implement one task.

## Verification

- [ ] Focused test passes.

## Exit evidence

- [ ] Commit recorded.
"""


class SprintCheckerTests(unittest.TestCase):
    def make_repo(
        self,
        temporary,
        *,
        backlink=True,
        plan_status="draft",
        plan_worktree=None,
    ):
        repo = Path(temporary)
        sprint_root = repo / "docs" / "sprints"
        sprint_path = sprint_root / "current" / "plan" / "pf-01-s01-one-task.md"
        sprint_path.parent.mkdir(parents=True)
        (sprint_root / "archive").mkdir(parents=True)
        plan_path = repo / "docs" / "plans" / "proposed" / "plan.md"
        plan_path.parent.mkdir(parents=True)
        link = Path(os.path.relpath(sprint_path, plan_path.parent)).as_posix()
        worktree_front = ""
        if plan_worktree is not None:
            path, branch, base_commit = plan_worktree
            worktree_front = (
                "implementation_worktrees:\n"
                f"  - path: \"{path}\"\n"
                f"    branch: \"{branch}\"\n"
                f"    base_commit: \"{base_commit}\"\n"
            )
        plan_path.write_text(
            f"---\nstatus: {plan_status}\n{worktree_front}---\n\n# Plan\n\nPF-01\n"
            + (f"\n[{link}]({link})\n" if backlink else ""),
            encoding="utf-8",
        )
        return repo, sprint_root, sprint_path

    def test_valid_draft_sprint_passes(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])
            self.assertEqual(result["current_count"], 1)

    def test_sprint_must_link_exactly_one_feature(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(feature="PF-01,PF-02"), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(any("exactly one PF-NN" in error for error in result["errors"]))

    def test_plan_backlink_is_required(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary, backlink=False)
            sprint.write_text(sprint_text(), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(any("missing sprint backlink" in error for error in result["errors"]))

    def test_ready_sprint_requires_active_plan_and_worktree(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(status="ready"), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(any("requires an active plan" in error for error in result["errors"]))
            self.assertTrue(any("requires an exact absolute worktree" in error for error in result["errors"]))

    def test_ready_sprint_coordinates_must_match_active_plan(self):
        with tempfile.TemporaryDirectory() as temporary:
            expected = ("/tmp/plan-worktree", "feat/security", "a" * 40)
            repo, root, sprint = self.make_repo(
                temporary,
                plan_status="active",
                plan_worktree=expected,
            )
            sprint.write_text(
                sprint_text(
                    status="ready",
                    worktree="/tmp/other-worktree",
                    branch=expected[1],
                    base_commit=expected[2],
                ),
                encoding="utf-8",
            )
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(any("do not match" in error for error in result["errors"]))

    def test_ready_sprint_with_matching_active_plan_passes(self):
        with tempfile.TemporaryDirectory() as temporary:
            coordinates = ("/tmp/plan-worktree", "feat/security", "a" * 40)
            repo, root, sprint = self.make_repo(
                temporary,
                plan_status="active",
                plan_worktree=coordinates,
            )
            sprint.write_text(
                sprint_text(
                    status="ready",
                    worktree=coordinates[0],
                    branch=coordinates[1],
                    base_commit=coordinates[2],
                ),
                encoding="utf-8",
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])


if __name__ == "__main__":
    unittest.main()
