#!/usr/bin/python3
"""Loopback DNS responder for a sealed guest: answers every query REFUSED at once.

The sealed guest resolves only what /etc/hosts pins; without a responder, every other lookup would
wait for timeouts instead of failing fast. Runs as root under launchd (seal-guest.sh).
"""

import socket

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(("127.0.0.1", 53))


def question_end(query):
    pos = 12
    while pos < len(query) and query[pos]:
        pos += 1 + query[pos]
    return pos + 5  # zero label, QTYPE, QCLASS


while True:
    query, peer = sock.recvfrom(4096)
    end = question_end(query)
    if len(query) < 12 or end > len(query):
        continue
    flags = (
        0x8000 | (query[2] & 0x79) << 8 | 0x0005
    )  # QR; opcode and RD copied; RCODE=REFUSED
    sock.sendto(
        query[:2] + flags.to_bytes(2, "big") + b"\0\1\0\0\0\0\0\0" + query[12:end], peer
    )
