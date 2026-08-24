CI is red in the text rendering pipeline.

A status line containing an escaped pipe renders through the table path instead of a
paragraph path, and the issue surfaces as a rendering mismatch. Fix the pipeline from
lexer through renderer. Do not modify tests or bypass the verifier.

Start with:

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

The full verifier will run additional private tests.
