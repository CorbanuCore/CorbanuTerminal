#!/usr/bin/env python3
"""Turn frozen-config.json into runnable benchmarks/coding/runner.py configs.

Two modes:

  refresh   Re-derive the coding model set and the planning spend estimate
            from a models.json catalogue and write them into frozen-config.json.
            Run it on the candidate commit before freezing; review the diff.

  generate  Write runner configs with absolute paths for this machine:
            coding-matrix.json (every catalogue task x every frozen Corbanu
            route) and hermes-efficiency.json (Corbanu, Hermes and Kilo on the
            shared OpenRouter route). Checks the task catalogue digest first.
            --dry-run swaps each harness binary for dry_run_stub.py, which
            records the argv and runs only `<binary> --version`.

No mode makes a model or provider call, reads a credential, or prints one.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]  # qa/release/<version>/benchmarks/ -> repository root
RUNNER = REPO / "benchmarks" / "coding" / "runner.py"
FREEZE = HERE / "frozen-config.json"


def load_runner() -> Any:
    spec = importlib.util.spec_from_file_location("bench_runner", RUNNER)
    if spec is None or spec.loader is None:
        raise SystemExit(f"cannot load {RUNNER}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = (
        module  # dataclasses resolve annotations through sys.modules
    )
    spec.loader.exec_module(module)
    return module


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def catalog_tasks(freeze: dict[str, Any]) -> list[dict[str, Any]]:
    catalog = REPO / freeze["coding_matrix"]["task_catalog"]["config"]
    base = catalog.parent
    tasks = []
    for raw in read_json(catalog)["tasks"]:
        task = dict(raw)
        for key in ("baseline", "prompt", "verifier"):
            task[key] = str((base / raw[key]).resolve())
        tasks.append(task)
    return tasks


def catalog_digests(runner: Any, tasks: list[dict[str, Any]]) -> dict[str, str]:
    digests = {}
    for task in tasks:
        task_dir = Path(task["verifier"]).parent.parent
        digests[task["name"]] = runner.tree_digest(task_dir)
    return digests


# ---------------------------------------------------------------- refresh


def route_price(model: dict[str, Any], fallback: dict[str, Any]) -> dict[str, Any]:
    billing = (model.get("orchestration") or {}).get("billing") or {}
    note = None
    if "peak" in billing:
        # Time-of-day schedule: estimate at the peak rate; actual spend uses the
        # schedule in the catalogue row at each request's timestamp.
        billing, note = billing["peak"], "peak rate of a time-of-day schedule"
    keys = ("input", "cached_input", "output")
    if billing.get("input_milli_usd_per_million_tokens") is not None:
        price: dict[str, Any] = {}
        for key in keys:
            raw = billing.get(f"{key}_milli_usd_per_million_tokens")
            # A missing cached price stays None: run_cost then charges cached
            # tokens at the full input price instead of treating them as free.
            price[key] = None if raw is None else raw / 1000
        price["source"] = "catalogue"
        if note:
            price["note"] = note
        return price
    slug = model["slug"]
    if slug in fallback:
        return {
            **{key: fallback[slug][key] for key in keys},
            "source": "openrouter-snapshot",
        }
    return {"source": "none"}


def run_cost(price: dict[str, Any], profile: dict[str, Any]) -> float | None:
    if price.get("source") == "none":
        return None
    cached = price.get("cached_input")
    if cached is None:
        cached = price["input"]
    return (
        profile["uncached_input_tokens"] * price["input"]
        + profile["cached_input_tokens"] * cached
        + profile["output_tokens"] * price["output"]
    ) / 1_000_000


def refresh(freeze: dict[str, Any], models_path: Path) -> dict[str, Any]:
    matrix = freeze["coding_matrix"]
    rules = matrix["model_set_rule"]
    profile = freeze["spend_estimate"]["per_run_token_profile"]
    multiplier = freeze["spend_estimate"]["planning_multiplier"]
    fallback = freeze["price_source"]["openrouter_equivalents"]
    env_by_provider = rules["required_env_by_provider"]
    excluded = set(matrix.get("approved_exclusions") or [])
    tasks = len(read_json(REPO / matrix["task_catalog"]["config"])["tasks"])
    routes = []
    for model in read_json(models_path)["models"]:
        orchestration = model.get("orchestration") or {}
        if model.get("visibility") != "list":
            continue
        if model["slug"] in excluded:
            continue
        provider = orchestration.get("provider_id") or ""
        price = route_price(model, fallback)
        per_run = run_cost(price, profile)
        routes.append(
            {
                "name": f"corbanu--{provider}--{model['slug']}".replace("/", "_"),
                "provider": provider,
                "model": model["slug"],
                "reasoning": model.get("default_reasoning_level"),
                "reasoning_levels": [
                    level.get("effort")
                    for level in model.get("supported_reasoning_levels") or []
                ],
                "orchestration_status": orchestration.get("status"),
                "capability": orchestration.get("capability"),
                "billing_kind": ((orchestration.get("billing") or {}).get("kind")),
                "required_env": env_by_provider.get(provider, []),
                "price_usd_per_million": price,
                "estimated_usd_per_run": None
                if per_run is None
                else round(per_run * multiplier, 4),
            }
        )
    known = [
        route["estimated_usd_per_run"]
        for route in routes
        if route["estimated_usd_per_run"] is not None
    ]
    matrix["model_set"] = routes
    matrix["model_set_source"] = str(models_path.resolve().relative_to(REPO))
    matrix["planned_runs"] = tasks * len(routes) * int(matrix["waves"])
    freeze["spend_estimate"]["coding_matrix_usd"] = round(
        sum(known) * tasks * int(matrix["waves"]), 2
    )
    freeze["spend_estimate"]["coding_matrix_plan_routes_usd"] = round(
        sum(
            route["estimated_usd_per_run"] or 0
            for route in routes
            if route["billing_kind"] == "plan"
        )
        * tasks
        * int(matrix["waves"]),
        2,
    )
    freeze["spend_estimate"]["coding_matrix_unpriced_routes"] = [
        route["model"] for route in routes if route["estimated_usd_per_run"] is None
    ]

    efficiency = freeze["hermes_efficiency"]
    shared = fallback[efficiency["route"]["model"]]
    shared_run = (
        run_cost({**shared, "source": "openrouter-snapshot"}, profile) * multiplier
    )
    harnesses = 3
    efficiency["planned_runs"] = tasks * int(efficiency["waves"]) * harnesses
    freeze["spend_estimate"]["shared_route_usd_per_run"] = round(shared_run, 4)
    freeze["spend_estimate"]["hermes_efficiency_usd"] = round(
        shared_run * efficiency["planned_runs"], 2
    )

    competitive = freeze["competitive"]
    live_runs = int(competitive["repeats"]) * harnesses
    freeze["spend_estimate"]["competitive_usd"] = round(
        shared_run * float(competitive["estimate_run_multiplier"]) * live_runs, 2
    )
    website = freeze["website_builder"]
    freeze["spend_estimate"]["website_builder_usd"] = round(
        len(website["waves"])
        * (website["claude_max_budget_usd"] + website["corbanu_estimated_usd_per_wave"])
        + website["judge_and_images_estimated_usd"],
        2,
    )
    parts = (
        "coding_matrix_usd",
        "hermes_efficiency_usd",
        "competitive_usd",
        "website_builder_usd",
    )
    freeze["spend_estimate"]["total_usd"] = round(
        sum(freeze["spend_estimate"][key] for key in parts), 2
    )
    return freeze


# --------------------------------------------------------------- generate


def wrapped(helper: Path, options: list[str], argv: list[str]) -> list[str]:
    return ["{python}", str(helper), *options, "--", "{binary}", *argv]


def corbanu_agent(
    helper: Path, route: dict[str, Any], lane: str, name: str
) -> dict[str, Any]:
    options = [
        "--home",
        "HOME={result_dir}/home",
        "--home",
        "CODEX_HOME={result_dir}/corbanu-home",
        "--home",
        "CORBANU_HOME={result_dir}/corbanu-home",
        "--unset",
        "PFTERMINAL_HOME",
        "--set",
        "CORBANU_TRACE_STREAM_TIMING=1",
        "--set",
        "CORBANU_DUMP_CHAT_REQUEST={result_dir}/corbanu.chat.request.json",
        "--set",
        "CORBANU_DUMP_ANTHROPIC_REQUEST={result_dir}/corbanu.anthropic.request.json",
        "--set",
        "CORBANU_DUMP_RESPONSES_REQUEST={result_dir}/corbanu.responses.request.json",
    ]
    argv = [
        "exec",
        "--json",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "-C",
        "{workspace}",
        "-c",
        'model_provider="{provider}"',
    ]
    if route.get("reasoning"):
        argv += ["-c", f'model_reasoning_effort="{route["reasoning"]}"']
    argv += ["-m", "{model}", "-"]
    return {
        "name": name,
        "kind": "corbanu",
        "binary": "corbanu",
        "provider": route["provider"],
        "model": route["model"],
        "lane": lane,
        "required_env": route["required_env"],
        "command": wrapped(helper, options, argv),
        "stdin_prompt": True,
    }


def hermes_agent(
    helper: Path, route: dict[str, Any], hermes_root: Path
) -> dict[str, Any]:
    argv = [
        "--provider",
        "{provider}",
        "-m",
        "{model}",
        "--reasoning",
        route["reasoning"],
        "--yolo",
        "--accept-hooks",
        "--usage-file",
        "{result_dir}/hermes.usage.json",
        "-z",
        "{prompt}",
    ]
    return {
        "name": "hermes",
        "kind": "hermes",
        "binary": "hermes",
        "provider": route["provider"],
        "model": route["hermes_model"],
        "lane": "hermes",
        "required_env": route["required_env"],
        "command": wrapped(
            helper,
            [
                "--home",
                "HOME={result_dir}/home",
                "--home-under",
                f"HERMES_HOME={hermes_root / 'profiles'}::{{result_dir}}",
            ],
            argv,
        ),
        "stdin_prompt": False,
    }


def kilo_agent(helper: Path, route: dict[str, Any]) -> dict[str, Any]:
    options = ["--home", "HOME={result_dir}/home"]
    for var, sub in (
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_STATE_HOME", "state"),
        ("XDG_CACHE_HOME", "cache"),
    ):
        options += ["--home", f"{var}={{result_dir}}/kilo-home/{sub}"]
    options += ["--set-rel", "KILO_CONFIG=kilo-frozen.json"]
    argv = [
        "run",
        "--model",
        "{model}",
        "--variant",
        route["reasoning"],
        "--dir",
        "{workspace}",
        "--format",
        "json",
        "--auto",
        "{prompt}",
    ]
    return {
        "name": "kilo",
        "kind": "kilo",
        "binary": "kilo",
        "provider": route["provider"],
        "model": route["kilo_model"],
        "lane": "kilo",
        "required_env": route["required_env"],
        "command": wrapped(helper, options, argv),
        "stdin_prompt": False,
    }


def bind_binary(
    agent: dict[str, Any], binaries: dict[str, str], dry_run: bool
) -> dict[str, Any]:
    real = binaries[agent["kind"]]
    if not dry_run:
        agent["binary"] = real
        return agent
    command = agent["command"]
    split = command.index("--")
    agent["command"] = (
        command[:split] + ["--set", f"BENCH_DRY_RUN_TARGET={real}"] + command[split:]
    )
    agent["binary"] = str(HERE / "dry_run_stub.py")
    agent["required_env"] = []
    return agent


def generate(freeze: dict[str, Any], args: argparse.Namespace) -> list[Path]:
    runner = load_runner()
    tasks = catalog_tasks(freeze)
    frozen = freeze["coding_matrix"]["task_catalog"]["tree_sha256"]
    actual = catalog_digests(runner, tasks)
    if frozen != actual:
        changed = sorted(
            name
            for name in set(frozen) | set(actual)
            if frozen.get(name) != actual.get(name)
        )
        raise SystemExit(
            f"task catalogue differs from the freeze: {', '.join(changed)}"
        )
    helper = HERE / "isolated_env.py"
    binaries = {
        "corbanu": args.corbanu_bin,
        "hermes": args.hermes_bin,
        "kilo": args.kilo_bin,
    }
    for kind, path in binaries.items():
        if not Path(path).is_file():
            raise SystemExit(f"{kind} binary not found: {path}")
    hermes_root = Path(args.hermes_root).resolve()
    if not (hermes_root / "config.yaml").is_file():
        raise SystemExit(
            f"--hermes-root is not an installed Hermes home: {hermes_root}"
        )
    (hermes_root / "profiles").mkdir(exist_ok=True)
    suffix = "-dry-run" if args.dry_run else ""
    timeout = int(freeze["cutoff"]["coding_run_timeout_seconds"])
    written = []

    matrix = freeze["coding_matrix"]
    agents = [
        bind_binary(
            corbanu_agent(helper, route, route["provider"], route["name"]),
            binaries,
            args.dry_run,
        )
        for route in matrix["model_set"]
    ]
    waves = int(matrix["waves"])
    config = {
        "run_dir": str((args.run_root / f"coding-matrix{suffix}").resolve()),
        "waves": waves,
        "caps": {
            "timeout_seconds": timeout,
            "max_total_runs": len(tasks) * len(agents) * waves,
        },
        "agents": agents,
        "tasks": tasks,
    }
    path = args.out / f"coding-matrix{suffix}.json"
    write_json(path, config)
    written.append(path)

    efficiency = freeze["hermes_efficiency"]
    route = efficiency["route"]
    agents = [
        bind_binary(
            corbanu_agent(helper, route, "corbanu", "corbanu"), binaries, args.dry_run
        ),
        bind_binary(hermes_agent(helper, route, hermes_root), binaries, args.dry_run),
        bind_binary(kilo_agent(helper, route), binaries, args.dry_run),
    ]
    waves = int(efficiency["waves"])
    config = {
        "run_dir": str((args.run_root / f"hermes-efficiency{suffix}").resolve()),
        "waves": waves,
        "caps": {
            "timeout_seconds": timeout,
            "max_total_runs": len(tasks) * len(agents) * waves,
        },
        "agents": agents,
        "tasks": tasks,
    }
    path = args.out / f"hermes-efficiency{suffix}.json"
    write_json(path, config)
    written.append(path)
    return written


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--freeze", type=Path, default=FREEZE)
    sub = parser.add_subparsers(dest="mode", required=True)
    ref = sub.add_parser("refresh")
    ref.add_argument(
        "--models",
        type=Path,
        default=REPO / "codex-rs" / "models-manager" / "models.json",
    )
    ref.add_argument(
        "--check", action="store_true", help="fail if the freeze would change"
    )
    gen = sub.add_parser("generate")
    gen.add_argument("--out", type=Path, required=True)
    gen.add_argument("--run-root", type=Path, required=True)
    gen.add_argument("--corbanu-bin", required=True)
    gen.add_argument("--hermes-bin", required=True)
    gen.add_argument("--kilo-bin", required=True)
    gen.add_argument(
        "--hermes-root",
        required=True,
        help="the Hermes home the pinned install used (--hermes-home); per-run homes go under its profiles/",
    )
    gen.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    freeze = read_json(args.freeze)
    if args.mode == "refresh":
        runner = load_runner()
        before = json.dumps(freeze, sort_keys=True)
        freeze["coding_matrix"]["task_catalog"]["tree_sha256"] = catalog_digests(
            runner, catalog_tasks(freeze)
        )
        freeze = refresh(freeze, args.models)
        if args.check:
            if json.dumps(freeze, sort_keys=True) != before:
                raise SystemExit(
                    "frozen-config.json is out of date; run refresh and review the diff"
                )
            print("frozen-config.json is current")
            return 0
        write_json(args.freeze, freeze)
        print(f"updated {args.freeze}")
        return 0
    for path in generate(freeze, args):
        print(path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
