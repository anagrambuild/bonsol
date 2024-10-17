{ dockerTools
, docker
, corepack_22
, runCommand

, imageDigest
, sha256
, finalImageTag
}:

let
  imageName = "risc0-groth16-prover";
  risc0-groth16-prover = dockerTools.pullImage {
    inherit
      imageName
      imageDigest
      sha256
      finalImageTag;
    finalImageName = imageName;
  };
  risc0-groth16-prover-stream = dockerTools.streamLayeredImage {
    name = imageName;
    tag = finalImageTag;
    fromImage = risc0-groth16-prover;
    config.Cmd = [ "${imageName}" ];
  };
in
runCommand "generate-groth16-prover" {
  buildInputs = [
    docker
    corepack_22
  ];
} ''
  # Load the docker image we've pulled above
  ${risc0-groth16-prover-stream} | ${docker}/bin/docker image load
  
''
