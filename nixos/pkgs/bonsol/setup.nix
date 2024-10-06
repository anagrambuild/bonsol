{ writeShellScriptBin }:
let
  name = "setup.sh";
in
writeShellScriptBin name (builtins.readFile ../../../${name})
