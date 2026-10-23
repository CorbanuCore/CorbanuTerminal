#!/usr/bin/env python3
"""usage: hold_lock.py <db> <seconds>: hold an EXCLUSIVE SQLite transaction on <db> (another writer keeps the store busy)."""
import sqlite3, sys, time
db = sqlite3.connect(sys.argv[1], timeout=30, isolation_level=None)
db.execute("BEGIN EXCLUSIVE")
print(f"locked {time.strftime('%H:%M:%S', time.gmtime())}Z", flush=True)
time.sleep(float(sys.argv[2]))
db.execute("ROLLBACK")
print(f"released {time.strftime('%H:%M:%S', time.gmtime())}Z", flush=True)
