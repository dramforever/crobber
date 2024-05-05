{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      eachSystem = nixpkgs.lib.genAttrs [ "x86_64-linux" ];
    in {
      devShell = eachSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          env = { mkShell, qemu, cargo-show-asm, rust, targetPlatform, rust-bin, pkgsCross }:
            mkShell {
              nativeBuildInputs = [
                qemu cargo-show-asm
                pkgsCross.riscv64.stdenv.cc

                (rust-bin.stable.latest.default.override {
                  extensions = [ "rust-src" ];
                  targets = [ (rust.toRustTarget targetPlatform) "riscv64gc-unknown-linux-gnu" ];
                })
              ];
            };
        in pkgs.callPackage env {});
    };
}
