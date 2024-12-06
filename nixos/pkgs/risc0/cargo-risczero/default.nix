{ lib
, stdenv
, fetchFromGitHub
, rustPlatform
, pkg-config
, openssl
, darwin
, risc0CircuitRecursionPatch
}:

{ version ? ""
, gitHash ? ""
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
    gitHash
    cargoHash;
  pname = "cargo-risczero";
  metaDescription = "Cargo extension to help create, manage, and test RISC Zero projects";
}
