# Bonsol CLI
This is a command line interface for bonsol. It allows you to execute a bonsol program both locally and through the prover network and deploy a bonsol program to the network.

## Installation
You must have cargo-binstall and cargo-risczero installed.

```
cargo install bonsol-cli --git https://github.com/anagrambuild/bonsol
```

## Usage
### Required arguments
* `-c` or `--config` : The path to the config file
* `-k` or `--keypair` : The path to the keypair file
* `-u` or `--rpc-url` : The url to the solana rpc

If you dont provide a keypair or rpc url or a config, the cli will use the default solana config file located in `~/.config/solana/`
example:
```
bonsol -k ./keypair.json -u http://localhost:8899 ...
```

### Build 
You can build a bonsol program with the following command

```
bonsol -k ./keypair.json -u http://localhost:8899 build -z {path to program folder}
```
In the above example the program folder is the folder that contains the Cargo.toml file. So if you have a program in the folder `my-program` you would run the command 
```bonsol -k ./keypair.json -u http://localhost:8899 build -z ./my-program```

The output of the build command is a manifest.json file which is placed in the root of the program folder. The manifest.json file contains needed information for deployment.
 
 The Cargo.toml file must have the following metadata
 ```
 [package.metadata.zkprogram]
 input_order = [...]
 ```
 The input_order is an array of strings that are the names of the inputs to the program. The options are `Public`, `Private`.
 For each input you expect in the program you must add an entry to the input_order array. This is used in deloyment to configure the order of the inputs.

### Deploy
You can deploy a bonsol program with the following command

```
bonsol -k ./keypair.json -u http://localhost:8899 deploy -m {path to manifest.json} -y {auto confirm} -t {s3|shadow-drive|url} ... {upload type specific options}

```
There will be many options for how to upload the program, the default is s3. Here is an example of how to deploy a program to s3
```
bonsol -k ./keypair.json -u http://localhost:8899 deploy -m program/manifest.json -t s3 --bucket bonsol-public-images --region us-east-1 --access-key {your key} --secret-key {your secret key}
```
In the above example the manifest.json file is the file that was created by the build command.
This will try to upload the binary to the s3 bucket and create a deployment account for the program. Programs are indexed by the image id, which is a kind of checksum of the program elf file. This means that if you change the elf file, the image id will change and the program will be deployed again under a new deployment account. Programs are immutable and can only be changed by redeploying the program. When a node downloads a program it will check the image id and if it doesnt match the deployment account it will reject the program. Furthermore when bonsol checks the proof, it will check the image id and if it doesnt match the deployment account and desired image id from execution request it will reject the proof.

### Execute
todo

### Prove
todo

### Input Sets
For convenience, json input sets can be created, read and updated from the bonsol cli.

Create a new input set with some arbitrary inputs, overwriting some previously existing inputs.
Creating a new input set without any inputs results in a file containing an empty inputs array.

```
bonsol -k ./keypair.json -u http://localhost:8899 input-set create --path <path/to/inputs.json> --input '{ "inputType": "PublicData", "data": "..." }' --input '{ "inputType": "Private", "data": "..." }' -t 
```

Read an input set, printing it to stdout.

```
bonsol -k ./keypair.json -u http://localhost:8899 input-set read -p <path/to/inputs.json>
```

Update an input set, appending inputs to the array of existing inputs.

```
bonsol -k ./keypair.json -u http://localhost:8899 input-set update --path <path/to/inputs.json> --input '{ "inputType": "PublicData", "data": "..." }' --input '{ "inputType": "Private", "data": "..." }'
```
