#!/usr/bin/env python3
"""CONNECT proxy for PF-60-S09 stand-ins: tunnels CONNECT <allowed host>:443 to a local TLS
server; refuses every other host. Logs hosts only.

usage: proxy.py <listen port> <target port> <host> [<host>...]
"""
import asyncio
import json
import sys
import time

LISTEN, TARGET, HOSTS = int(sys.argv[1]), int(sys.argv[2]), set(sys.argv[3:])


def log(entry):
    print(json.dumps(dict(t=round(time.time(), 3), **entry)), flush=True)


async def pipe(reader, writer):
    try:
        while data := await reader.read(65536):
            writer.write(data)
            await writer.drain()
    except (ConnectionError, asyncio.CancelledError):
        pass
    finally:
        try:
            writer.close()
        except Exception:
            pass


async def handle(reader, writer):
    head = await reader.readuntil(b"\r\n\r\n")
    method, target, _ = head.decode("latin1").split("\r\n")[0].split(" ", 2)
    host, _, port = target.rpartition(":")
    if method != "CONNECT" or host not in HOSTS or port != "443":
        log({"refused": f"{method} {target}"})
        writer.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
        await writer.drain()
        writer.close()
        return
    log({"tunnel": host})
    up_reader, up_writer = await asyncio.open_connection("127.0.0.1", TARGET)
    writer.write(b"HTTP/1.1 200 Connection Established\r\n\r\n")
    await writer.drain()
    await asyncio.gather(pipe(reader, up_writer), pipe(up_reader, writer))


async def main():
    server = await asyncio.start_server(handle, "127.0.0.1", LISTEN)
    async with server:
        await server.serve_forever()


asyncio.run(main())
