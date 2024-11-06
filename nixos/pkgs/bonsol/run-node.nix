{ writeShellScriptBin
, bonsol-node
, node_toml
, use-nix ? false # whether or not to use the nix pre-built bonsol-node binary
}:
let
  node_path = "${bonsol-node}/bin/bonsol-node";
  name = "run-node.sh";

  # Patches that will use the nix built version of bonsol-node.
  # This also avoids unnecessary build times.
  from = [ "cargo run --release -p bonsol-node --" "cargo run --release -p bonsol-node --features metal --" ];
  to = [ node_path node_path ];

  # Override the path to the Node.toml to reference the gorth16 tools in the nix store
  contents = (builtins.replaceStrings [ "./Node.toml" ] [ "${node_toml}" ] (builtins.readFile ../../../bin/${name}));
in
writeShellScriptBin name (
  if use-nix then
    (builtins.replaceStrings from to contents)
  else
    contents
)
