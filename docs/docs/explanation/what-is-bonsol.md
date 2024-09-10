# Bonsol: ZK Co-Processor for Solana

## Introduction

Bonsol (乃ㄖ几丂ㄖㄥ) is an innovative ZK "co-processor" designed for the Solana blockchain. It enables developers to run and verify complex computations off-chain while maintaining the security and trustlessness of the blockchain. This documentation will provide an overview of Bonsol, its underlying technologies, and how it integrates with Solana.

## What is Bonsol?

Bonsol acts as a bridge between Solana's on-chain capabilities and off-chain computational power. It allows developers to execute computationally intensive tasks off-chain and then verify the results on-chain, leveraging the power of verifiable computation. Using bonsol developers can lower their regulatory burden, build trust with their commpunity and simplify protocol design. This is in part because with verfiable computation, the computation can be proven to be correct with regard to how it was written. This is most useful when the code that describes the computation is available to the public.
The interseted parties can see that while the data being processed is private, the computation is correct. 
Bonsol is deeply integrated with Solana and can be used to build a variety of use cases. You can compose other programs on top of Bonsol to add verifiable compute to your protocol, or  to add verifiable layer on top of existing primitives.  Bonsol is built atop the excellent [RISC Zero](https://github.com/risc0/risc0) zkVM, which allows developers to write arbitrary programs and generate verifiable proofs of their execution, in some cases those proofs can be zero-knowledge with regard to the inputs. 

## A high level overview of how Bonsol works

1. Developers create zkprograms using RISC Zero Tooling
2. These zkprograms are registered with Bonsol
3. Users can request execution of these zkprograms through Bonsol
4. Provers run the zkprograms and generate STARK proofs
5. Bonsol wraps the STARK proof into a SNARK (Succinct Non-interactive ARgument of Knowledge)
6. The SNARK proof is verified natively on Solana

### RISC0 STARK Proofs

RISC Zero generates STARK proofs, which have several important properties:

1. **Scalability**: STARK proofs can handle arbitrarily large computations, with proof size and verification time growing logarithmically with the computation size.

2. **Transparency**: STARKs don't require a trusted setup, enhancing their security and reducing reliance on external parties.

3. **Variable Length**: The size of a STARK proof is directly related to the complexity and length of the computation being proved. This means that for simple computations, the proof can be quite small, while for more complex ones, it can grow larger.

4. **Post-Quantum Security**: STARKs are believed to be secure against attacks from quantum computers.

However, these proofs can become quite large for complex computations, which can be problematic for on-chain verification on Solana.

### STARK to SNARK Conversion

To address the potential size issues of STARK proofs, Bonsol converts them into Groth16 SNARKs. This process involves several steps:

1. **Proof Aggregation**: In the case of using Proofs as Inputs,Bonsol may first aggregate multiple proof segments into a single, more compact proof.

2. **Circuit Generation**: The STARK verification circuit is transformed into an arithmetic circuit suitable for SNARK proving.

3. **Trusted Setup**: A one-time trusted setup is performed for the Groth16 scheme. This setup is universal for all STARK to SNARK conversions in Bonsol.

4. **Proof Generation**: Using the Groth16 scheme, a new SNARK proof is generated that attests to the validity of the original STARK proof.

### Benefits of Groth16 SNARKs

The conversion to Groth16 SNARKs offers several advantages:

1. **Constant-size proofs**: Regardless of the complexity of the original computation, the Groth16 SNARK proof has a fixed, small size.

2. **Fast verification**: Groth16 proofs can be verified extremely quickly, which is crucial for on-chain verification.

3. **Efficient implementation**: The algebraic structure of Groth16 proofs allows for efficient implementation on Solana.

### Native Verification on Solana

Bonsol implements a native Groth16 verifier on Solana, allowing for:
- Efficient proof verification, the verification call happenes in less that 200k cu
- TH his means we can compose over other programs in the same transaction

### Input Digest Verification

To ensure the integrity of inputs, Bonsol:
1. Ensures that Zkprograms compute a digest (hash) of all inputs (public and private)
2. Commits this digest as part of the zkprogram execution
3. Verifies the digest on-chain during proof verification

This additional step prevents potential attacks where a malicious prover might try to use different inputs than those specified in the execution request.
