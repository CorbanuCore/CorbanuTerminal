# Canary proxy: records which synthetic bearer (per account) each request carried.
# Bearers listed in FORWARD are forwarded to the real upstream with the real key (taken from
# the env var REAL_KEY, never logged); every other bearer gets a 401. Logs canary ids only.
# usage: REAL_KEY=... python3 canary_proxy.py <port> <log.jsonl> <canaries.json> <upstream> <forward ids,comma>
import hashlib, http.server, json, os, sys, urllib.request, urllib.error

PORT, LOG, CAN, UP, FWD = int(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4].rstrip('/'), set(sys.argv[5].split(','))
KNOWN = json.load(open(CAN))  # {canary_value: id}
REAL = os.environ.get('REAL_KEY', '')


def first_user_text(body):
    try:
        msgs = json.loads(body).get('messages') or []
    except Exception:
        return None
    for m in msgs:
        if m.get('role') == 'user':
            c = m.get('content')
            if isinstance(c, list):
                c = ' '.join(p.get('text', '') for p in c if isinstance(p, dict))
            c = (c or '').replace('\n', ' ')
            if not c.startswith('<'):
                return c[:70]
    return None


def tool_view(body):
    """What the model sees: does spawn_agent offer `account`; spawn results mentioning an account."""
    try:
        d = json.loads(body)
    except Exception:
        return None, [], []
    offer, names = None, []
    for t in d.get('tools') or []:
        f = t.get('function') or t
        names.append(f.get('name') or t.get('type'))
        if 'spawn' in (f.get('name') or ''):
            props = ((f.get('parameters') or {}).get('properties') or {})
            offer = 'account' in props
    results = []
    for m in d.get('messages') or []:
        if m.get('role') == 'tool':
            c = m.get('content')
            c = c if isinstance(c, str) else json.dumps(c)
            if 'account' in c:
                results.append(c.replace('\n', ' ')[:200])
    return offer, results, names


class H(http.server.BaseHTTPRequestHandler):
    def _h(self):
        n = int(self.headers.get('Content-Length') or 0)
        body = self.rfile.read(n) if n else b''
        auth = self.headers.get('Authorization') or ''
        tok = auth[7:] if auth.lower().startswith('bearer ') else auth
        bid = KNOWN.get(tok, 'UNKNOWN' if tok else 'NONE')
        rec = {'path': self.path, 'bearer': bid,
               'bearer_sha12': hashlib.sha256(tok.encode()).hexdigest()[:12] if tok else None,
               'first_user': first_user_text(body),
               'canary_values_in_body': [v for k, v in KNOWN.items() if k.encode() in body],
               'real_key_in_body': bool(REAL) and REAL.encode() in body}
        rec['spawn_offers_account'], rec['tool_results_with_account'], rec['tools'] = tool_view(body)
        status, ctype, out = 401, 'application/json', json.dumps(
            {'error': {'message': 'canary-proxy: invalid api key', 'code': 'invalid_api_key'}}).encode()
        if bid in FWD and REAL:
            req = urllib.request.Request(UP + self.path.replace('/proxy', '', 1), data=body if self.command == 'POST' else None,
                                         method=self.command, headers={'Authorization': 'Bearer ' + REAL,
                                                                       'Content-Type': 'application/json'})
            try:
                with urllib.request.urlopen(req, timeout=300) as r:
                    status, ctype, out = r.status, r.headers.get('Content-Type', 'application/json'), r.read()
            except urllib.error.HTTPError as e:
                status, ctype, out = e.code, 'application/json', e.read()
        rec['status'] = status
        with open(LOG, 'a') as f:
            f.write(json.dumps(rec) + '\n')
        self.send_response(status)
        self.send_header('Content-Type', ctype)
        self.send_header('Content-Length', str(len(out)))
        self.end_headers()
        self.wfile.write(out)

    do_POST = do_GET = _h

    def log_message(self, *a):
        pass


http.server.ThreadingHTTPServer(('127.0.0.1', PORT), H).serve_forever()
