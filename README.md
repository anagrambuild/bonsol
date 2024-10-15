# 乃ㄖ几丂ㄖㄥ
Bonsol is the Offchain compute framework to make everything possible on solana.

Interact with the docs at [Bonsol.sh](https://bonsol.sh)
## Roadmap
Stage 1: Dawn (current stage)
* Developer feedback
    * New features 
        * Interfaces
            * More Ingesters, Senders
            * More Input Types
        * Adding Integrations
            * Zktls,web proofs, client proving
    * Node Ops
        * Claim based prover network (SOL)
        * Prover Supply Integrations
* Community Building

## Local Development

### Prerequisites:
- docker
- probably a fast computer

### Rust 
You will need to install rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install 1.80.0
```

### Risc0 Toolchain
```bash
cargo install cargo-binstall
cargo binstall cargo-risczero
cargo risczero install
```

### ZK Snark deps
Run the setup script to install the zksnark deps and compile the zksnark prover
``` bash
./setup.sh
```

### Running a Local Environment 

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
cargo run -p bonsol-cli deploy -m images/simple/manifest.json -t url --url https://bonsol-public-images.s3.amazonaws.com/simple-68f4b0c5f9ce034aa60ceb264a18d6c410a3af68fafd931bcfd9ebe7c1e42960
```
6. Use the bonsol cli to execute a zkprogram
```bash
cargo run -p bonsol-cli execute -f testing-examples/example-execution-request.json -x 2000 -m 2000 -w
```

## Contributing
Please see our [Contributing Guide](https://bonsol.sh/docs/contributing) for details on how to get started building 乃ㄖ几丂ㄖㄥ.
