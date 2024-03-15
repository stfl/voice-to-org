{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "voice-to-org";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.nix-gitignore.gitignoreSource [] ./.;

  buildInputs = with pkgs; [
    openai-whisper
  ];
}
