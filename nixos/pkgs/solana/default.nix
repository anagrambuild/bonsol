{ stdenv
, fetchFromGitHub
, fetchzip
, lib
, rustPlatform
, pkg-config
, darwin
, udev
, protobuf
, openssl
, libcxx
, rocksdb_8_3
, zlib
, clang
, llvm
, perl
, libclang
, solanaPkgs ? [
    "solana"
    "solana-bench-tps"
    "solana-faucet"
    "solana-gossip"
    "solana-install"
    "solana-keygen"
    "solana-ledger-tool"
    "solana-log-analyzer"
    "solana-net-shaper"
    "solana-validator"
    "solana-test-validator"
    "cargo-build-sbf"
    # "cargo-test-sbf"
    # "solana-dos"
    # "solana-install-init"
    # "solana-stake-accounts"
    # "solana-tokens"
    # "solana-watchtower"
  ] ++ [
    # XXX: Ensure `solana-genesis` is built LAST!
    # See https://github.com/solana-labs/solana/issues/5826
    "solana-genesis"
  ]
}:
let
  pname = "solana-cli";
  version = "1.18.22";
  hash = "sha256-MQcnxMhlD0a2cQ8xY//2K+EHgE6rvdUtqufhOw6Ib0Y=";
  # Fetches the solana source to place the Cargo.lock in the nix store so we don't have to keep track of a Cargo.lock file for it.
  solanaSource = fetchzip {
    name = "${pname}-${version}";
    url = "https://github.com/solana-labs/solana/archive/refs/tags/v${version}.zip";
    sha256 = hash;
  };
  rocksdb = rocksdb_8_3;
  inherit (darwin.apple_sdk_11_0) Libsystem;
  inherit (darwin.apple_sdk_11_0.frameworks) System IOKit AppKit Security;
in
rustPlatform.buildRustPackage {
  inherit pname version;

  src = fetchFromGitHub {
    owner = "solana-labs";
    repo = "solana";
    rev = "v${version}";
    inherit hash;
  };

  cargoLock = {
    lockFile = "${solanaSource}/Cargo.lock";
    outputHashes = {
      "crossbeam-epoch-0.9.5" = "sha256-Jf0RarsgJiXiZ+ddy0vp4jQ59J9m0k3sgXhWhCdhgws=";
      "tokio-1.29.1" = "sha256-Z/kewMCqkPVTXdoBcSaFKG5GSQAdkdpj3mAzLLCjjGk=";
      "aes-gcm-siv-0.10.3" = "sha256-N1ppxvew4B50JQWsC3xzP0X4jgyXZ5aOQ0oJMmArjW8=";
      "curve25519-dalek-3.2.1" = "sha256-FuVNFuGCyHXqKqg+sn3hocZf1KMCI092Ohk7cvLPNjQ=";
    };
  };

  cargoBuildFlags = builtins.map (n: "--bin=${n}") solanaPkgs;

  nativeBuildInputs = [ pkg-config protobuf clang llvm perl ];
  buildInputs =
    [ openssl rustPlatform.bindgenHook zlib libclang rocksdb ]
    ++ (lib.optionals stdenv.hostPlatform.isLinux [ udev ])
    ++ lib.optionals stdenv.hostPlatform.isDarwin [ Security System Libsystem libcxx IOKit AppKit ];

  postInstall = ''
    mkdir -p $out/bin/sdk/sbf
    cp -a ./sdk/sbf/* $out/bin/sdk/sbf/
  '';

  strictDeps = true;
  doCheck = false;

  # Used by build.rs in the rocksdb-sys crate. If we don't set these, it would
  # try to build RocksDB from source.
  ROCKSDB_LIB_DIR = "${rocksdb}/lib";

  # If set, always finds OpenSSL in the system, even if the vendored feature is enabled.
  OPENSSL_NO_VENDOR = "1";

  # Require this on darwin otherwise the compiler starts rambling about missing
  # cmath functions
  CPPFLAGS = lib.optionals stdenv.hostPlatform.isDarwin "-isystem ${lib.getDev libcxx}/include/c++/v1";
  LDFLAGS = lib.optionals stdenv.hostPlatform.isDarwin "-L${lib.getLib libcxx}/lib";

  meta = with lib; {
    description = "Web-Scale Blockchain for fast, secure, scalable, decentralized apps and marketplaces.";
    homepage = "https://solana.com";
    license = licenses.asl20;
    maintainers = with maintainers; [ eureka-cpu ];
    platforms = platforms.unix ++ platforms.darwin;
  };
}
