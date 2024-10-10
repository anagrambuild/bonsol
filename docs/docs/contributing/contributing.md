# GeneralContributing Guide

Welcome! We're glad you're here. Before we get started, let's make sure everyone is using the same tools.

乃ㄖ几丂ㄖㄥ relies on multiple toolchains, and that can cause issues between developer environments. We recommend using Nix in order to alleviate the issue.
The nix flake in the root of the project provides a layer of safety by locking in the versions of packages provided by solana, risc0, rust, node
and any other tools that are necessary to build all of the goodness that is 乃ㄖ几丂ㄖㄥ.

## Getting Started

> NOTE: Currently only supported on _Non-NixOS_ `x86_64-linux` machines, this will eventually change to support other architectures including NixOS.
> Full NixOS sandbox compatibility will require patching the ELF for the prover since nix is unable to run dynamically linked executables.

Development in the sandbox provides reproducible build guarantees, which can be a huge timesaver and is great for maintaining sanity.
The 乃ㄖ几丂ㄖㄥ nix flake is as reproducible as it gets for building with solana and risc0, providing automatically patched ELFs and handles linking solana tools in a reproducible fashion
that avoids unecessary downloading, rebuilding and patching. With one command you can be up and running!

### Sandbox Prerequisites:
- multi-user `nix` with the `flakes` and `nix-command` features enabled. For this we recommend the [Determinate Nix Installer](https://zero-to-nix.com/start/install) which has these features enabled by default.

The following link will install nix with the above features and include the bonsol binary cache as a trusted substitutor. Without the substitutor many dependencies will build from source, which could take a the first time they are built!
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install --extra-conf "extra-substitutors = https://bonsol.cachix.org" --extra-conf "extra-trusted-public-keys = bonsol.cachix.org-1:yz7vi1rCPW1BpqoszdJvf08HZxQ/5gPTPxft4NnT74A="
```

- `docker` ([see why](https://nixos.wiki/wiki/Docker#Running_the_docker_daemon_from_nix-the-package-manager_-_not_NixOS))

> Note that upon installation, the current terminal does not have the nix executable on $PATH. Open a new terminal and verify the installation with `nix --version`.
> Double check that `cat /etc/nix/nix.conf` includes this line: `experimental-features = nix-command flakes`.

### Fork and Clone the Repo
```bash
git clone https://github.com/<your-fork>/bonsol.git
cd bonsol
```

### Sandbox Development Environment

```bash
# By default nix develop will enter a new bash shell with developer tools on $PATH.
# If you have a preferred shell, it can be passed as a command with the `-c` option.
nix develop -c zsh
```

This development environment overrides pre-existing global tools (excluding docker) with the ones provided by nix for this sub-shell instance.
Exiting the nix `devShell` with `exit` will place you back in the shell environment prior to entering the nix sub-shell.

With nix we can also run our CI checks locally, try it out with:
```bash
nix flake check
```

### Testing Your Environment

#### Build Tools

Solana has its own version of the `cargo` and `rustc` binaries that it uses under the hood to run `cargo-build-sbf` called `platform-tools`.
Let's be sure that we have all of the `cargo` plugins, that `cargo-build-sbf` knows where to find `platform-tools` and that we have the required rust toolchains:

> NOTE: after running `cargo build-sbf` for the first time you may also see `solana` as a toolchain when running `rustup toolchain list`.

```bash
rustup toolchain add nightly # needed for some rust formatting options
rustup toolchain list
# nightly-...
# ...
cargo --list
# risczero
# build-sbf
cargo build-sbf --help
# ...
# --sbf-sdk <PATH>
#   Path to the Solana SBF SDK [env: SBF_SDK_PATH=] [default:
#   /nix/store/zy86csn7rinyx964rv04ia7p0lwf93ak-solana-cli-1.18.22/bin/sdk/sbf]
taplo --version # TOML formatter
```

Some tools are necessary for running the docker container for the `groth16` prover. If this displays a nix store path, they are loaded correctly.
```bash
which pnpm
# /nix/store/zy86csn7rinyx964rv04ia7p0lwf93ak-corepack-nodejs-22.8.0/bin/pnpm
```

One last check that we have the correct versions of our build dependencies (These should be the same as the ones in `./Cargo.toml`).
If the versions do not match, please check that the path is to a nix store, eg. `which solana`.

```bash
solana --version
# solana-cli 1.18.22
r0vm --version
# risc0-r0vm 1.0.1
```

Great! Our tools are there. Let's run the `simple` example already provided from the README to make sure our environment is working properly.

1. Download and setup the system with the needed binaries and keys to run the groth16 prover over the risc0 FRI
```bash
./setup.sh
```
2. Compile the 乃ㄖ几丂ㄖㄥ on chain program and start a localnet with the program loaded
```bash
./validator.sh
```
3. On a separate terminal, compile the 乃ㄖ几丂ㄖㄥ off chain relay and starts it
```bash
./run-relay.sh
```
4. Build the image binary if it hasn't already been built, this will result in the binary's path being available in the `manifest.json` (in this case `images/simple/manifest.json`)
```bash
cargo run -p bonsol-cli build -z images/simple
```
5. Use the bonsol cli to deploy a zkprogram (here is a example already uploaded for you)
```bash
cargo run -p bonsol-cli deploy -m images/simple/manifest.json -t url --url https://bonsol-public-images.s3.amazonaws.com/simple-7cb4887749266c099ad1793e8a7d486a27ff1426d614ec0cc9ff50e686d17699
```
6. Use the bonsol cli to execute a zkprogram
```bash
cargo run -p bonsol-cli execute -f testing-examples/example-execution-request.json -x 2000 -m 2000 -w
```

## Pull Requests

Thank you for your hard work! It's well appreciated (: Most of our code quality standards can be kept up-to-code by running `nix flake check` and following the prompts.

Otherwise refer to this checklist:

- [] Before filing an issue, please check if an existing issue matches the description of the problem you are having or feature you'd like to see implemented.
- [] Please ensure when upstreaming pull requests that the problem you are solving has a corresponding GitHub issue, and that the issue is linked in the PR description with closing keywords, eg. `Closes #1578`.
- [] Add a clear and concise description of your changes in the PR description.
- [] Add relevant tests that showcase the effectiveness of your changes, where applicable.
- [] GitHub action CI will run the same checks in the nix sandbox that you can run locally on your machine with `nix flake check`, which may be helpful in diagnosing issues before pushing changes.
- [] Please format your rust code with the cargo nightly formatter as some options require it: `cargo +nightly fmt`, and format toml files with `taplo fmt`.
- [] When adding dependencies, please be cautious that the dependency is well maintained and does not create a security vulnerability. The flake checks will prevent this, and `cargo deny` can give other safe options to choose from.
- [] Ensure your PR is not introducing new lints: `cargo clippy`.

## Troubleshooting

- Docker Permissions: Adding your user to the `docker` group may help -- https://docs.docker.com/engine/install/linux-postinstall/

- Nix Permissions: Anything that would require root privileges, ie. dependencies that are installed globally only for root (docker), or commands that change ownership (lchown)
may require nix to also be elevated. If permission is denied or a dependency that would have been handled by nix (like pnpm) is not found, try `sudo su` then `nix develop`.

- Invalid Keypair: You need to generate a solana keypair, which you can check for at `~/.config/solana/id.json`. Generate a new keypair with `solana-keygen new` if it's missing.

- Keypair Not Found: If your keypair isn't automatically found, you can pass the keypair and RPC URL to the `bonsol-cli` invocation like so:

```bash
cargo run -p bonsol-cli -- --keypair $HOME/.config/solana/id.json --rpc-url http://localhost:8899 build -z images/simple
```

