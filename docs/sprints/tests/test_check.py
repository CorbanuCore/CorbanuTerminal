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
                f'  - path: "{path}"\n'
                f'    branch: "{branch}"\n'
                f'    base_commit: "{base_commit}"\n'
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
            self.assertTrue(
                any("exactly one PF-NN" in error for error in result["errors"])
            )

    def test_plan_backlink_is_required(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary, backlink=False)
            sprint.write_text(sprint_text(), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("missing sprint backlink" in error for error in result["errors"])
            )

    def test_ready_sprint_requires_active_plan_and_worktree(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(status="ready"), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("requires an active plan" in error for error in result["errors"])
            )
            self.assertTrue(
                any(
                    "requires an exact absolute worktree" in error
                    for error in result["errors"]
                )
            )

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

    def test_parallel_metadata_is_checked_end_to_end(self):
        with tempfile.TemporaryDirectory() as temporary:
            coordinates = ("/tmp/first", "feat/first", "a" * 40)
            repo, root, first = self.make_repo(
                temporary,
                plan_status="active",
                plan_worktree=coordinates,
            )
            second = first.with_name("pf-01-s02-second-task.md")
            plan = repo / "docs/plans/proposed/plan.md"
            plan.write_text(
                plan.read_text()
                .replace(
                    "status: active\n",
                    'status: active\nparallel_sprint_limit: 3\nintegration_owner: "Alex"\n',
                )
                .replace(
                    '    base_commit: "' + "a" * 40 + '"\n',
                    '    base_commit: "' + "a" * 40 + '"\n'
                    '  - path: "/tmp/second"\n    branch: "feat/second"\n'
                    '    base_commit: "' + "b" * 40 + '"\n',
                )
                + f"\n[{second.name}](../../sprints/current/plan/{second.name})\n"
            )
            for index, path in enumerate((first, second), 1):
                value = sprint_text(
                    status="in_progress",
                    worktree=f"/tmp/{'first' if index == 1 else 'second'}",
                    branch=f"feat/{'first' if index == 1 else 'second'}",
                    base_commit=("a" if index == 1 else "b") * 40,
                ).replace(
                    'owner: "Owner"',
                    f'owner: "Worker {index}"\n'
                    f'parallel_lane: "lane-{index}"\n'
                    f'write_scope: "src/module{index}/"\n'
                    'integration_gate: "Alex merges and reruns contract tests"',
                )
                if index == 2:
                    value = value.replace("PF-01-S01", "PF-01-S02").replace(
                        "execution_order: 1", "execution_order: 2"
                    )
                path.write_text(value)
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])
            second.write_text(
                second.read_text().replace("src/module2/", "src/module1/child.rs")
            )
            self.assertTrue(
                any(
                    "overlapping write_scope" in e
                    for e in checker.check_sprints(root, repo)["errors"]
                )
            )

    def test_cycle_and_wrong_order_are_rejected_even_for_drafts(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, first = self.make_repo(temporary)
            second = first.with_name("pf-01-s02-second-task.md")
            first.write_text(
                sprint_text().replace('depends_on: "none"', 'depends_on: "PF-01-S02"')
            )
            second.write_text(
                sprint_text()
                .replace("PF-01-S01", "PF-01-S02")
                .replace("execution_order: 1", "execution_order: 2")
                .replace('depends_on: "none"', 'depends_on: "PF-01-S01"')
            )
            plan = repo / "docs/plans/proposed/plan.md"
            plan.write_text(
                plan.read_text()
                + f"\n[{second.name}](../../sprints/current/plan/{second.name})\n"
            )
            errors = checker.check_sprints(root, repo)["errors"]
            self.assertTrue(any("dependency cycle" in e for e in errors))
            self.assertTrue(any("dependency order" in e for e in errors))

    def test_ready_still_requires_completed_archived_dependency(self):
        with tempfile.TemporaryDirectory() as temporary:
            coordinates = ("/tmp/first", "feat/first", "a" * 40)
            repo, root, first = self.make_repo(
                temporary, plan_status="active", plan_worktree=coordinates
            )
            dependency = first.with_name("pf-01-s02-dependency.md")
            first.write_text(
                sprint_text(
                    status="ready",
                    worktree=coordinates[0],
                    branch=coordinates[1],
                    base_commit=coordinates[2],
                )
                .replace("execution_order: 1", "execution_order: 2")
                .replace('depends_on: "none"', 'depends_on: "PF-01-S02"')
            )
            dependency.write_text(sprint_text().replace("PF-01-S01", "PF-01-S02"))
            plan = repo / "docs/plans/proposed/plan.md"
            plan.write_text(
                plan.read_text()
                + f"\n[{dependency.name}](../../sprints/current/plan/{dependency.name})\n"
            )
            self.assertTrue(
                any(
                    "not completed and archived" in e
                    for e in checker.check_sprints(root, repo)["errors"]
                )
            )
            dependency.unlink()
            archived = root / "archive" / dependency.name
            archived.write_text(
                sprint_text(status="completed")
                .replace("PF-01-S01", "PF-01-S02")
                .replace("- [ ]", "- [x]")
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])


class ParallelAllocationTests(unittest.TestCase):
    def records(self, count=3):
        return [
            dict(
                path=f"sprint-{i}.md",
                lifecycle="current",
                status="in_progress",
                plan_file="plan.md",
                owner=f"Named worker {i}",
                parallel_lane=f"lane-{i}",
                worktree=f"/tmp/worker-{i}",
                branch=f"feat/worker-{i}",
                write_scope=f"src/module-{i}/,tests/module-{i}.py",
                integration_gate="Alex reviews scope, merges and reruns contract tests",
            )
            for i in range(count)
        ]

    def check(self, records, **values):
        return checker.check_parallel(
            records,
            {
                "plan.md": {
                    "parallel_sprint_limit": "3",
                    "integration_owner": "Alex",
                    **values,
                }
            },
        )

    def test_three_independent_allocations_pass(self):
        self.assertEqual(self.check(self.records()), [])

    def test_global_and_per_plan_limits(self):
        self.assertTrue(
            any("global reserved" in e for e in self.check(self.records(4)))
        )
        self.assertTrue(
            any(
                "plan limit 2" in e
                for e in self.check(self.records(), parallel_sprint_limit="2")
            )
        )
        self.assertTrue(
            any(
                "plan limit 1" in e
                for e in checker.check_parallel(self.records(2), {"plan.md": {}})
            )
        )

    def test_global_limit_applies_across_plans(self):
        records = self.records(4)
        for record in records[2:]:
            record["plan_file"] = "other.md"
        plans = {
            name: {"parallel_sprint_limit": "2", "integration_owner": "Alex"}
            for name in ("plan.md", "other.md")
        }
        self.assertTrue(
            any("global reserved" in e for e in checker.check_parallel(records, plans))
        )

    def test_blocked_keeps_reservation_ready_does_not(self):
        records = self.records(4)
        records[-1]["status"] = "blocked"
        self.assertTrue(any("global reserved" in e for e in self.check(records)))
        records[-1]["status"] = "ready"
        self.assertEqual(self.check(records), [])

    def test_invalid_limits_and_missing_integration_owner(self):
        for value in ("0", "4", "three", "", "1.5"):
            with self.subTest(value=value):
                self.assertTrue(
                    any(
                        "must be 1, 2, or 3" in e
                        for e in self.check([], parallel_sprint_limit=value)
                    )
                )
        self.assertTrue(
            any(
                "integration_owner" in e
                for e in self.check([], integration_owner="UNALLOCATED")
            )
        )

    def test_required_parallel_fields_even_for_first_worker_in_opted_in_plan(self):
        for key in ("owner", "parallel_lane", "write_scope", "integration_gate"):
            with self.subTest(key=key):
                records = self.records(1)
                records[0][key] = "UNALLOCATED"
                self.assertTrue(any(key in e for e in self.check(records)))

    def test_duplicate_owners_lanes_worktrees_and_branches(self):
        for key in ("owner", "parallel_lane", "worktree", "branch"):
            with self.subTest(key=key):
                records = self.records(2)
                records[1][key] = " " + records[0][key].upper() + " "
                self.assertTrue(
                    any(f"shared parallel {key}" in e for e in self.check(records))
                )

    def test_scope_overlap_includes_parent_prefix_and_case(self):
        for value in (
            "src/module-0/a.rs",
            "SRC/MODULE-0/",
            "src/",
            "tests/module-0.py",
        ):
            with self.subTest(value=value):
                records = self.records(2)
                records[1]["write_scope"] = value
                self.assertTrue(
                    any("overlapping write_scope" in e for e in self.check(records))
                )

    def test_invalid_scope_paths(self):
        for value in (
            "/",
            ".",
            "../src",
            "src/../file",
            "src/*",
            "src/[ab]",
            "src/**",
            "C:/src",
            "src\\file",
            "src//file",
            "",
            "UNALLOCATED",
        ):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    checker.write_paths(value)

    def test_default_single_worker_does_not_need_parallel_metadata(self):
        record = self.records(1)[0]
        for key in ("parallel_lane", "write_scope", "integration_gate"):
            del record[key]
        self.assertEqual(checker.check_parallel([record], {"plan.md": {}}), [])


if __name__ == "__main__":
    unittest.main()
