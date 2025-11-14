
{
  description = "A simple Rust project with a stable toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    # This makes your flake compatible with different systems (Linux, macOS) [web:3][web:5]
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Apply the rust-overlay to your Nix packages [attached_file:1]
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        # This defines the development shell that `nix develop` will use
        devShells.default = with pkgs; mkShell {
          # These are the packages that will be available in your shell [attached_file:1]
          buildInputs = [
            # The full, stable Rust toolchain [attached_file:1]
            rust-bin.stable.latest.default
            rust-analyzer
            posting
            # Common dependencies for Rust projects
            openssl
            pkg-config
          ];
        };
      }
    );
}
