"""Bounded local inputs for named Bazel evidence, adapted from citadelculture #1394.

No remote fetches or raw diagnostics. The caller owns the testlogs directory:
these lexical/link checks are not an OS sandbox against concurrent mutation.
"""

import os
from pathlib import Path
import stat
from urllib.parse import unquote, urlsplit
from urllib.request import url2pathname

MAX_LOG_BYTES = 16 * 1024 * 1024


class EvidenceError(Exception):
    """A fixed public-safe category, never an input path or raw output."""


def is_link(path):
    return path.is_symlink() or getattr(path, "is_junction", lambda: False)()


def bounded_read(path, limit):
    """Read a regular local file, refusing links and oversized inputs."""
    if type(limit) is not int or limit < 0:
        raise EvidenceError("invalid_size_limit")
    path = Path(path)
    try:
        if is_link(path):
            raise EvidenceError("symlink_rejected")
        before = path.stat(follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode):
            raise EvidenceError("nonregular_file")
        flags = (
            os.O_RDONLY
            | getattr(os, "O_NOFOLLOW", 0)
            | getattr(os, "O_NONBLOCK", 0)
            | getattr(os, "O_BINARY", 0)
        )
        with os.fdopen(os.open(path, flags), "rb") as stream:
            metadata = os.fstat(stream.fileno())
            if not stat.S_ISREG(metadata.st_mode):
                raise EvidenceError("nonregular_file")
            if (metadata.st_dev, metadata.st_ino) != (before.st_dev, before.st_ino):
                raise EvidenceError("file_changed")
            if metadata.st_size > limit:
                raise EvidenceError("size_limit")
            data = stream.read(limit + 1)
    except (OSError, ValueError):
        raise EvidenceError("missing_local_file") from None
    if len(data) > limit:
        raise EvidenceError("size_limit")
    return data


def local_log(uri, root, remaining_bytes):
    """Read only an explicit local BEP reference beneath the owned log root."""
    if not isinstance(uri, str) or len(uri) > 8192:
        raise EvidenceError("invalid_reference")
    try:
        parsed = urlsplit(uri)
    except ValueError:
        raise EvidenceError("invalid_reference") from None
    if parsed.scheme != "file" or parsed.netloc not in ("", "localhost"):
        raise EvidenceError("unavailable_nonlocal_output")
    if parsed.query or parsed.fragment:
        raise EvidenceError("invalid_reference")
    decoded = unquote(parsed.path)
    if "\x00" in decoded or ".." in decoded.replace("\\", "/").split("/"):
        raise EvidenceError("path_escape")
    try:
        path = Path(url2pathname(parsed.path))
    except (OSError, ValueError):
        raise EvidenceError("invalid_reference") from None
    if not path.is_absolute():
        raise EvidenceError("invalid_reference")
    try:
        # The caller may supply Bazel's workspace testlogs symlink as the root.
        root = Path(root).resolve(strict=True)
        relative = path.relative_to(root)
    except ValueError:
        raise EvidenceError("path_escape") from None
    except OSError:
        raise EvidenceError("unavailable_local_root") from None
    if not relative.parts:
        raise EvidenceError("invalid_reference")
    try:
        current = root
        for part in relative.parts:
            current = current / part
            if is_link(current):
                raise EvidenceError("symlink_rejected")
        path.resolve(strict=True).relative_to(root)
    except ValueError:
        raise EvidenceError("path_escape") from None
    except OSError:
        raise EvidenceError("missing_local_file") from None
    return bounded_read(path, min(MAX_LOG_BYTES, remaining_bytes))
