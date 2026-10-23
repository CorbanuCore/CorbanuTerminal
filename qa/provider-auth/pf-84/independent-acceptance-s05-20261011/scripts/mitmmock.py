#!/usr/bin/env python3
"""mitmmock.py <port> <log> <certdir>: an HTTPS_PROXY that terminates CONNECT tunnels with
a throwaway CA (certdir/ca.pem, leaf certdir/leaf.pem+leaf.key) and answers every request
itself with the usageproxy mock (429 for MOCK_429 bearers, "pong" otherwise). Nothing is
forwarded upstream in mock mode, so only synthetic canary keys are used against it; in proxy mode
(MITM_MODE=proxy, UPSTREAM=https://api.z.ai) it relays to the real provider and logs its usage.
Hosts outside MITM_HOSTS (comma list) are refused, so no other traffic is intercepted."""
import os, socket, ssl, sys, threading
# MITM_MODE=proxy forwards to UPSTREAM (e.g. https://api.z.ai) instead of answering itself.
sys.argv = [sys.argv[0], os.environ.get("MITM_MODE", "mock"), sys.argv[1], sys.argv[2], sys.argv[3]]
import usageproxy  # noqa: E402  (reads MODE/PORT/LOG from argv)

PORT, CERTDIR = int(sys.argv[2]), sys.argv[4]
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(os.path.join(CERTDIR, "leaf.pem"), os.path.join(CERTDIR, "leaf.key"))
ctx.set_alpn_protocols(["http/1.1"])


def handle(conn):
    f = conn.makefile("rb")
    line = f.readline()
    while f.readline() not in (b"\r\n", b"\n", b""):
        pass
    if not line.startswith(b"CONNECT"):
        conn.close(); return
    host = line.split()[1].decode()
    if os.environ.get("MITM_HOSTS") and host.split(":")[0] not in os.environ["MITM_HOSTS"].split(","):
        usageproxy.log({"connect_refused": host}); conn.close(); return
    conn.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
    try:
        tls = ctx.wrap_socket(conn, server_side=True)
    except ssl.SSLError as e:
        usageproxy.log({"tls_error": str(e), "connect": host}); return
    usageproxy.H(tls, ("127.0.0.1", 0), None)


srv = socket.socket(); srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
srv.bind(("127.0.0.1", PORT)); srv.listen(64)
while True:
    c, _ = srv.accept()
    threading.Thread(target=handle, args=(c,), daemon=True).start()
