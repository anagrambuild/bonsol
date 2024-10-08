{
  description = "Build a cargo workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    nix-core = {
      url = "github:Cloud-Scythe-Labs/nix-core";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, nix-core, advisory-db, ... }:
    with flake-utils.lib;
    eachSystem
      (with system; [
        # Currently only known to run on x86-linux but this may change soon
        x86_64-linux
      ])
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          inherit (pkgs) lib;

          rustToolchain = nix-core.toolchains.${system}.mkRustToolchainFromTOML
            ./rust-toolchain.toml
            "sha256-VZZnlyP69+Y3crrLHQyJirqlHrTtGTsyiSnZB8jEvVo=";
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain.fenix-pkgs;
          workspace = rec {
            root = ./.;
            src = craneLib.cleanCargoSource root;
            canonicalizePath = crate: root + "/${crate}";
            canonicalizePaths = crates: map (crate: canonicalizePath crate) crates;
          };

          # Returns true if the dependency requires `risc0-circuit-recursion` as part of its build.
          isRisc0CircuitRecursion = p: lib.hasPrefix
            "git+https://github.com/anagrambuild/risc0?branch=v1.0.1-bonsai-fix#189829d0b84d57e8928a85aa4fac60dd6ce45ea9"
            p.source;
          # Pre-pull the zkr file in order to apply in the postPatch phase for dependencies that require `risc0-circuit-recursion`.
          risc0CircuitRecursionPatch =
            let
              # see https://github.com/risc0/risc0/blob/v1.0.5/risc0/circuit/recursion/build.rs
              sha256Hash = "4e8496469e1efa00efb3630d261abf345e6b2905fb64b4f3a297be88ebdf83d2";
              recursionZkr = pkgs.fetchurl {
                name = "recursion_zkr.zip";
                url = "https://risc0-artifacts.s3.us-west-2.amazonaws.com/zkr/${sha256Hash}.zip";
                hash = "sha256-ToSWRp4e+gDvs2MNJhq/NF5rKQX7ZLTzope+iOvfg9I=";
              };
            in
            ''
              ln -sf ${recursionZkr} ./risc0/circuit/recursion/src/recursion_zkr.zip
            '';
          # Patch dependencies that require `risc0-circuit-recursion`.
          cargoVendorDir = craneLib.vendorCargoDeps (workspace // {
            overrideVendorGitCheckout = ps: drv:
              if lib.any (p: (isRisc0CircuitRecursion p)) ps then
              # Apply the patch for fetching the zkr zip file.
                drv.overrideAttrs
                  {
                    postPatch = risc0CircuitRecursionPatch;
                  }
              else
              # Nothing to change, leave the derivations as is.
                drv;
          });

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            inherit (workspace) src;
            inherit cargoVendorDir;
            strictDeps = true;

            nativeBuildInputs = with pkgs; [
              pkg-config
              perl
              autoPatchelfHook
            ];

            buildInputs = with pkgs; [
              openssl.dev
              libgcc
              libclang.lib
            ];
          };

          # Build *just* the cargo dependencies (of the entire workspace),
          # so we can reuse all of that work (e.g. via cachix) when running in CI
          # It is *highly* recommended to use something like cargo-hakari to avoid
          # cache misses when building individual top-level-crates
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          individualCrateArgs = commonArgs // {
            inherit cargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            doCheck = false;
          };

          # Function for including a set of files for a specific crate,
          # avoiding unnecessary files.
          fileSetForCrate = crate: deps: lib.fileset.toSource {
            inherit (workspace) root;
            fileset = lib.fileset.unions ([
              ./Cargo.toml
              ./Cargo.lock
              (workspace.canonicalizePath crate)
            ] ++ (workspace.canonicalizePaths deps));
          };

          # Build the top-level crates of the workspace as individual derivations.
          # This allows consumers to only depend on (and build) only what they need.
          # Though it is possible to build the entire workspace as a single derivation,
          # in this case the workspace itself is not a package.
          #
          # Function for creating a crate derivation, which takes the relative path
          # to the crate as a string, and a list of any of the workspace crates
          # that it will need in order to build.
          # NOTE: All paths exclude the root, eg "my/dep" not "./my/dep". Root is mapped
          # during file set construction.
          #
          # Example:
          # ```nix
          #   my-crate =
          #     let
          #       deps = [ "path/to/dep1" "path/to/dep2" ];
          #     in
          #     mkCrateDrv "path/to/crate" deps;
          # ```
          mkCrateDrv = name: crate: deps:
            let
              manifest = craneLib.crateNameFromCargoToml {
                cargoToml = ((workspace.canonicalizePath crate) + "/Cargo.toml");
              };
            in
            craneLib.buildPackage (individualCrateArgs // {
              inherit (manifest) version pname;
              cargoExtraArgs = "--locked --bin ${name}";
              src = fileSetForCrate crate deps;
            });

          # The root Cargo.toml requires all of the workspace crates, otherwise this would be a bit neater.
          bonsol-cli = mkCrateDrv "bonsol" "cli" [ "sdk" "onchain" "schemas-rust" "iop" "relay" ];
          bonsol-relay = mkCrateDrv "relay" "relay" [ "sdk" "onchain" "schemas-rust" "iop" "cli" ];

          setup = pkgs.callPackage ./nixos/pkgs/bonsol/setup.nix { };
          validator = pkgs.callPackage ./nixos/pkgs/bonsol/validator.nix { };
          run-relay = pkgs.callPackage ./nixos/pkgs/bonsol/run-relay.nix { inherit bonsol-relay; };

          # Internally managed versions of risc0 binaries that are pinned to
          # the version that bonsol relies on.
          cargo-risczero = pkgs.callPackage ./nixos/pkgs/risc0/cargo-risczero {
            inherit risc0CircuitRecursionPatch;
          };
          r0vm = pkgs.callPackage ./nixos/pkgs/risc0/r0vm {
            inherit risc0CircuitRecursionPatch;
          };
          solana-platform-tools = pkgs.callPackage ./nixos/pkgs/solana/platform-tools { };
          solana-cli = pkgs.callPackage ./nixos/pkgs/solana { inherit solana-platform-tools; };
        in
        {
          checks = {
            # Build the crates as part of `nix flake check` for convenience
            inherit
              bonsol-cli
              bonsol-relay
              cargo-risczero
              r0vm;

            # Run clippy (and deny all warnings) on the workspace source,
            # again, reusing the dependency artifacts from above.
            #
            # Note that this is done as a separate derivation so that
            # we can block the CI if there are issues here, but not
            # prevent downstream consumers from building our crate by itself.
            # TODO: uncomment once all clippy lints are fixed
            # workspace-clippy = craneLib.cargoClippy (commonArgs // {
            #   inherit cargoArtifacts;
            #   cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            # });

            workspace-doc = craneLib.cargoDoc (commonArgs // {
              inherit cargoArtifacts;
            });

            # Check formatting
            workspace-fmt = craneLib.cargoFmt {
              inherit (workspace) src;
            };

            workspace-toml-fmt = craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices workspace.src [ ".toml" ];
              # taplo arguments can be further customized below as needed
              # taploExtraArgs = "--config ./taplo.toml";
            };

            # Audit dependencies
            # TODO: Uncoment once all audits are fixed
            # workspace-audit = craneLib.cargoAudit {
            #   inherit (workspace) src;
            #   inherit advisory-db;
            # };

            # Audit licenses
            workspace-deny = craneLib.cargoDeny {
              inherit (workspace) src;
            };

            # Run tests with cargo-nextest
            # Consider setting `doCheck = false` on other crate derivations
            # if you do not want the tests to run twice
            workspace-nextest = craneLib.cargoNextest (commonArgs // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            });

            # TODO: Consider using cargo-hakari workspace hack for dealing with
            # the unsightly requirements of the iop crate.
            # Ensure that cargo-hakari is up to date
            # workspace-hakari = craneLib.mkCargoDerivation {
            #   inherit src;
            #   pname = "my-workspace-hakari";
            #   cargoArtifacts = null;
            #   doInstallCargoArtifacts = false;

            #   buildPhaseCargoCommand = ''
            #     cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
            #     cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
            #     cargo hakari verify
            #   '';

            #   nativeBuildInputs = [
            #     pkgs.cargo-hakari
            #   ];
            # };
          };

          packages = {
            inherit
              bonsol-cli
              bonsol-relay

              setup
              validator
              run-relay

              cargo-risczero
              r0vm
              solana-cli
              solana-platform-tools;

            simple-e2e-script = pkgs.writeShellApplication {
              name = "simple-e2e-test";

              runtimeInputs = with pkgs; [
                docker
                corepack_22
                nodejs_22
                python3
                udev
                rustup
              ] ++ [
                r0vm
                cargo-risczero
                solana-cli
                bonsol-cli
                bonsol-relay
                setup
                validator
                (run-relay.override {
                  use-nix = true;
                })
              ];

              text = ''
                ${setup}/bin/setup.sh
                ${bonsol-cli}/bin/bonsol --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 build -z images/simple
                ${validator}/bin/validator.sh &
                validator_pid=$!
                sleep 25
                ${run-relay}/bin/run-relay.sh &
                relay_pid=$!
                sleep 25
                ${bonsol-cli}/bin/bonsol --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 deploy -m images/simple/manifest.json -t url --url https://bonsol-public-images.s3.amazonaws.com/simple-7cb4887749266c099ad1793e8a7d486a27ff1426d614ec0cc9ff50e686d17699
                sleep 20
                resp = $(${bonsol-cli}/bin/bonsol --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 execute -f testing-examples/example-execution-request.json -x 2000 -m 2000 -w)
                kill $validator_pid
                kill $relay_pid
                if [[ "$resp" =~ "success" ]]; then
                  exit 0
                else
                  echo "response was not success"
                  exit 1
                fi
              '';

              checkPhase = "true";
            };
          };

          apps = { };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              # pkgs.cargo-hakari

              nil # nix lsp
              nixpkgs-fmt # nix formatter
              rustup

              # `setup.sh` dependencies
              docker
              corepack_22
              nodejs_22
              python3
              udev

              # checked for at runtime but never used
              cargo-binstall
            ] ++ [
              setup
              validator
              run-relay
              r0vm
              cargo-risczero
              solana-cli
            ];

            # Useful for debugging, sets the path that `cargo-build-sbf` will use to find `platform-tools`
            #
            # SBF_SDK_PATH = "${solana-cli}/bin/sdk/sbf"; # This is the default

            shellHook = ''
              # TODO: use `rustup toolchain link` to link fenix toolchain to rustup as the override toolchain
              cache_dir="''$HOME/.cache/solana"
              # if the cache dir exists, ask if the user wants to remove it
              if [[ -d "''$cache_dir" ]]; then
                read -p "'$cache_dir' will be removed and replaced with a nix store symbolic link, continue? (y/n): " response
                response=$(echo "$response" | tr '[:upper:]' '[:lower:]')
                if [[ "''$response" == "y" || "''$response" == "yes" ]]; then
                  rm -rf "''$cache_dir"
                  # create the cache dir
                  mkdir -p "''$cache_dir"
                  # symlink the platform tools to the cache dir
                  ln -s ${solana-platform-tools}/v${solana-platform-tools.version} ''$cache_dir
                fi
              else
                # create the cache dir
                mkdir -p "''$cache_dir"
                # symlink the platform tools to the cache dir
                ln -s ${solana-platform-tools}/v${solana-platform-tools.version} ''$cache_dir
              fi
            '';
          };

          # Run nix fmt to format nix files in file tree
          # using the specified formatter
          formatter = pkgs.nixpkgs-fmt;
        });
}
