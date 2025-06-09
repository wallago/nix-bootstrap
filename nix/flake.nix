{
  description = "Nix dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.nightly.latest.default;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [ pkg-config openssl sops rust-analyzer qemu ] ++ [ rust ];
          shellHook = ''
            export PATH=$PATH:${toString ./shell}
            echo "Welcome to your QEMU NixOS dev shell!"
            echo "Available commands:"
            echo "- create-qemu-disk.sh"
            echo "- run-qemu.sh (--iso optional)"
            echo "- ssh-vm.sh"
          '';
        };
      });
}
