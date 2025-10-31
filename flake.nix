{
  description = "gai";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    sussg.url = "github:nuttycream/sussg";
  };

  outputs = {
    sussg,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
        with pkgs; {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            name = "gai";
            src = ./.;

            buildInputs = [
              openssl
            ];

            nativeBuildInputs = [
              pkg-config
            ];

            cargoHash = "sha256-+Db1P74Fupho2bgSn2qWyL2zd1+c2XQZVOJC+bZOmLY=";
          };

          devShells.default = mkShell {
            name = "gai";
            packages = with pkgs; [
              just
              rust-bin.stable.latest.default
              sussg.packages.${system}.default
            ];

            buildInputs = [
              openssl
              pkg-config
            ];
          };
        }
    );
}
