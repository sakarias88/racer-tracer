{
  nix-gitignore
, stdenv
, cacert
, xorg
, lib
, rust
}:
stdenv.mkDerivation {
  name = "racer-tracer";
  src = nix-gitignore.gitignoreSource [] ./racer-tracer;
  nativeBuildInputs = [ rust cacert ];

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
