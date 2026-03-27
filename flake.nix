{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      naersk,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # ── Toolchain ─────────────────────────────────────────────
        rust = pkgs.rust-bin.nightly.latest.default;

        naersk' = pkgs.callPackage naersk {
          cargo = rust;
          rustc = rust;
        };

        # ── Build helper ──────────────────────────────────────────
        buildApp =
          { release }:
          naersk'.buildPackage {
            name = "nix-bootstrap";
            src = ./.;
            inherit release;

            # Uncomment if your crate has C dependencies
            # buildInputs = with pkgs; [ openssl pkg-config ];

            meta = with pkgs.lib; {
              description = "Setup NixOS config with sops like a boss";
              homepage = "https://github.com/wallago/nix-bootstrap";
              license = [
                licenses.mit
                licenses.asl20
              ];
            };
          };
      in
      rec {
        # ── Packages ──────────────────────────────────────────────
        packages = rec {
          nix-bootstrap = buildApp { release = true; };
          nix-bootstrap-debug = buildApp { release = false; };
          default = nix-bootstrap;
        };

        # ── Checks (nix flake check) ─────────────────────────────
        checks.check = packages.nix-bootstrap-debug;

        # ── Dev Shell (nix develop) ──────────────────────────────
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            rust-analyzer
            just
          ];
        };
      }
    );
}
