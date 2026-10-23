#!/usr/bin/env bash
# PF-60-S09: throwaway TLS material for the stand-ins, written to <dir> (never commit it).
#   ca.pem/ca.key        a throwaway CA; point the candidate at it with CODEX_CA_CERTIFICATE=<dir>/ca.pem
#   chatgpt.pem/.key     a server certificate for auth.openai.com, chatgpt.com and ab.chatgpt.com
#   offer.sdp            a WebRTC SDP offer (audio + data channel) for the realtime driver's --transport webrtc
# usage: make_certs.sh <dir>
set -eu
dir=$1; mkdir -p "$dir"; cd "$dir"
openssl req -x509 -newkey rsa:2048 -nodes -keyout ca.key -out ca.pem -days 3 \
  -subj "/CN=PF-60-S09 throwaway test CA" \
  -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
openssl req -newkey rsa:2048 -nodes -keyout chatgpt.key -out chatgpt.csr -subj "/CN=chatgpt.com" 2>/dev/null
printf "subjectAltName=DNS:chatgpt.com,DNS:auth.openai.com,DNS:ab.chatgpt.com\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n" > chatgpt.ext
openssl x509 -req -in chatgpt.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out chatgpt.pem -days 3 -extfile chatgpt.ext 2>/dev/null
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout dtls.key -out dtls.pem -days 2 \
  -subj "/CN=pf60-s09-webrtc" 2>/dev/null
fp=$(openssl x509 -in dtls.pem -noout -fingerprint -sha256 | cut -d= -f2)
python3 - "$fp" <<'PY'
import sys
fp = sys.argv[1]
media = """a=ice-ufrag:pf60
a=ice-pwd:pf60s09webrtcofferpassword
a=ice-options:trickle
a=fingerprint:sha-256 {fp}
a=setup:actpass
""".format(fp=fp)
offer = ("v=0\no=- 4611731400430051336 2 IN IP4 127.0.0.1\ns=-\nt=0 0\na=group:BUNDLE 0 1\na=msid-semantic: WMS\n"
         "m=audio 9 UDP/TLS/RTP/SAVPF 111\nc=IN IP4 0.0.0.0\na=rtcp:9 IN IP4 0.0.0.0\n" + media +
         "a=mid:0\na=sendrecv\na=rtcp-mux\na=rtpmap:111 opus/48000/2\na=fmtp:111 minptime=10;useinbandfec=1\n"
         "m=application 9 UDP/DTLS/SCTP webrtc-datachannel\nc=IN IP4 0.0.0.0\n" + media +
         "a=mid:1\na=sctp-port:5000\na=max-message-size:262144\n")
open("offer.sdp", "w").write(offer.replace("\n", "\r\n"))
PY
echo "wrote $dir"
