{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
  buildInputs = [
    alsaLib
    cargo
    cmake
    expat
    freetype
    pkgconfig
    xlibs.libX11
  ];

  APPEND_LIBRARY_PATH = lib.makeLibraryPath [
    vulkan-loader
    xlibs.libXcursor
    xlibs.libXrandr
    xlibs.libXi
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';
}
