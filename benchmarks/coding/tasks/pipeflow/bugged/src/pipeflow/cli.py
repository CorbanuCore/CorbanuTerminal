from __future__ import annotations

import argparse
import json
import sys

from .scheduler import PipelineRunner


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="pipeflow")
    sub = parser.add_subparsers(dest="command", required=True)
    run_cmd = sub.add_parser("run")
    run_cmd.add_argument("config")
    run_cmd.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if args.command == "run":
        result = PipelineRunner(args.config).run()
        if args.json:
            print(json.dumps(result, sort_keys=True))
        else:
            print("ok", result["order"])
        return 0
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
