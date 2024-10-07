{ writeShellScriptBin
, bonsol-relay
, use-nix ? false # whether or not to use the nix pre-built bonsol-relay binary
}:
let
  relay_path = "${bonsol-relay}/bin/relay";
  name = "run-relay.sh";

  # Patches that will use the nix built version of bonsol-relay.
  # This also avoids unnecessary build times.
  from = [ "cargo run --release -p relay --" "cargo run --release -p relay --features metal --" ];
  to = [ relay_path relay_path ];
  contents = (builtins.readFile ../../../${name});
in
writeShellScriptBin name (
  if use-nix then
    (builtins.replaceStrings from to contents)
  else
    contents
)
