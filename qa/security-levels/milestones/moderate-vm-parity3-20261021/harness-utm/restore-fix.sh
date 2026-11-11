echo PF83BEGIN
ps -axo pid,uid,comm | awk '$2>=610 && $2<=613'
for p in $(ps -axo pid=,uid= | awk '$2>=610 && $2<=613 {print $1}'); do kill -9 $p 2>/dev/null; done
sleep 2
echo "[restore-fix] processes for removed uids: $(ps -axo uid= | awk '$1>=610 && $1<=613' | wc -l)"
mkdir -p "/Volumes/My Shared Files"
mount_virtiofs com.apple.virtio-fs.automount "/Volumes/My Shared Files" 2>&1 | sed 's/^/[restore-fix] mount_virtiofs: /'
echo "[restore-fix] virtiofs mounted: $(mount | grep -ci virtiofs)"; mount | grep -i virtiofs
