{ rustPlatform
, stdenv
, fetchFromGitHub
, pkg-config
, perl
, openssl
, lib
, darwin
, risc0CircuitRecursionPatch
}:

{ version ? ""
, hash ? ""
, cargoHash ? ""
}:
import ../mkRisc0Package.nix {
  inherit
    lib
    stdenv
    fetchFromGitHub
    rustPlatform
    pkg-config
    openssl
    darwin
    risc0CircuitRecursionPatch

    version
    hash
    cargoHash;
  pname = "r0vm";
  extraNativeBuildInputs = [ perl ];
  metaDescription = "RISC Zero zero-knowledge VM";
}
