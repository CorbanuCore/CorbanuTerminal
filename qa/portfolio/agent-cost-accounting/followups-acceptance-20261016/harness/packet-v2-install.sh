echo PF83BEGIN
# Packet v2 (navigation note only): replace navigation.md and the prompts of groups not yet started (E, F) and add
# replay prompts C2, D2. Running and finished groups are untouched; design-cases.md, amendments.md, answers.md unchanged.
set -e
S=/Users/agent/agms/stage/packet-v2
cd /opt/agms/packet
chmod u+w . prompts navigation.md prompts/E.md prompts/F.md
for f in navigation.md prompts/E.md prompts/F.md prompts/C2.md prompts/D2.md; do cp "$S/$f" "$f.new"; mv -f "$f.new" "$f"; done
chown -R root:wheel /opt/agms/packet; find /opt/agms/packet -type f -exec chmod 444 {} +; find /opt/agms/packet -type d -exec chmod 555 {} +
shasum -a 256 navigation.md prompts/*.md design-cases.md amendments.md answers.md
