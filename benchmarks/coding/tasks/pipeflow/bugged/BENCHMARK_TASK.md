CI is red on the integration suite.

Users report that pipelines sometimes hang or fail after resume, especially when
environment overrides disable optional source stages.

Fix the system. Do not modify tests. Do not bypass the verifier. Keep the
behavior regression-safe: existing config loading, DAG ordering, retry,
checkpoint/resume, metrics, and CLI behavior must continue to work.

Start with:

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

The full verifier will run additional private tests.
