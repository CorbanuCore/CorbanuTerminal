echo PF83BEGIN
# Harness fix 1: agms-product's corbanu route (empty provider array under bash 3.2 `set -u`), and the E2 replay prompt.
set -e
cp /Users/agent/agms/stage/h/agms-product /opt/agms/bin/agms-product.new; chown root:wheel /opt/agms/bin/agms-product.new; chmod 555 /opt/agms/bin/agms-product.new; mv -f /opt/agms/bin/agms-product.new /opt/agms/bin/agms-product
cd /opt/agms/packet; chmod u+w . prompts
cp /Users/agent/agms/stage/packet-v3/prompts/E2.md prompts/E2.md.new; mv -f prompts/E2.md.new prompts/E2.md
chown -R root:wheel /opt/agms/packet; find /opt/agms/packet -type f -exec chmod 444 {} +; find /opt/agms/packet -type d -exec chmod 555 {} +
shasum -a 256 /opt/agms/bin/agms-product prompts/E2.md
