{
  description = "sussg";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
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
            buildInputs = [];
            nativeBuildInputs = [];
            cargoHash = lib.fakeHash;
          };

          devShells.default = mkShell {
            name = "gai";
            packages = with pkgs; [
              just
              bacon
              rust-bin.stable.latest.default
            ];

            buildInputs = [
              openssl
              pkg-config
            ];
          };
        }
    );
}
