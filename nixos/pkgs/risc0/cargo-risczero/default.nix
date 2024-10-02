{ lib
, stdenv
, fetchFromGitHub
, rustPlatform
, pkg-config
, openssl
, darwin
, risc0CircuitRecursionPatch
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
    risc0CircuitRecursionPatch;
  pname = "cargo-risczero";
  version = "1.0.1";
  gitHash = "sha256-0Y7+Z2TEm5ZbEkbO8nSOZulGuZAgl9FdyEVNmqV7S8U=";
  cargoHash = "sha256-G3S41Je4HJCvaixjPpNWnHHJgEjTVj83p5xLkXVsASU=";
  metaDescription = "Cargo extension to help create, manage, and test RISC Zero projects";
}
