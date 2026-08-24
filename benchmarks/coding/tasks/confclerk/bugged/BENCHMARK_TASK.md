CI is red in the rendered configuration integration path.

A preview feature is disabled with an environment override, but the application still
attempts to render preview-only template data and raises from the templating layer.

Fix the configuration system. Do not modify tests. Do not bypass the verifier.
Start with:

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

The full verifier will run additional private tests.
