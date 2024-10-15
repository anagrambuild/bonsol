{ writeShellScriptBin
, bonsol-node
, use-nix ? false # whether or not to use the nix pre-built bonsol-node binary
}:
let
  node_path = "${bonsol-node}/bin/bonsol-node";
  name = "run-node.sh";

  # Patches that will use the nix built version of bonsol-node.
  # This also avoids unnecessary build times.
  from = [ "cargo run --release -p bonsol-node --" "cargo run --release -p bonsol-node --features metal --" ];
  to = [ node_path node_path ];
  contents = (builtins.readFile ../../../${name});
in
writeShellScriptBin name (
  if use-nix then
    (builtins.replaceStrings from to contents)
  else
    contents
)
