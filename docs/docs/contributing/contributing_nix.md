# Using Nix to Contribute

### Sandbox Prerequisites:
- multi-user `nix` with the `flakes` and `nix-command` features enabled. For this we recommend the [Determinate Nix Installer](https://zero-to-nix.com/start/install) which has these features enabled by default.

The following link will install nix with the above features and include the bonsol binary cache as a trusted substitutor. Without the substitutor many dependencies will build from source, which could take a the first time they are built!
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install --extra-conf "extra-substituters = https://bonsol.cachix.org" --extra-conf "extra-trusted-public-keys = bonsol.cachix.org-1:yz7vi1rCPW1BpqoszdJvf08HZxQ/5gPTPxft4NnT74A="
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
