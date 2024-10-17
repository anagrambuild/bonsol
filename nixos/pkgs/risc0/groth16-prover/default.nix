{ dockerTools
, docker
, corepack_22
, runCommand
, setup

, imageDigest
, sha256
, finalImageTag
}:

let
  owner = "risczero";
  pname = "risc0-groth16-prover";
  imageName = "${owner}/${pname}";
  risc0-groth16-prover = dockerTools.pullImage {
    inherit
      imageName
      imageDigest
      sha256
      finalImageTag;
  };
  risc0-groth16-prover-stream = dockerTools.streamLayeredImage {
    name = imageName;
    tag = finalImageTag;
    fromImage = risc0-groth16-prover;
    config.Cmd = [ "${pname}" ];
  };
in
runCommand "${pname}" {
  buildInputs = [
    docker
    corepack_22
  ];
} ''
  mkdir -p $out/node/stark
  mkdir -p $out/vkey

  # Load the docker image we've pulled above by streaming from stdin
  # See https://nixos.org/manual/nixpkgs/stable/#ssec-pkgs-dockerTools-streamLayeredImage
  ${risc0-groth16-prover-stream} | ${docker}/bin/docker image load
  # ${setup}/bin/setup.sh
''

