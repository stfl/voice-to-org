{
  description = "An AI powered voice transcription pipeline into your knowledge base";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # We want to use packages from the binary cache
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in rec {
      packages.default = pkgs.callPackage ./default.nix {};

      devShell = pkgs.mkShell {
        inputsFrom = [packages.default];
        buildInputs = with pkgs; [
          rust-analyzer
          rustfmt
          clippy
        ];
      };
    });
}
