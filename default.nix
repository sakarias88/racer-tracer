let
  sources = import ./nix/sources.nix;
  nixpkgs = with
    {
      overlay = _: pkgs:
        {
          niv = (import sources.niv { }).niv;
        };
    };
    import sources.nixpkgs
      {
        overlays = [ overlay (import sources.rust) ];
        config = { };
      };

  rustBin = nixpkgs.rust-bin.stable."1.65.0".rust;

  # rust-analyzer cannot handle symlinks
  # so we need to create a derivation with the
  # correct rust source without symlinks
  rustSrcNoSymlinks = nixpkgs.stdenv.mkDerivation {
    name = "rust-src-no-symlinks";

    rustWithSrc = (rustBin.override {
      extensions = [ "rust-src" ];
    });
    rust = rustBin;

    builder = builtins.toFile "builder.sh" ''
      source $stdenv/setup
      mkdir -p $out
      cp -r -L $rustWithSrc/lib/rustlib/src/rust/library/. $out/
      '';
  };
in
nixpkgs.stdenv.mkDerivation {
  name = "racer-tracer";
  src = nixpkgs.nix-gitignore.gitignoreSource [] ./racer-tracer;
  nativeBuildInputs = [ rustBin nixpkgs.cacert nixpkgs.xorg.libX11 ];
  configurePhase = ''
    export CARGO_HOME=$PWD
  '';

  buildPhase = ''
    cargo build --release
  '';

  checkPhase = ''
    cargo fmt -- --check
    cargo clippy
    cargo test
  '';

  doCheck = true;

  installPhase = ''
    mkdir -p $out/bin
    find target/release -executable -type f -exec cp {} $out/bin \;
  '';

  shellHook = ''
    export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${nixpkgs.lib.makeLibraryPath [ nixpkgs.xorg.libX11 nixpkgs.xorg.libXcursor ] }
    export RUST_SRC_PATH=${rustSrcNoSymlinks}
  '';
}
