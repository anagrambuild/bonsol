# 乃ㄖ几丂ㄖㄥ
Bonsol is the Offchain compute framework to make everything possible on solana.

Interact with the docs at [Bonsol.sh](https://bonsol.sh)
# Roadmap
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

### Prequisites:
docker
probbably a fast computer

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
2. Compile the 乃ㄖ几丂ㄖㄥ on chain program and starts a localnet with the program loaded
```bash
./validator.sh
```
3. On a separate terminal, compile the 乃ㄖ几丂ㄖㄥ off chain relay and starts it
```bash
./run-relay.sh
```
4. Use the bonsol cli to deploy a zkprogram(here is a example already uploaded for you)
```bash
cargo run -p bonsol-cli deploy -m images/simple/manifest.json -t url --url https://bonsol-public-images.s3.amazonaws.com/simple-7cb4887749266c099ad1793e8a7d486a27ff1426d614ec0cc9ff50e686d17699
```
5. Use the bonsol cli to execute a zkprogram
```bash
cargo run -p bonsol-cli execute -f testing-examples/example-execution-request.json -x 2000 -m 2000 -w
```