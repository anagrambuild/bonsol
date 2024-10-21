{ dockerTools
, stdenv

, imageDigest
, sha256
, finalImageTag
}:

let
  owner = "risczero";
  pname = "risc0-groth16-prover";
  imageName = "${owner}/${pname}";
  src = dockerTools.pullImage {
    inherit
      imageName
      imageDigest
      sha256
      finalImageTag;
  };
in
stdenv.mkDerivation {
  inherit pname src;
  version = finalImageTag;

  unpackPhase = ''
    mkdir -p $out/app
    # Extract files from the Docker image tarball
    tar -xf ${src} -C $out/app
  '';

  buildPhase = ''
    # Create output directories
    mkdir -p $out/stark
    
    # Copy specific files to the output
    cp $out/app/stark_verify $out/stark/
    cp $out/app/stark_verify.dat $out/stark/
    cp $out/app/stark_verify_final.zkey $out/stark/
    cp $out/usr/local/sbin/rapidsnark $out/stark/
  '';
}
