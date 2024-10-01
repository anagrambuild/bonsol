{ lib
, stdenv
, fetchFromGitHub
, fetchurl
, rustPlatform
, pkg-config
, perl
, openssl
, darwin
}:

rustPlatform.buildRustPackage rec {
  pname = "cargo-risczero";
  version = "1.0.1";

  src = fetchFromGitHub {
    owner = "risc0";
    repo = "risc0";
    rev = "v${version}";
    hash = "sha256-0Y7+Z2TEm5ZbEkbO8nSOZulGuZAgl9FdyEVNmqV7S8U=";
  };
  cargoHash = "sha256-G3S41Je4HJCvaixjPpNWnHHJgEjTVj83p5xLkXVsASU=";

  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl.dev
  ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  buildAndTestSubdir = "risc0/cargo-risczero";
  doCheck = false;
  postPatch =
    let
      # see https://github.com/risc0/risc0/blob/v1.0.5/risc0/circuit/recursion/build.rs
      sha256Hash = "4e8496469e1efa00efb3630d261abf345e6b2905fb64b4f3a297be88ebdf83d2";
      recursionZkr = fetchurl {
        name = "recursion_zkr.zip";
        url = "https://risc0-artifacts.s3.us-west-2.amazonaws.com/zkr/${sha256Hash}.zip";
        hash = "sha256-ToSWRp4e+gDvs2MNJhq/NF5rKQX7ZLTzope+iOvfg9I=";
      };
    in
    ''
      ln -sf ${recursionZkr} ./risc0/circuit/recursion/src/recursion_zkr.zip
    '';

  meta = with lib; {
    description = "Cargo extension to help create, manage, and test RISC Zero projects";
    mainProgram = "cargo-risczero";
    homepage = "https://risczero.com";
    license = with licenses; [ asl20 ];
    maintainers = with maintainers; [ eureka-cpu ];
  };
}
