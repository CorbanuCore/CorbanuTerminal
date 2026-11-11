"""Stage 2 of citadelculture's #1394: bounded, local, owned input files."""

import os
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from bazel_evidence_io import EvidenceError, bounded_read, local_log


class EvidenceIOTest(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.logs = self.root / "testlogs"
        self.logs.mkdir()
        self.log = self.logs / "test.log"
        self.log.write_bytes(b"fixture")

    def rejected(self, category, fn, *args):
        with self.assertRaises(EvidenceError) as caught:
            fn(*args)
        self.assertEqual(str(caught.exception), category)
        self.assertNotIn(str(self.root), str(caught.exception))

    def test_exact_limit_empty_and_oversized(self):
        self.assertEqual(bounded_read(self.log, 7), b"fixture")
        self.rejected("size_limit", bounded_read, self.log, 6)
        self.log.write_bytes(b"")
        self.assertEqual(bounded_read(self.log, 0), b"")
        for limit in (-1, True, 1.5):
            self.rejected("invalid_size_limit", bounded_read, self.log, limit)

    def test_missing_directory_and_fifo_do_not_block(self):
        self.rejected("missing_local_file", bounded_read, self.root / "absent", 10)
        self.rejected("nonregular_file", bounded_read, self.logs, 10)
        if hasattr(os, "mkfifo"):
            fifo = self.logs / "pipe"
            os.mkfifo(fifo)
            self.rejected("nonregular_file", bounded_read, fifo, 10)

    def test_local_uri_and_shared_budget(self):
        self.assertEqual(local_log(self.log.as_uri(), self.logs, 7), b"fixture")
        self.rejected("size_limit", local_log, self.log.as_uri(), self.logs, 6)
        with patch("bazel_evidence_io.MAX_LOG_BYTES", 5):
            self.rejected("size_limit", local_log, self.log.as_uri(), self.logs, 7)

    def test_nonlocal_and_invalid_references_are_never_opened(self):
        for uri, category in [
            ("https://example.invalid/test.log", "unavailable_nonlocal_output"),
            ("file://remotehost/test.log", "unavailable_nonlocal_output"),
            (self.log.as_uri() + "?secret", "invalid_reference"),
            (self.log.as_uri() + "#secret", "invalid_reference"),
            ("file:relative", "invalid_reference"),
            ("file:///tmp/%00secret", "path_escape"),
            ("file:///tmp/%2e%2e/secret", "path_escape"),
            ("file:///tmp/%5c..%5csecret", "path_escape"),
            (None, "invalid_reference"),
            ("a" * 8193, "invalid_reference"),
        ]:
            with self.subTest(category=category), patch("os.open") as opened:
                self.rejected(category, local_log, uri, self.logs, 100)
                opened.assert_not_called()

    def test_conversion_and_component_errors_are_sanitized(self):
        with patch("bazel_evidence_io.url2pathname", side_effect=OSError("private URI")):
            self.rejected("invalid_reference", local_log, "file:///1:/private/test.log", self.logs, 100)
        with patch("bazel_evidence_io.is_link", side_effect=PermissionError("private component")):
            self.rejected("missing_local_file", local_log, self.log.as_uri(), self.logs, 100)

    def test_sibling_prefix_and_missing_roots(self):
        outside = self.root / "testlogs-other"
        outside.mkdir()
        path = outside / "test.log"
        path.write_bytes(b"outside")
        self.rejected("path_escape", local_log, path.as_uri(), self.logs, 100)
        self.rejected("invalid_reference", local_log, self.logs.as_uri(), self.logs, 100)
        self.rejected("unavailable_local_root", local_log, self.log.as_uri(), self.root / "absent", 100)

    def test_no_follow_for_file_and_parent_links(self):
        link = self.logs / "link.log"
        try:
            link.symlink_to(self.log)
        except OSError:
            self.skipTest("host does not grant fixture symlink creation")
        self.rejected("symlink_rejected", bounded_read, link, 100)
        self.rejected("symlink_rejected", local_log, link.as_uri(), self.logs, 100)
        parent = self.logs / "parent"
        parent.symlink_to(self.root, target_is_directory=True)
        self.rejected("symlink_rejected", local_log, (parent / "testlogs" / "test.log").as_uri(), self.logs, 100)

    def test_bazel_root_alias_is_explicitly_owned(self):
        alias = self.root / "bazel-testlogs"
        try:
            alias.symlink_to(self.logs, target_is_directory=True)
        except OSError:
            self.skipTest("host does not grant fixture symlink creation")
        self.assertEqual(local_log(self.log.as_uri(), alias, 7), b"fixture")


if __name__ == "__main__":
    unittest.main()
