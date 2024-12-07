{ lib
, stdenv
, fetchzip
, autoPatchelfHook
, zlib
, openssl
, libpanel
, ncurses
, python39
, libxml2
, lldb
}:

# The platform tools version is baked in at https://github.com/solana-labs/solana/blob/27eff8408b7223bb3c4ab70523f8a8dca3ca6645/sdk/cargo-build-sbf/src/main.rs#L916
{ version ? ""
, hash ? ""
}:
let
  owner = "anza-xyz";
  repo = "platform-tools";
  system = "linux-x86_64"; # TODO: Add other archs
  src = fetchzip {
    inherit hash;
    name = "${owner}-${repo}-${version}-${system}";
    url = "https://github.com/${owner}/${repo}/releases/download/v${version}/platform-tools-${system}.tar.bz2";
    stripRoot = false;
  };
  python38 = (python39.override {
    sourceVersion = {
      major = "3";
      minor = "8";
      patch = "9";
      suffix = "";
    };
    hash = "sha256-XjkfPsRdopVEGcqwvq79i+OIlepc4zV3w+wUlAxLlXI=";
  });
in
stdenv.mkDerivation {
  inherit src version;
  pname = repo;

  nativeBuildInputs = [ autoPatchelfHook ];
  buildInputs = [
    zlib
    stdenv.cc.cc
    openssl
    libpanel
    ncurses
    libxml2
    lldb
    python38
  ];

  installPhase = ''
    mkdir -p $out/v${version}/platform-tools
    cp -r ${src}/* $out/v${version}/platform-tools/
  '';

  meta = with lib; {
    homepage = "https://github.com/anza-xyz/platform-tools";
    description = ''
      Builds Clang and Rust compiler binaries that incorporate customizations and fixes required by Solana but not yet upstreamed into Rust or LLVM.
    '';
    license = licenses.asl20;
    maintainers = with maintainers; [ eureka-cpu ];
    platforms = platforms.unix ++ platforms.darwin; # NOTE: See todo above
  };
}
