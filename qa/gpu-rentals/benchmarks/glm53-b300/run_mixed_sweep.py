#!/usr/bin/env python3
"""Closed-loop mixed-context benchmark for the GLM-5.3-Flash B300 preset."""

from __future__ import annotations

import argparse
import asyncio
import csv
import json
import math
import os
import random
import statistics
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit, urlunsplit

MODEL_ID = "zai-org/GLM-5.3-Flash"
MODEL_REVISION = "3f1971b7b5f7a528c9c4ef6212c8785298a8c24a"
DEFAULT_CONCURRENCIES = (4, 8, 16, 32, 64, 128, 256)
SEED = 20_260_826
MAX_CONTEXT_TOKENS = 131_072


@dataclass(frozen=True)
class Bucket:
    name: str
    weight_eighths: int
    input_tokens: int
    output_tokens: int


BUCKETS = (
    Bucket("short", 4, 1_024, 2_000),
    Bucket("medium", 2, 8_192, 6_000),
    Bucket("large", 1, 32_768, 8_000),
    Bucket("long", 1, 96_000, 20_000),
)


@dataclass
class RequestResult:
    request_id: str
    bucket: str
    target_input_tokens: int
    target_output_tokens: int
    input_tokens: int
    output_tokens: int
    ttft_ms: float | None
    tpot_ms: float | None
    e2el_ms: float
    error: str | None = None


def percentile(values: list[float], quantile: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    position = (len(ordered) - 1) * quantile
    lower = math.floor(position)
    upper = math.ceil(position)
    if lower == upper:
        return ordered[lower]
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def weighted_mean(field: str) -> float:
    return sum(
        getattr(bucket, field) * bucket.weight_eighths for bucket in BUCKETS
    ) / 8


def validate_contract(concurrencies: tuple[int, ...]) -> None:
    if not concurrencies:
        raise ValueError("at least one concurrency is required")
    if any(value < 1 or value > 256 for value in concurrencies):
        raise ValueError("concurrencies must be between 1 and 256")
    if any((2 * value) % 8 for value in concurrencies):
        raise ValueError("two waves at each concurrency must divide into eighths")
    if sum(bucket.weight_eighths for bucket in BUCKETS) != 8:
        raise ValueError("bucket weights must total eight eighths")
    if weighted_mean("output_tokens") != 6_000:
        raise ValueError("weighted requested output must equal 6,000 tokens")
    for bucket in BUCKETS:
        if bucket.input_tokens + bucket.output_tokens > MAX_CONTEXT_TOKENS:
            raise ValueError(f"{bucket.name} exceeds the preset context limit")


def workload_for(concurrency: int) -> list[Bucket]:
    request_count = 2 * concurrency
    workload: list[Bucket] = []
    for bucket in BUCKETS:
        workload.extend([bucket] * (request_count * bucket.weight_eighths // 8))
    random.Random(SEED + concurrency).shuffle(workload)
    return workload


def api_urls(base_url: str) -> tuple[str, str]:
    parsed = urlsplit(base_url.rstrip("/"))
    path = parsed.path.rstrip("/")
    if path.endswith("/v1"):
        root_path = path[:-3]
        openai_path = path
    else:
        root_path = path
        openai_path = f"{path}/v1"
    root_url = urlunsplit((parsed.scheme, parsed.netloc, root_path, "", "")).rstrip("/")
    openai_url = urlunsplit(
        (parsed.scheme, parsed.netloc, openai_path, "", "")
    ).rstrip("/")
    return root_url, openai_url


def prompt_text(repetitions: int, request_id: str) -> str:
    return (
        f"Unique synthetic load-test request {request_id}. "
        "Analyze the following independent benchmark payload.\n"
        + " benchmark" * repetitions
        + "\nContinue the response until the requested output limit."
    )


async def tokenize_count(
    client: Any,
    root_url: str,
    token: str,
    prompt: str,
) -> int:
    response = await client.post(
        f"{root_url}/tokenize",
        headers={"Authorization": f"Bearer {token}"},
        json={
            "model": MODEL_ID,
            "messages": [{"role": "user", "content": prompt}],
        },
    )
    response.raise_for_status()
    payload = response.json()
    count = payload.get("count")
    if not isinstance(count, int):
        tokens = payload.get("tokens")
        if isinstance(tokens, list):
            count = len(tokens)
    if not isinstance(count, int):
        raise RuntimeError("/tokenize response did not contain a token count")
    return count


async def calibrate_repetitions(
    client: Any,
    root_url: str,
    token: str,
    target_tokens: int,
) -> int:
    low = 0
    high = target_tokens * 2
    marker = f"calibration-{target_tokens:06d}"
    while low < high:
        middle = (low + high) // 2
        count = await tokenize_count(
            client, root_url, token, prompt_text(middle, marker)
        )
        if count < target_tokens:
            low = middle + 1
        else:
            high = middle
    candidates = [max(0, low - 1), low]
    measured = [
        await tokenize_count(client, root_url, token, prompt_text(value, marker))
        for value in candidates
    ]
    return min(zip(candidates, measured), key=lambda item: abs(item[1] - target_tokens))[0]


async def run_request(
    openai_client: Any,
    semaphore: asyncio.Semaphore,
    bucket: Bucket,
    prompt: str,
    request_id: str,
    timeout_s: float,
) -> RequestResult:
    async with semaphore:
        started = time.perf_counter()
        first_token_at: float | None = None
        usage: Any = None
        try:
            async with asyncio.timeout(timeout_s):
                stream = await openai_client.chat.completions.create(
                    model=MODEL_ID,
                    messages=[{"role": "user", "content": prompt}],
                    max_tokens=bucket.output_tokens,
                    temperature=0.0,
                    stream=True,
                    stream_options={"include_usage": True},
                    extra_body={"ignore_eos": True},
                )
                async for chunk in stream:
                    if chunk.usage is not None:
                        usage = chunk.usage
                    if first_token_at is None and chunk.choices:
                        delta = chunk.choices[0].delta
                        if (
                            getattr(delta, "content", None)
                            or getattr(delta, "reasoning_content", None)
                            or getattr(delta, "tool_calls", None)
                        ):
                            first_token_at = time.perf_counter()
            finished = time.perf_counter()
            if usage is None:
                raise RuntimeError("stream ended without token usage")
            input_tokens = int(usage.prompt_tokens)
            output_tokens = int(usage.completion_tokens)
            if first_token_at is None:
                raise RuntimeError("stream ended without a generated token")
            ttft_s = first_token_at - started
            e2el_s = finished - started
            tpot_s = (
                (e2el_s - ttft_s) / (output_tokens - 1)
                if output_tokens > 1
                else 0.0
            )
            error = None
            if output_tokens != bucket.output_tokens:
                error = (
                    f"expected {bucket.output_tokens} output tokens, "
                    f"received {output_tokens}"
                )
            return RequestResult(
                request_id=request_id,
                bucket=bucket.name,
                target_input_tokens=bucket.input_tokens,
                target_output_tokens=bucket.output_tokens,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                ttft_ms=ttft_s * 1_000,
                tpot_ms=tpot_s * 1_000,
                e2el_ms=e2el_s * 1_000,
                error=error,
            )
        except Exception as exc:
            elapsed_ms = (time.perf_counter() - started) * 1_000
            return RequestResult(
                request_id=request_id,
                bucket=bucket.name,
                target_input_tokens=bucket.input_tokens,
                target_output_tokens=bucket.output_tokens,
                input_tokens=0,
                output_tokens=0,
                ttft_ms=None,
                tpot_ms=None,
                e2el_ms=elapsed_ms,
                error=f"{type(exc).__name__}: {exc}",
            )


def metric_summary(
    concurrency: int,
    elapsed_s: float,
    results: list[RequestResult],
) -> dict[str, Any]:
    successful = [result for result in results if result.error is None]
    failed = len(results) - len(successful)
    total_output = sum(result.output_tokens for result in successful)
    aggregate_output_tps = total_output / elapsed_s if elapsed_s else 0.0

    def values(name: str) -> list[float]:
        return [
            float(value)
            for result in successful
            if (value := getattr(result, name)) is not None
        ]

    def mean_or_none(name: str) -> float | None:
        samples = values(name)
        return statistics.fmean(samples) if samples else None

    summary: dict[str, Any] = {
        "concurrency": concurrency,
        "requests": len(results),
        "completed": len(successful),
        "failed": failed,
        "duration_s": elapsed_s,
        "mean_input_tokens": mean_or_none("input_tokens"),
        "mean_output_tokens": mean_or_none("output_tokens"),
        "aggregate_output_tokens_per_s": aggregate_output_tps,
        "output_tokens_per_s_per_stream": aggregate_output_tps / concurrency,
        "requests_per_s": len(successful) / elapsed_s if elapsed_s else 0.0,
    }
    for name in ("ttft_ms", "tpot_ms", "e2el_ms"):
        samples = values(name)
        summary[f"median_{name}"] = percentile(samples, 0.50)
        summary[f"p90_{name}"] = percentile(samples, 0.90)
        summary[f"p99_{name}"] = percentile(samples, 0.99)
    return summary


def write_outputs(
    result_dir: Path,
    summaries: list[dict[str, Any]],
    detailed: dict[int, list[RequestResult]],
) -> None:
    result_dir.mkdir(parents=True, exist_ok=True)
    payload = {
        "model": MODEL_ID,
        "model_revision": MODEL_REVISION,
        "seed": SEED,
        "load_model": "closed-loop",
        "waves_per_concurrency": 2,
        "weighted_target_input_tokens": weighted_mean("input_tokens"),
        "weighted_target_output_tokens": weighted_mean("output_tokens"),
        "buckets": [asdict(bucket) for bucket in BUCKETS],
        "summaries": summaries,
    }
    (result_dir / "summary.json").write_text(
        json.dumps(payload, indent=2, allow_nan=False) + "\n", encoding="utf-8"
    )

    if summaries:
        with (result_dir / "summary.csv").open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=list(summaries[0]))
            writer.writeheader()
            writer.writerows(summaries)

    for concurrency, results in detailed.items():
        detail_payload = {
            "model": MODEL_ID,
            "model_revision": MODEL_REVISION,
            "concurrency": concurrency,
            "requests": [asdict(result) for result in results],
        }
        (result_dir / f"concurrency-{concurrency}.json").write_text(
            json.dumps(detail_payload, indent=2, allow_nan=False) + "\n",
            encoding="utf-8",
        )


async def warm_up(openai_client: Any, timeout_s: float) -> None:
    bucket = Bucket("warmup", 8, 128, 128)
    semaphore = asyncio.Semaphore(4)
    results = await asyncio.gather(
        *[
            run_request(
                openai_client,
                semaphore,
                bucket,
                prompt_text(128, f"warmup-{index:02d}"),
                f"warmup-{index:02d}",
                timeout_s,
            )
            for index in range(4)
        ]
    )
    errors = [result.error for result in results if result.error]
    if errors:
        raise RuntimeError(f"warmup failed: {errors[0]}")


async def benchmark(args: argparse.Namespace) -> int:
    try:
        import httpx
        from openai import AsyncOpenAI
    except ImportError as exc:
        print(f"missing benchmark dependency: {exc}", file=sys.stderr)
        return 2

    token = os.environ.get("OPENAI_API_KEY") or os.environ.get("PFT_ENDPOINT_TOKEN")
    if not token:
        print(
            "set OPENAI_API_KEY or PFT_ENDPOINT_TOKEN in the environment",
            file=sys.stderr,
        )
        return 2

    root_url, openai_url = api_urls(args.base_url)
    result_dir = Path(args.result_dir)
    summaries: list[dict[str, Any]] = []
    detailed: dict[int, list[RequestResult]] = {}

    async with httpx.AsyncClient(timeout=args.timeout_s) as raw_client:
        repetitions: dict[int, int] = {}
        for bucket in BUCKETS:
            print(f"calibrating {bucket.name} prompt to {bucket.input_tokens} tokens")
            repetitions[bucket.input_tokens] = await calibrate_repetitions(
                raw_client,
                root_url,
                token,
                bucket.input_tokens,
            )

    openai_client = AsyncOpenAI(
        api_key=token,
        base_url=openai_url,
        timeout=args.timeout_s,
        max_retries=0,
    )
    try:
        print("running four-request warmup")
        await warm_up(openai_client, args.timeout_s)

        for concurrency in args.concurrencies:
            workload = workload_for(concurrency)
            semaphore = asyncio.Semaphore(concurrency)
            tasks = []
            for index, bucket in enumerate(workload):
                request_id = f"c{concurrency:03d}-r{index:04d}"
                prompt = prompt_text(repetitions[bucket.input_tokens], request_id)
                tasks.append(
                    run_request(
                        openai_client,
                        semaphore,
                        bucket,
                        prompt,
                        request_id,
                        args.timeout_s,
                    )
                )

            print(
                f"running concurrency={concurrency}: "
                f"{len(tasks)} requests, two closed-loop waves"
            )
            started = time.perf_counter()
            results = await asyncio.gather(*tasks)
            elapsed_s = time.perf_counter() - started
            detailed[concurrency] = list(results)
            summary = metric_summary(concurrency, elapsed_s, list(results))
            summaries.append(summary)
            write_outputs(result_dir, summaries, detailed)
            print(
                f"concurrency={concurrency} completed={summary['completed']} "
                f"failed={summary['failed']} "
                f"aggregate_output_tps={summary['aggregate_output_tokens_per_s']:.2f} "
                f"per_stream_tps={summary['output_tokens_per_s_per_stream']:.2f}"
            )
            if summary["failed"]:
                print("stopping after a failed concurrency level", file=sys.stderr)
                return 1
    finally:
        await openai_client.close()

    print(f"wrote benchmark evidence to {result_dir}")
    return 0


def parse_concurrencies(value: str) -> tuple[int, ...]:
    try:
        return tuple(int(item.strip()) for item in value.split(",") if item.strip())
    except ValueError as exc:
        raise argparse.ArgumentTypeError(str(exc)) from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base-url",
        help="authenticated endpoint base URL, with or without /v1",
    )
    parser.add_argument(
        "--result-dir",
        default="qa/gpu-rentals/benchmarks/glm53-b300/results",
    )
    parser.add_argument(
        "--concurrencies",
        type=parse_concurrencies,
        default=DEFAULT_CONCURRENCIES,
        help="comma-separated closed-loop concurrency levels",
    )
    parser.add_argument(
        "--timeout-s",
        type=float,
        default=7_200,
        help="per-request and tokenization timeout",
    )
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="validate and print the workload without contacting an endpoint",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    validate_contract(args.concurrencies)
    print(
        f"weighted target: input={weighted_mean('input_tokens'):.0f}, "
        f"output={weighted_mean('output_tokens'):.0f} tokens"
    )
    for concurrency in args.concurrencies:
        counts = {
            bucket.name: workload_for(concurrency).count(bucket) for bucket in BUCKETS
        }
        print(
            f"concurrency={concurrency} requests={2 * concurrency} "
            + " ".join(f"{name}={count}" for name, count in counts.items())
        )
    if args.validate_only:
        return 0
    if not args.base_url:
        print("--base-url is required unless --validate-only is used", file=sys.stderr)
        return 2
    return asyncio.run(benchmark(args))


if __name__ == "__main__":
    raise SystemExit(main())
