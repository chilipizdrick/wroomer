{
  lib,
  rustPlatform,
  pkg-config,
  wayland,
  libxkbcommon,
  vulkan-loader,
  libgbm,
  libGL,
}:
rustPlatform.buildRustPackage rec {
  pname = "wroomer";
  name = pname;
  version = "0.1.0";
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "libwayshot-0.3.2-dev" = "sha256-frCOq0iX8F5MP1UteeNSogtIqjG2lb8b+USd48MkPnA=";
    };
  };

  src = lib.cleanSource ./.;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    wayland
    libxkbcommon
    vulkan-loader
    libgbm
    libGL
  ];

  postFixup = let
    rpathWayland = lib.makeLibraryPath [
      wayland
      vulkan-loader
      libxkbcommon
      libgbm
      libGL
    ];
  in ''
    rpath=$(patchelf --print-rpath $out/bin/wroomer)
    patchelf --set-rpath "$rpath:${rpathWayland}" $out/bin/wroomer
  '';
}
