{
  description = "wroomer nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {pkgs, ...}: {
        packages = rec {
          wroomer = pkgs.callPackage ./package.nix {};
          default = wroomer;
        };
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            pkg-config
            wayland
            vulkan-loader
            libxkbcommon
            libgbm
            libGL
          ];
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };
      };
      imports = [];
      flake = {};
    };
}
