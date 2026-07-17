{
  description = "Phylax";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rust

            cargo
            rust-analyzer
            rustfmt
            clippy

            cargo-edit
            cargo-watch
            cargo-expand
            cargo-nextest
            cargo-audit
            cargo-deny
            cargo-outdated
            cargo-llvm-cov
            cargo-udeps
            cargo-dist

            openssl
            pkg-config
            sqlite

            just
            jq
            mdbook
            graphviz

            clang
            lld
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            mold
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
          ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      });
}