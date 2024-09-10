import Prereq from '../shared/prereq.mdx';
import InitNewBonsol from '../shared/init-new-bonsol.mdx';
import Build from '../shared/build.mdx';
import Deploy from '../shared/deploy.mdx';

# CLI Commands
The bonsol cli is a command line interface for interacting with bonsol programs, building, and deploying zk programs.
<Prereq />

## Installation 
To install the bonsol cli you can use the following command.

```bash
cargo install bonsol-cli --git https://github.com/anagrambuild/bonsol
```

## Usage
### Required arguments
* `-c` or `--config` : The path to the config file
* `-k` or `--keypair` : The path to the keypair file
* `-u` or `--rpc-url` : The url to the solana rpc

If you dont provide a keypair or rpc url or a config, the cli will use  the default solana config file located in `~/.config/solana/`
example:
```
bonsol -k ./keypair.json -u http://localhost:8899 ...
```

<Build />

<Deploy />