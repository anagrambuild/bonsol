{ lib
, dockerTools
, stdenv
, autoPatchelfHook
, libclang
, libsodium
, gmp
, fetchurl

, imageDigest
, sha256
, finalImageTag
}:

let
  owner = "risczero";
  pname = "risc0-groth16-prover";
  imageName = "${owner}/${pname}";
  # Pulls the image from docker hub and builds the layered image
  # producing a tarball.
  src = dockerTools.exportImage {
    name = imageName;
    fromImage = dockerTools.pullImage {
      inherit
        imageName
        imageDigest
        sha256
        finalImageTag;
    };
    diskSize = 10240; # 10G is necessary to build the layered image.
  };
in
stdenv.mkDerivation {
  inherit pname src;
  version = finalImageTag;

  nativeBuildInputs = [ autoPatchelfHook ];

  buildInputs = [
    libclang
    # This just overrides the version of libsodium from the current one
    # available in nixpkgs to use the version required by the version
    # of rapidsnark.
    (libsodium.overrideAttrs (old: rec {
      version = "1.0.18";
      src = fetchurl {
        url = "https://download.libsodium.org/libsodium/releases/libsodium-${version}.tar.gz";
        hash = "sha256-b1BEkLNCpPikxKAvybhmy++GItXfTlRStGvhIeRmNsE=";
      };
    }))
    gmp
  ];

  # Unpack the layered image into the source directory.
  unpackPhase = ''
    tar -xf $src
  '';

  # Create the output directory where the prover is expected to be by bonsol binaries
  # and copy the files from the prover image into it.
  buildPhase = ''
    mkdir -p $out/stark
    cp app/stark_verify $out/stark/
    cp app/stark_verify.dat $out/stark/
    cp app/stark_verify_final.zkey $out/stark/
    cp usr/local/sbin/rapidsnark $out/stark/
  '';

  meta = with lib; {
    description = ''
      Utilities for performing a "stark2snark" workflow, useful for transforming a RISC Zero
      STARK proof into a Groth16 SNARK proof which is suitable for publishing on-chain.
    '';
    homepage = "https://hub.docker.com/r/risczero/risc0-groth16-prover";
    license = licenses.asl20;
    maintainers = with maintainers; [ eureka-cpu ];
    platforms = platforms.linux;
  };
}
