with_iso="$1"
boot_flag="-boot c"
iso_flag=""
network="-net nic -net user,hostfwd=tcp::10022-:2222"

if [ ! -f ${diskImage} ]; then
  echo "Disk image '${diskImage}' not found. Run create-qemu-disk first."
  return 1
fi

if [ "$with_iso" = "--iso" ]; then
  boot_flag="-boot once=d"
  iso_flag="-cdrom ${nixosIso}"
  network="-net nic -net user,hostfwd=tcp::10022-:22"
fi

qemu-system-x86_64 \
  -enable-kvm \
  -m 4096 \
  -cpu host \
  $boot_flag \
  $iso_flag \
  $network \
  -drive file=vm-disk.qcow2,format=qcow2 \
  -vga virtio \
  -usb -device usb-tablet
