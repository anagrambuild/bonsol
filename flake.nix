{
  description = ''
    Build and develop Bonsol programs without jeoprodizing your existing Solana or Risc0 toolchain.
    Ensure all dependencies align with the requirements set by the Bonsol project.
  '';

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

  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, nix-core, ... }:
    with flake-utils.lib;
    eachSystem
      (with system; [
        # Currently only supported on x86-linux due to groth16-prover
        x86_64-linux
      ])
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          inherit (pkgs) lib;

          # System dependencies that are pinned to the version that bonsol relies on.
          toolchain = pkgs.callPackage ./nixos/lib/toolchain.nix {
            inherit (nix-core.toolchains.${system}) mkRustToolchainFromTOML;
          };
          inherit (toolchain) flatc rustToolchain;
          inherit (toolchain.risc0) r0vm cargo-risczero risc0-groth16-prover isRisc0CircuitRecursion risc0CircuitRecursionPatch;
          inherit (toolchain.solana) solana-platform-tools solana-cli;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain.fenix-pkgs;

          workspace = rec {
            root = ./.;
            src = craneLib.cleanCargoSource root;
            mkCratePath = crate: root + "/${crate}";
          };

          # Patch git dependencies that require `risc0-circuit-recursion` for bonsol specifically.
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
            ] ++ [ flatc ];

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
            inherit (craneLib.crateNameFromCargoToml { inherit (workspace) src; }) version;
            doCheck = false;
          };

          # Function for including a set of files for a specific crate,
          # avoiding unnecessary files.
          fileSetForCrate = crate: lib.fileset.toSource {
            inherit (workspace) root;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./rust-toolchain.toml
              ./schemas
              ./schemas-rust
              ./iop
              ./cli
              ./sdk
              ./node
              ./onchain
              ./prover
              ./tester
              (workspace.mkCratePath crate)
            ];
          };

          # Build the top-level crates of the workspace as individual derivations.
          # This allows consumers to only depend on (and build) only what they need.
          # Though it is possible to build the entire workspace as a single derivation,
          # in this case the workspace itself is not a package.
          mkCrateDrv = name: crate:
            let
              manifest = craneLib.crateNameFromCargoToml {
                cargoToml = ((workspace.mkCratePath crate) + "/Cargo.toml");
              };
            in
            craneLib.buildPackage (individualCrateArgs // {
              inherit (manifest) pname;
              cargoExtraArgs = "--locked --bin ${name}";
              src = fileSetForCrate crate;
            });

          bonsol-cli = mkCrateDrv "bonsol" "cli";
          bonsol-node = mkCrateDrv "bonsol-node" "node";

          node_toml = pkgs.callPackage ./nixos/pkgs/bonsol/Node.toml.nix { inherit risc0-groth16-prover; };
          validator = pkgs.callPackage ./nixos/pkgs/bonsol/validator.nix { };
          run-node = pkgs.callPackage ./nixos/pkgs/bonsol/run-node.nix { inherit bonsol-node node_toml; };
        in
        {
          # NOTE: Some checks below fail due to generated code not being present
          # this is because `cargo build` isn't run on the source, just their
          # respective commands for checking formatting, docs, etc.
          checks = {
            # Build the crates as part of `nix flake check` for convenience
            inherit
              bonsol-cli
              bonsol-node;

            workspace-toml-fmt = craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices workspace.src [ ".toml" ];
            };

            # Audit licenses
            # TODO: Many problems still need to be addressed in the deny.toml
            workspace-deny = craneLib.cargoDeny {
              inherit (workspace) src;
            };
          };

          packages = {
            inherit
              bonsol-cli
              bonsol-node
              validator;
            run-node = (run-node.override {
              use-nix = true;
            });

            inherit
              flatc
              cargo-risczero
              r0vm
              risc0-groth16-prover
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
                risc0-groth16-prover
                solana-cli
                bonsol-cli
                bonsol-node
                validator
                (run-node.override {
                  use-nix = true;
                })
              ];

              text = ''
                ${bonsol-cli}/bin/bonsol --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 build -z images/simple
                echo "building validator"
                ${validator}/bin/validator.sh > /dev/null 2>&1 &
                validator_pid=$!
                sleep 30
                echo "validator is running: PID $validator_pid"
                echo "building node"
                ${run-node}/bin/run-node.sh > /dev/null 2>&1 &
                node_pid=$!
                sleep 30
                echo "node is running: PID $node_pid"
                ${bonsol-cli}/bin/bonsol --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 deploy url https://bonsol-public-images.s3.amazonaws.com/simple-68f4b0c5f9ce034aa60ceb264a18d6c410a3af68fafd931bcfd9ebe7c1e42960 -m images/simple/manifest.json -y
                sleep 20
                resp=$(${bonsol-cli}/bin/bonsol --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 execute -f testing-examples/example-execution-request.json -x 2000 -m 2000 -w)
                echo "execution response was: $resp"
                kill $validator_pid
                kill $node_pid
                if [[ "$resp" =~ "Success" ]]; then
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
              # TODO: Remove these once rustup toolchains are linked
              # cargo-hakari
              taplo
              cargo-deny
              cargo-audit
              cargo-nextest

              nil # nix lsp
              nixpkgs-fmt # nix formatter
              # TODO: use `rustup toolchain link` to link fenix toolchain to rustup as the override toolchain
              rustup

              # `setup.sh` dependencies
              docker
              corepack_22
              nodejs_22
              python3
              udev
            ] ++ [
              validator
              run-node

              r0vm
              cargo-risczero
              risc0-groth16-prover
              solana-cli
              flatc
            ];

            # Useful for debugging, sets the path that `cargo-build-sbf` will use to find `platform-tools`
            #
            # SBF_SDK_PATH = "${solana-cli}/bin/sdk/sbf"; # This is the default
          };

          # Run nix fmt to format nix files in file tree
          # using the specified formatter
          formatter = pkgs.nixpkgs-fmt;
        });
}
