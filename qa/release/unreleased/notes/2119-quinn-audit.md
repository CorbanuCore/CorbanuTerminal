## Security

- Update quinn-proto to 0.11.19 and quinn-udp to 0.5.16 to address RUSTSEC-2026-0523, RUSTSEC-2026-0524 and RUSTSEC-2026-0538 (QUIC stream limits, Retry token handling and a receive-loop spin after a truncated datagram) in the QUIC stack used by the TCP tunnel. No audit exception is added.
