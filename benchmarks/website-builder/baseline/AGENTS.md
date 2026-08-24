# Benchmark workspace

Build the site requested in `BENCHMARK_TASK.md`. Treat all product claims as
benchmark copy, not verified production claims.

Never print, echo, log, embed, or save API keys. The environment supplies
provider credentials only for runtime use. Do not inspect unrelated files
outside this workspace except for the verifier named in the task.

Do not edit `AGENTS.md` or `BENCHMARK_TASK.md`. Run the supplied verifier and
fix failures before finishing.
