{ writeTextFile
, risc0-groth16-prover
}:
let
  name = "Node.toml";
  text = (builtins.replaceStrings [ "./stark/" ] [ "${risc0-groth16-prover}/stark/" ] (builtins.readFile ../../../${name}));
in
writeTextFile {
  inherit name text;
}
