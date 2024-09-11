# Bonsol Input Types
Bonsol has a variety of input types that can be used to pass data to the prover. These input types help developers deeply integrate with the web and solana.

## Public Inputs
Public inputs are inputs that are passed in the execution request. There are four types of public inputs.
* `PublicData` - A byte array that is passed in the execution request. 
* `PublicAccountData` - The pubkey of a solana account that is passed in the execution request. The prover will pull this account data from the solana blockchain and use it as a public input.
* `PublicUrl` - A url that the prover will pull data from and use as a public input.
* `PublicProof` - A proof and its output that the prover will use as a public input.

:::info
A note on pulling data from the solana blockchain or urls. Work is in process to make this more secure, currently the best way to ensure the prover is pulling the correct data is to configure the execution request to verify an input hash. This can be limiting if you are working with rapidly changing data. We hope to allow url input types to have a precompiled http verification circuit that can prove the origin of the data.
:::

## Private Inputs
Private inputs are inputs that are passed in the execution request. There is only one type of private input.

* `PrivateUrl` - A url that the prover will pull data from and use as a private input. This is a complicated one and caveats apply. Once a prover node has claimed the execution request, it must sign a request to the private input server to get the private input. The private input server will return the private input to the prover node. The input is no longer globally private so use this in scenarios where its okay if the prover node can see the input. We reccomend looking at Proof Composition through the `PublicProof` input type as an alternative to this.

## Input Sets

Input sets are used to pass multiple inputs to the prover. This is like a preconfigured set of inputs that a developer has made to simplify the process of passing inputs to the prover. It also can help with complex applications where immutability of most of the inputs is desired. For seasoned solana devs think of this as being like and account lookup table.


