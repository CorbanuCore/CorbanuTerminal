# Records which bearer each request carried (as canary id / sha12), answers 401.
import hashlib, http.server, json, sys
LOG = sys.argv[2]
KNOWN = json.loads(open(sys.argv[3]).read())  # {canary_value: id}
class H(http.server.BaseHTTPRequestHandler):
    def _h(self):
        n = int(self.headers.get('Content-Length') or 0)
        body = self.rfile.read(n) if n else b''
        auth = self.headers.get('Authorization') or ''
        tok = auth[7:] if auth.lower().startswith('bearer ') else auth
        rec = {'method': self.command, 'path': self.path,
               'bearer_id': KNOWN.get(tok, 'UNKNOWN' if tok else 'NONE'),
               'bearer_sha12': hashlib.sha256(tok.encode()).hexdigest()[:12] if tok else None,
               'body_has_canary': [v for k, v in KNOWN.items() if k.encode() in body]}
        with open(LOG, 'a') as f: f.write(json.dumps(rec) + '\n')
        msg = json.dumps({'error': {'message': 'mock: invalid api key', 'type': 'invalid_request_error', 'code': 'invalid_api_key'}}).encode()
        self.send_response(401); self.send_header('Content-Type', 'application/json'); self.send_header('Content-Length', str(len(msg))); self.end_headers(); self.wfile.write(msg)
    do_POST = do_GET = _h
    def log_message(self, *a): pass
http.server.ThreadingHTTPServer(('127.0.0.1', int(sys.argv[1])), H).serve_forever()
