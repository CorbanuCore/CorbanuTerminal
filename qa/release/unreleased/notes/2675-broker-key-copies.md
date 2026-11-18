## Security

- With the model credential broker on (Moderate and Aggressive by default), a provider key it holds is now also removed from Corbanu's environment when a copy of it sits in another variable (for example a prefixed header value), so agent commands no longer inherit that copy. If the broker cannot check the environment, it is not used and model requests are refused.
