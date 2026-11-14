## Reliability

Remote MCP process handles remain eligible for cleanup when a startup timeout cancels an outstanding termination request. Successful termination still suppresses duplicate cleanup. Thanks to @bradsbots for the fix and cancellation regression test (#2132).
