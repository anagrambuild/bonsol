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
  pname = "r0vm";
  version = "1.0.1";
  gitHash = "sha256-0Y7+Z2TEm5ZbEkbO8nSOZulGuZAgl9FdyEVNmqV7S8U=";
  cargoHash = "sha256-3DwrWkjPCE4f/FHjzWyRGAXJPv30B4Ce8fh2oKDhpMM=";
  extraNativeBuildInputs = [ perl ];
  metaDescription = "RISC Zero zero-knowledge VM";
}
