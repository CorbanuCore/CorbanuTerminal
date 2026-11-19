## Security

- With the model credential broker on (Moderate and Aggressive by default), any other environment variable that contains a copy of a provider key the broker holds (for example a prefixed header value) is now removed from Corbanu's environment too, so agent commands no longer inherit it. Keys shorter than 8 characters (local-server placeholders) are not looked for. If the broker cannot check the environment, it is not used and model requests are refused; such copies are then left in place.
