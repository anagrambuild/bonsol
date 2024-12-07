{ flatbuffers
, fetchFromGitHub
}:

{ version ? ""
, hash ? ""
}:
(flatbuffers.overrideAttrs (old: {
  inherit version;
  src = fetchFromGitHub {
    inherit hash;
    inherit (old.src) owner repo;
    rev = "v${version}";
  };
}))
