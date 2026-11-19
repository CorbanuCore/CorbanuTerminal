## Security

- With the network proxy's credential broker on (off by default), any environment variable that contains a copy of a key the broker holds (for example `Bearer <key>` in another variable) is now removed from the environment of proxied commands, not only the provider variable itself. Keys shorter than 8 characters are not looked for. If the broker cannot check an environment, it gives no request a key from then on.
