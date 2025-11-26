{
  lib,
  rustPlatform,
  pkg-config,
  wayland,
  libxkbcommon,
  vulkan-loader,
  libgbm,
  libGL,
  pipewire,
  libclang,
  stdenv,
  libxcb,
  waylandSupport ? false,
}:
rustPlatform.buildRustPackage rec {
  pname = "wroomer";
  name = pname;
  version = "0.1.0";
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "libwayshot-0.3.2-dev" = "sha256-yPO39u+EUW18PclkZjkxWZr4Q7nilew4oC3rP+Do2D4=";
    };
  };

  src = lib.cleanSource ./.;

  cargoBuildFlags = lib.optionals waylandSupport [
    "--no-default-features"
    "--features"
    "wayland"
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    wayland
    libxkbcommon
    vulkan-loader
    libgbm
    pipewire
    libclang
    libxcb
    libGL
  ];

  LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";

  preBuild = ''
      export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
      $(< ${stdenv.cc}/nix-support/libc-cflags) \
      $(< ${stdenv.cc}/nix-support/cc-cflags) \
      $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
      ${lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include"} \
      ${lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include"} \
    "
  '';

  postFixup = let
    rpath = lib.makeLibraryPath [
      wayland
      vulkan-loader
      libxkbcommon
      libgbm
      pipewire
      libxcb
      libGL
    ];
  in ''
    rpath=$(patchelf --print-rpath $out/bin/wroomer)
    patchelf --set-rpath "$rpath:${rpath}" $out/bin/wroomer
  '';
}
