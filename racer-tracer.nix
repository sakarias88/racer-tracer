{
  nix-gitignore
, stdenv
, cacert
, xorg
, lib
, cargo
, rustc
, rust
, rustfmt
, rust-analyzer
, clippy
}:
stdenv.mkDerivation {
  name = "racer-tracer";
  src = nix-gitignore.gitignoreSource [] ./racer-tracer;
  nativeBuildInputs = [ cargo rustc cacert xorg.libX11 ];

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
}
