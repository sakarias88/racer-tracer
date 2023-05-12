{
  description = "A ray tracer written in rust";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.11";
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
        legacyPkgs = nixpkgs.legacyPackages."${system}";
        rust = pkgs.rust-bin.stable.${"1.65.0"}.default;
      in
      rec {
        packages.default = pkgs.callPackage ./racer-tracer.nix { cargo = rust; rustc = rust; };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ packages.default ];
          shellInputs = [ pkgs.clippy pkgs.rust-analyzer pkgs.rustfmt ];
          shellHook = ''
            export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [ pkgs.xorg.libX11 pkgs.xorg.libXcursor ] }
            cd ./racer-tracer
          '';
        };
      }
    );
}
