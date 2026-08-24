# APIGate Debugging Task

The in-process APIGate service has regressed. It should expose a small HTTP-style API with:

- path routing with static and dynamic route support
- token issue, refresh, expiry, and authentication
- permission checks with cached grants and revokes
- request-scoped context
- per-user write rate limits
- JSON response serialization and structured error mapping

Start with the visible failing test:

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

Fix the product behavior, not just the traceback location. Do not remove or weaken tests. Keep the implementation standard-library only.
