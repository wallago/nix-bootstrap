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
        # nixosIso =
        #   "https://github.com/nix-community/nixos-images/releases/download/nixos-unstable/nixos-installer-x86_64-linux.iso";
        # diskImage = "vm-disk.qcow2";
        commonBuildInputs = with pkgs; [ openssl ];
        commonNativeBuildInputs = with pkgs; [ pkg-config ];
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "nix-bootstrap";
          src = ./..;
          cargoLock = { lockFile = ../Cargo.lock; };
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        };
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = with pkgs;
              [ sops rust-analyzer ] ++ [ rust ] ++ commonBuildInputs;
            shellHook = ''
              echo "
              🐚 Rust dev shell ready!
              Run: cargo build / cargo test / etc."
            '';
          };
          # qemu = pkgs.mkShell {
          #   buildInputs = with pkgs;
          #     [ pkg-config openssl sops rust-analyzer qemu ] ++ [ rust ];
          #   shellHook = ''
          #     export PATH=$PATH:${toString ./shell}
          #     export nixosIso=${nixosIso}
          #     export diskImage=${diskImage}
          #     echo "
          #     Welcome to your QEMU NixOS dev shell! 
          #     Available commands: 
          #     - create-qemu-disk.sh 
          #     - run-qemu.sh (--iso optional)
          #     - ssh-vm.sh"
          #   '';
          # };
        };
      });
}
