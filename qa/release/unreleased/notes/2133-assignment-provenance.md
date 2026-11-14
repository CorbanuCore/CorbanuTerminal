## Security

Post-taint shell checks no longer trust assignments that might be skipped or run
outside the parent shell. Conditional, pipeline, deferred-function and background
forms retain protected-action checks; possible literal protected-file paths remain
checked too. Nested background-group analysis uses bounded interval processing.
Thanks to @araupooo for the contribution and regression cases (#2133).
