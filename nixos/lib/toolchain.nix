{ lib
, callPackage
, fetchurl
, mkRustToolchainFromTOML
}:

let
  rustToolchainPath = ../../rust-toolchain.toml;
  rustToolchainTOML = lib.importTOML rustToolchainPath;
  rustToolchain = mkRustToolchainFromTOML rustToolchainPath rustToolchainTOML.toolchain.hash;
in
with rustToolchainTOML.toolchain.system-dependencies;
{
  inherit rustToolchain;

  flatc = callPackage ../pkgs/flatc { } {
    inherit (flatc) version hash;
  };

  risc0 = rec {
    # Returns true if the dependency requires `risc0-circuit-recursion` as part of its build.
    isRisc0CircuitRecursion = p: lib.hasPrefix
      "git+https://github.com/anagrambuild/risc0?branch=v1.0.1-bonsai-fix#189829d0b84d57e8928a85aa4fac60dd6ce45ea9"
      p.source;

    # Pre-pull the zkr file in order to apply in the postPatch phase for dependencies that require `risc0-circuit-recursion`.
    risc0CircuitRecursionPatch =
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

    r0vm = callPackage ../pkgs/risc0/r0vm {
      inherit risc0CircuitRecursionPatch;
    } {
      inherit (risc0) version hash;
      cargoHash = "sha256-3DwrWkjPCE4f/FHjzWyRGAXJPv30B4Ce8fh2oKDhpMM=";
    };

    cargo-risczero = callPackage ../pkgs/risc0/cargo-risczero {
      inherit risc0CircuitRecursionPatch;
    } {
      inherit (risc0) version hash;
      cargoHash = "sha256-G3S41Je4HJCvaixjPpNWnHHJgEjTVj83p5xLkXVsASU=";
    };

    risc0-groth16-prover = callPackage ../pkgs/risc0/groth16-prover { } {
      inherit (groth16-prover) version hash;
      imageDigest = "sha256:5a862bac2c5c070ec50ff615572a05d870c1372818cf0f5d8bb9effc101590c8";
    };
  };

  solana = rec {
    solana-platform-tools = callPackage ../pkgs/solana/platform-tools { } {
      inherit (platform-tools) version hash;
    };

    solana-cli = callPackage ../pkgs/solana { inherit solana-platform-tools; } {
      inherit (solana) version hash;
    };
  };
}
