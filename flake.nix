{
  description = "A ray tracer written in rust";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        rust = pkgs.rust-bin.stable.${"1.65.0"}.default.override {
          extensions = [ "rust-src" ];
        };
      in
      rec {
        packages.default = pkgs.callPackage ./racer-tracer.nix { inherit rust; };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ packages.default ];
          shellHook = ''
            cargo() {
              if [ "$1" = "run" ]; then
                LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [ pkgs.xorg.libX11 pkgs.xorg.libXcursor ] } command cargo "$@"
              else
                command cargo "$@"
              fi
            }

            cd ./racer-tracer
          '';
        };
      }
    );
}
