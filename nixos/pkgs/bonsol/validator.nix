{ writeShellScriptBin }:
let
  name = "validator.sh";
in
writeShellScriptBin name (builtins.readFile ../../../bin/${name})
