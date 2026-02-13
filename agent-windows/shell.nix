let
  pkgs = import <nixpkgs> {
    crossSystem = { config = "x86_64-w64-mingw32"; };
  };
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
  ];
  buildInputs = with pkgs.buildPackages; [
    pkgsCross.mingwW64.stdenv.cc
    pkgsCross.mingwW64.windows.pthreads
  ];
}
