#!/usr/bin/env python3

import unittest

from check_blob_size import ChangedBlob
from check_blob_size import blob_status
from check_blob_size import violates_size_policy


MAX_BYTES = 500


def blob(
    size_bytes: int,
    base_size_bytes: int | None,
    *,
    is_allowlisted: bool = False,
) -> ChangedBlob:
    return ChangedBlob(
        path="fixture.txt",
        size_bytes=size_bytes,
        base_size_bytes=base_size_bytes,
        is_allowlisted=is_allowlisted,
        is_binary=False,
    )


class BlobSizePolicyTest(unittest.TestCase):
    def test_blocks_new_oversized_blob(self) -> None:
        self.assertTrue(violates_size_policy(blob(501, None), MAX_BYTES))

    def test_blocks_blob_that_crosses_limit(self) -> None:
        self.assertTrue(violates_size_policy(blob(501, 499), MAX_BYTES))

    def test_blocks_growth_of_grandfathered_blob(self) -> None:
        self.assertTrue(violates_size_policy(blob(601, 600), MAX_BYTES))

    def test_allows_grandfathered_blob_that_does_not_grow(self) -> None:
        unchanged = blob(600, 600)
        shrunk = blob(550, 600)

        self.assertFalse(violates_size_policy(unchanged, MAX_BYTES))
        self.assertFalse(violates_size_policy(shrunk, MAX_BYTES))
        self.assertEqual(
            blob_status(unchanged, MAX_BYTES, is_violation=False),
            "grandfathered (not grown)",
        )

    def test_allowlist_remains_an_explicit_override(self) -> None:
        allowlisted = blob(700, None, is_allowlisted=True)

        self.assertFalse(violates_size_policy(allowlisted, MAX_BYTES))
        self.assertEqual(
            blob_status(allowlisted, MAX_BYTES, is_violation=False), "allowlisted"
        )


if __name__ == "__main__":
    unittest.main()
