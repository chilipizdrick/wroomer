{
  description = "wroomer nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {pkgs, ...}: {
        packages = {
          wroomer = pkgs.callPackage ./package.nix {};
          wroomer-wayland = pkgs.callPackage ./package.nix {waylandSupport = true;};
        };

        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            cargo-xwin

            libGL
            libclang
            libgbm
            libxcb
            libxkbcommon
            pipewire
            pkg-config
            vulkan-loader
            wayland
          ];

          shellHook = let
            stdenv = pkgs.stdenv;
            lib = pkgs.lib;
          in ''
            export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
              $(< ${stdenv.cc}/nix-support/libc-cflags) \
              $(< ${stdenv.cc}/nix-support/cc-cflags) \
              $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
              ${lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include"} \
              ${lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include"} \
            "
          '';

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };
      };

      flake = {
        githubActions = inputs.nix-github-actions.lib.mkGithubMatrix {checks = inputs.self.packages;};
      };

      imports = [];
    };
}
