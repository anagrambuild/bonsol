{ rustPlatform
, stdenv
, fetchFromGitHub
, pkg-config
, openssl
, lib
, darwin

, pname
, version
, hash
, cargoHash
, extraNativeBuildInputs ? [ ]
, extraBuildInputs ? [ ]
, metaDescription
, risc0CircuitRecursionPatch
}:

rustPlatform.buildRustPackage rec {
  inherit pname version cargoHash;

  src = fetchFromGitHub {
    inherit hash;
    owner = "risc0";
    repo = "risc0";
    rev = "v${version}";
  };

  nativeBuildInputs = [
    pkg-config
  ] ++ extraNativeBuildInputs;

  buildInputs = [
    openssl.dev
  ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
    darwin.apple_sdk.frameworks.SystemConfiguration
  ] ++ extraBuildInputs;

  buildAndTestSubdir = "risc0/${pname}";

  doCheck = false;

  postPatch = risc0CircuitRecursionPatch;

  meta = with lib; {
    description = metaDescription;
    homepage = "https://github.com/risc0/risc0";
    changelog = "https://github.com/risc0/risc0/blob/${src.rev}/CHANGELOG.md";
    license = licenses.asl20;
    maintainers = with maintainers; [ eureka-cpu ];
    mainProgram = "${pname}";
  };
}
