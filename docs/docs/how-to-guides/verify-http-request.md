# Verify an Http Request

Verifying an http request is a complicated but highly sought after feature. While bonsol supports public and private http inputs, its is currently not possible to verify that the prover pulled the correct data from the url, unless you have the `verify_input_hash` flag set to true in the execution request, and you provide the input hash.
This requires the requester of the execution request to provide the input hash, which is a keccak256 hash of the all of the input data. 

In highly dynamic cases, your inputs may change frequently and you may not be able to depend on the input hash to ensure the prover pulled the correct data. While our roadmap includes general Http input verification, we dont have a full solution for this yet. There is one reccomended way to do this with the current version of Bonsol, which is to use the `PublicProof` input type. In this case you will have used the Bonsol cli or Sdk to produce a proof, in this case the execution happens on a local or trusted machine relative to you the developer or the user. 

:::info
The term 'proof' here and risc0 receipts are used interchangeably.
:::

See the [Local Proving with the CLI](/docs/how-to/local-proving-cli) guide for more information on how to use the PublicProof input type.
And the [Proof Composition](/docs/how-to/proof-composition) guide for more information on how to proof composition.