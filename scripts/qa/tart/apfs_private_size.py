#!/usr/bin/env python3
"""Print the bytes a path holds privately on APFS, i.e. not shared with its clone source.

Uses getattrlist(ATTR_CMNEXT_PRIVATESIZE); `du` would count shared clone blocks as used.
Usage: apfs_private_size.py PATH...   (files or directories; prints the total in bytes)
"""

import ctypes
import ctypes.util
import os
import struct
import sys

ATTR_CMN_RETURNED_ATTRS = 0x80000000
ATTR_CMNEXT_PRIVATESIZE = 0x00000008
FSOPT_ATTR_CMN_EXTENDED = 0x00000020
libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)


class AttrList(ctypes.Structure):
    _fields_ = [
        ("bitmapcount", ctypes.c_ushort),
        ("reserved", ctypes.c_uint16),
        ("commonattr", ctypes.c_uint32),
        ("volattr", ctypes.c_uint32),
        ("dirattr", ctypes.c_uint32),
        ("fileattr", ctypes.c_uint32),
        ("forkattr", ctypes.c_uint32),
    ]


def private_size(path):
    request = AttrList(5, 0, ATTR_CMN_RETURNED_ATTRS, 0, 0, 0, ATTR_CMNEXT_PRIVATESIZE)
    buf = ctypes.create_string_buffer(64)
    if (
        libc.getattrlist(
            path.encode(), ctypes.byref(request), buf, len(buf), FSOPT_ATTR_CMN_EXTENDED
        )
        != 0
    ):
        err = ctypes.get_errno()
        raise OSError(err, os.strerror(err), path)
    returned = struct.unpack_from(
        "5I", buf, 4
    )  # after the u_int32 length: attribute_set_t
    return (
        struct.unpack_from("q", buf, 24)[0]
        if returned[4] & ATTR_CMNEXT_PRIVATESIZE
        else 0
    )


def files(root):
    if os.path.isfile(root):
        yield root
        return
    for base, _dirs, names in os.walk(root):
        for name in names:
            path = os.path.join(base, name)
            if os.path.isfile(path) and not os.path.islink(path):
                yield path


if __name__ == "__main__":
    print(sum(private_size(path) for root in sys.argv[1:] for path in files(root)))
