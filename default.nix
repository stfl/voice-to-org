{pkgs ? import <nixpkgs> {}}:
with pkgs;
  rustPlatform.buildRustPackage rec {
    pname = "voice-to-org";
    version = "0.1";
    cargoLock.lockFile = ./Cargo.lock;
    src = nix-gitignore.gitignoreSource [] ./.;

    # OPENSSL_NO_VENDOR = 1;

    # TODO nativeBuildInput work with nix develop but not with direnv?!...
    # nativeBuildInputs = [
    #   openssl
    #   pkg-config
    # ];

    buildInputs = [
      openai-whisper
      openssl
      pkg-config
    ];
  }
