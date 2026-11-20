## Security

Command rules can now opt into case-insensitive token comparison with `canonical_matching=True` (an upstream rule option). Rules that don't set it keep 0.1.49's exact-spelling requirement for `allow` rules on Windows (D696, #2553), and program paths listed for an `allow` rule still require their recorded spelling even when it is set.
