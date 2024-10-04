{ writeShellScriptBin }:
let
  name = "run-relay.sh";
in
writeShellScriptBin name (builtins.readFile ../../../${name})
