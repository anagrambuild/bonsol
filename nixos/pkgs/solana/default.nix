{ stdenv
, fetchFromGitHub
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
, cmake
, autoPatchelfHook
, solana-platform-tools
, solanaPkgs ? [
    "cargo-build-bpf"
    "cargo-build-sbf"
    "cargo-test-bpf"
    "cargo-test-sbf"
    "rbpf-cli"
    "solana"
    "solana-bench-tps"
    "solana-dos"
    "solana-faucet"
    "solana-gossip"
    "solana-install"
    "solana-keygen"
    "solana-ledger-tool"
    "solana-log-analyzer"
    "solana-net-shaper"
    "solana-stake-accounts"
    "solana-test-validator"
    "solana-tokens"
    "solana-validator"
    "solana-watchtower"
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
  src = fetchFromGitHub {
    owner = "solana-labs";
    repo = "solana";
    rev = "v${version}";
    inherit hash;
  };
  rocksdb = rocksdb_8_3;
  inherit (darwin.apple_sdk_11_0) Libsystem;
  inherit (darwin.apple_sdk_11_0.frameworks) System IOKit AppKit Security;
in
rustPlatform.buildRustPackage {
  inherit pname version src;
  strictDeps = true;
  doCheck = false;

  # Build only the specified solana packages
  cargoBuildFlags = builtins.map (n: "--bin=${n}") solanaPkgs;

  cargoLock = {
    lockFile = "${src}/Cargo.lock"; # NOTE: To support backwards compatibility, this will need to be in a local file
    outputHashes = {
      "crossbeam-epoch-0.9.5" = "sha256-Jf0RarsgJiXiZ+ddy0vp4jQ59J9m0k3sgXhWhCdhgws=";
      "tokio-1.29.1" = "sha256-Z/kewMCqkPVTXdoBcSaFKG5GSQAdkdpj3mAzLLCjjGk=";
      "aes-gcm-siv-0.10.3" = "sha256-N1ppxvew4B50JQWsC3xzP0X4jgyXZ5aOQ0oJMmArjW8=";
      "curve25519-dalek-3.2.1" = "sha256-FuVNFuGCyHXqKqg+sn3hocZf1KMCI092Ohk7cvLPNjQ=";
    };
  };

  # Solana tries to dynamically link platform-tools to `~/.cache/solana/<version>/platform-tools/`
  # and will forcibly delete, download and replace symbolic links regardless of if they already
  # exist and are up-to-date. This just stops it from doing that since we handle that through
  # `autoPatchelfHook` and creating a symbolic link to the dependencies in the nix store.
  cargoPatches = [
    ./patches/v1.18.22/cargo-build-sbf.diff
  ];

  nativeBuildInputs = [
    pkg-config
    protobuf
    clang
    llvm
    perl
    autoPatchelfHook
    cmake
    libclang.lib
  ];

  buildInputs = [
    openssl
    rustPlatform.bindgenHook
    zlib
    libclang.lib
    rocksdb
  ] ++ (lib.optionals stdenv.hostPlatform.isLinux [ udev ])
  ++ lib.optionals stdenv.hostPlatform.isDarwin [
    Security
    System
    Libsystem
    libcxx
    IOKit
    AppKit
  ];

  postInstall = ''
    # The sbf portion of the sdk is currently the only part of the sdk that is supported for version
    # 1.18.22 however earlier versions relied on bpf (berklee package filter) sdk. Future versions
    # may evolve as well, as solana transitions to agave.
    mkdir -p $out/bin/sdk/sbf/dependencies
    cp -a ./sdk/sbf/* $out/bin/sdk/sbf/

    # Avoid dynamically linking the sdk by symlinking the `platform-tools` in `sdk/sbf/dependencies`
    # as specified in the source code where they are expected to be found if not in `~/.cache/solana`.
    ln -s ${solana-platform-tools}/v${solana-platform-tools.version}/${solana-platform-tools.pname} $out/bin/sdk/sbf/dependencies/

    # Mimic placement of platform-tools for historical reasons.
    # `platfrom-tools` is dynamically linked to `~/.cache/solana/<version>/platfrom-tools`
    # but we handle this by symbolically linking to the nix store path.
    mkdir -p $out/bin/.cache/solana
    ln -s ${solana-platform-tools}/v${solana-platform-tools.version} $out/bin/.cache/solana
  '';

  # Used by build.rs in the rocksdb-sys crate. If we don't set these, it would
  # try to build RocksDB from source.
  ROCKSDB_LIB_DIR = "${rocksdb}/lib";

  # Require this on darwin otherwise the compiler starts rambling about missing
  # cmath functions
  CPPFLAGS = lib.optionals stdenv.hostPlatform.isDarwin "-isystem ${lib.getDev libcxx}/include/c++/v1";
  LDFLAGS = lib.optionals stdenv.hostPlatform.isDarwin "-L${lib.getLib libcxx}/lib";

  # This is run only when the package is used in a nix-shell, in the event that it's consumed by outside sources.
  # This effectively "re-creates" the logic removed from `cargo-build-sbf` that forcibly removes and symlinks `platform-tools`.
  shellHook = ''
    # make cargo-build-sbf of platfrom-tools
    export SBF_SDK_PATH=$out/bin/sdk
    cache_dir="''$HOME/.cache/solana"
    # if the cache dir exists, ask if the user wants to remove it
    if [[ -d "''$cache_dir" ]]; then
      read -p "''$cache_dir will be removed and replaced with a nix store symbolic link, continue? (y/n): " response
      response=$(echo "$response" | tr '[:upper:]' '[:lower:]')
      if [[ "''$response" == "y" || "''$response" == "yes" ]]; then
        rm -rf "''$cache_dir"
      else
        exit 0
      fi
    fi
    # create the cache dir
    mkdir -p "''$cache_dir"
    # symlink the platform tools to the cache dir
    ln -s ${solana-platform-tools}/v${solana-platform-tools.version} ''$cache_dir
  '';

  meta = with lib; {
    description = "Web-Scale Blockchain for fast, secure, scalable, decentralized apps and marketplaces.";
    homepage = "https://solana.com";
    license = licenses.asl20;
    maintainers = with maintainers; [ eureka-cpu ];
    platforms = platforms.unix ++ platforms.darwin;
  };
}
