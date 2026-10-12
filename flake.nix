{
  description = "wroomer nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    nix-github-actions,
    ...
  }: let
    systems = [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-darwin"
      "x86_64-linux"
    ];
    lib = nixpkgs.lib;
    forAllSystems = lib.genAttrs systems;
    forAllPkgs = f: forAllSystems (system: let pkgs = nixpkgs.legacyPackages.${system}; in f pkgs);
  in {
    packages =
      forAllPkgs (pkgs: {
        wroomer = pkgs.callPackage ./package.nix {};
      })
      // (lib.genAttrs ["x86_64-linux" "aarch64-linux"] (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in {wroomer-wayland = pkgs.callPackage ./package.nix {extraWaylandSupport = true;};}
      ));

    devShells = forAllPkgs (pkgs: {
      default = with pkgs; let
        libs = [
          libGL
          libclang
          libgbm
          libxcb
          libxkbcommon
          pipewire
          vulkan-loader
          wayland
        ];
      in
        mkShell {
          buildInputs = [pkg-config] ++ libs;

          shellHook = let
            stdenv = pkgs.stdenv;
            lib = pkgs.lib;
          in ''
            export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
              $(< ${stdenv.cc}/nix-support/libc-cflags) \
              $(< ${stdenv.cc}/nix-support/cc-cflags) \
              $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
              ${lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include"} \
              ${lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include"}
            "
          '';

          LD_LIBRARY_PATH = lib.makeLibraryPath libs;
        };
    });

    githubActions = nix-github-actions.lib.mkGithubMatrix {checks = self.packages;};
  };
}
