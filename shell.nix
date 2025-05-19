{ pkgs ? import <nixpkgs> { } }:
let
  nixosIso = pkgs.fetchurl {
    url =
      "https://channels.nixos.org/nixos-24.11/latest-nixos-minimal-x86_64-linux.iso";
    sha256 = "05jhl9qva3mgqrf7az3f7zbsp7ys2rqcr4ia0v01aiy43hk66r4i";
  };
in pkgs.mkShell {
  buildInputs = with pkgs; [ pkg-config openssl qemu ];
  shellHook = ''
    echo "Create disk"
    echo "qemu-img create -f qcow2 vm-disk.qcow2 20G\n"
    echo "Running NixOS ISO from: ${nixosIso}"
    echo "qemu-system-x86_64 \\"
    echo "  -enable-kvm \\"
    echo "  -m 4096 \\"
    echo "  -cpu host \\"
    echo "  -boot d \\"
    echo "  -cdrom ${nixosIso} \\"
    echo "  -drive file=vm-disk.qcow2,format=qcow2 \\"
    echo "  -net nic -net user,hostfwd=tcp::10022-:22 \\"
    echo "  -vga virtio \\"
    echo "  -usb -device usb-tablet\n"
    echo "Running NixOS from:"
    echo "qemu-system-x86_64 \\"
    echo "  -enable-kvm \\"
    echo "  -m 4096 \\"
    echo "  -cpu host \\"
    echo "  -boot d \\"
    echo "  -drive file=vm-disk.qcow2,format=qcow2 \\"
    echo "  -net nic -net user,hostfwd=tcp::10022-:2222 \\"
    echo "  -vga virtio \\"
    echo "  -usb -device usb-tablet\n"
  '';
}

