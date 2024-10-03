{ writeTextFile }:
let
  name = "validator.sh";
in
writeTextFile {
  inherit name;
  text = builtins.readFile ../../../validator.sh;
  executable = true;
  destination = "/${name}";
}
