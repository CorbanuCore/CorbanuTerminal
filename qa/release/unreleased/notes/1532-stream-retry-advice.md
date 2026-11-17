## Reliability

Rate-limited requests now carry the provider's retry deadline into the shared request throttle. Websocket retries honor that advice instead of waiting for an unrelated default cooldown.
