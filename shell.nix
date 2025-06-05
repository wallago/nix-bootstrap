{ pkgs ? import <nixpkgs> { } }:
let
  nixosIso = pkgs.fetchurl {
    url =
      "https://channels.nixos.org/nixos-24.11/latest-nixos-minimal-x86_64-linux.iso";
    sha256 = "05jhl9qva3mgqrf7az3f7zbsp7ys2rqcr4ia0v01aiy43hk66r4i";
  };
  diskImage = "vm-disk.qcow2";
in pkgs.mkShell {
  buildInputs = with pkgs; [ sops pkg-config openssl qemu ];
  shellHook = ''
    create-qemu-disk() {
      qemu-img create -f qcow2 vm-disk.qcow2 20G
    }

    run-qemu() {
      local with_iso="$1"
      local boot_flag="-boot c"
      local iso_flag=""

      if [ ! -f ${diskImage} ]; then
        echo "Disk image '${diskImage}' not found. Run create-qemu-disk first."
        return 1
      fi

      if [ "$with_iso" = "--iso" ]; then
        boot_flag="-boot once=d"
        iso_flag="-cdrom ${nixosIso}"
      fi

      qemu-system-x86_64 \
        -enable-kvm \
        -m 4096 \
        -cpu host \
        $boot_flag \
        $iso_flag \
        -drive file=vm-disk.qcow2,format=qcow2 \
        -net nic -net user,hostfwd=tcp::10022-:22 \
        -vga virtio \
        -usb -device usb-tablet
    }

    ssh-vm() {
      local user="$1"

      if [[ -z "$user" ]]; then
        echo "Usage: ssh-vm <user>"
        return 1
      fi

      ssh -p 10022 $user@localhost
    }

    echo "Welcome to your QEMU NixOS dev shell!"
    echo "Available commands:"
    echo "- create-qemu-disk"
    echo "- run-qemu (--iso optional)"
    echo "- ssh-vm"
  '';
}
