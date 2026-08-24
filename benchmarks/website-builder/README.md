# Website builder benchmark

This directory is the canonical Corbanu Terminal website benchmark harness. It
compares Corbanu and Claude Code using the same Anthropic model, a frozen
baseline and prompt, isolated homes and credentials, deterministic static and
browser checks, six matched screenshots, and balanced blind visual judging.

Historical pre-Corbanu experiment packets remain evidence only. New campaigns
must use this repository-owned harness and write results to the ignored `runs/`
directory or to the applicable release evidence directory.

## Install the verifier and judge dependencies

```bash
python3 -m venv benchmarks/website-builder/.venv
benchmarks/website-builder/.venv/bin/pip install \
  -r benchmarks/website-builder/requirements.txt
benchmarks/website-builder/.venv/bin/playwright install chromium
```

Set `CORBANU_BENCH_BROWSER` only when the verifier should use a specific Chrome
or Chromium executable. Otherwise Playwright uses its installed Chromium.

## Run contestants

Use separate Anthropic keys for the contestant lanes. The shared OpenAI key is
available only to the contestants for the task's three required image calls.
No key value is written to the manifest.

```bash
benchmarks/website-builder/.venv/bin/python \
  benchmarks/website-builder/run_pair.py \
  --run-root benchmarks/website-builder/runs/campaign-YYYYMMDD \
  --corbanu-bin /path/to/corbanu \
  --claude-bin /path/to/claude \
  --corbanu-anthropic-key-file /path/to/corbanu-anthropic-key \
  --claude-anthropic-key-file /path/to/claude-anthropic-key \
  --openai-key-file /path/to/openai-key \
  --confirm-paid-run
```

The two lanes run concurrently. Waves are serial inside each lane. Every wave
gets a clean baseline, an isolated tool home, route evidence, source-tree
integrity evidence, an independent verifier run, and the six capture files
required by the judge. A benchmark-source mutation skips verification and fails
the lane.

## Judge a matched wave

```bash
benchmarks/website-builder/.venv/bin/python \
  benchmarks/website-builder/judge_pair.py \
  --run-root benchmarks/website-builder/runs/campaign-YYYYMMDD \
  --openai-key-file /path/to/openai-key \
  --wave 1 \
  --confirm-paid-run
```

The judge sees only opaque A/B screenshot packets. It runs normal and swapped
orders. Order disagreement produces an inconclusive visual result rather than a
forced winner. Functional validity, visual quality, wall time, and cost remain
separate measurements.

## Scan artifacts for exact keys

```bash
python3 benchmarks/scan_exact_keys.py \
  --path benchmarks/website-builder \
  --path benchmarks/website-builder/runs/campaign-YYYYMMDD \
  --key-file /path/to/corbanu-anthropic-key \
  --key-file /path/to/claude-anthropic-key \
  --key-file /path/to/openai-key
```

This harness measures website-building capability. It does not replace the
competitive release benchmark, true-TUI qualification, or human release
acceptance.
