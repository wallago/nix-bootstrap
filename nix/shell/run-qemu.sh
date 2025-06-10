FILE="$HOME/Downloads/nixos-installer-x86_64-linux.iso"

if [ ! -f "$FILE" ]; then
  echo "Downloading NixOS ISO..."
  curl -L "${url}" -o "$FILE"
else
  echo "ISO already downloaded: $FILE"
fi

if [ ! -f ${diskImage} ]; then
  echo "Disk image '${diskImage}' not found. Run create-qemu-disk first."
  return 1
fi

qemu-system-x86_64 \
  -enable-kvm \
  -m 4096 \
  -cpu host \
  -boot once=d \
  -cdrom ${nixosIso} \
  -nic user,hostfwd=tcp::10022-:22,hostfwd=tcp::12222-:2222 \
  -drive file=vm-disk.qcow2,format=qcow2 \
  -vga std \
  -usb -device usb-tablet
