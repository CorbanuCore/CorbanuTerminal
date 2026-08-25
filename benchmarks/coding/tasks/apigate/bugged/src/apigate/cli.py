from __future__ import annotations

import argparse
import json
import sys

from .app import create_demo_app
from .types import Request


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="apigate")
    parser.add_argument("method")
    parser.add_argument("path")
    parser.add_argument("--body", default="{}")
    parser.add_argument("--token")
    args = parser.parse_args(argv)
    headers = {"authorization": f"Bearer {args.token}"} if args.token else {}
    body = json.loads(args.body)
    result = create_demo_app().handle(Request(args.method, args.path, headers=headers, body=body))
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
