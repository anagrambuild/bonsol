{ lib
, stdenv
, fetchFromGitHub
, fetchurl
, rustPlatform
, pkg-config
, openssl
, darwin
, risc0CircuitRecursionPatch
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
  postPatch = risc0CircuitRecursionPatch;

  meta = with lib; {
    description = "Cargo extension to help create, manage, and test RISC Zero projects";
    mainProgram = "cargo-risczero";
    homepage = "https://risczero.com";
    license = with licenses; [ asl20 ];
    maintainers = with maintainers; [ eureka-cpu ];
  };
}
