{ writeTextFile }:
let
  name = "run-relay.sh";
in
writeTextFile {
  inherit name;
  text = builtins.readFile ../../../run-relay.sh;
  executable = true;
  destination = "/${name}";
}
