# NOTE: The output of this script results in dynamically linked executables
# which have been yet to be produced in the nix store, and thus will not run
# on NixOS systems. This does not stop the executable from being run on any
# non-NixOS x86_64-linux distro that uses Nix for the development environment.
{ writeShellScriptBin }:
let
  name = "setup.sh";
in
writeShellScriptBin name (builtins.readFile ../../../${name})
