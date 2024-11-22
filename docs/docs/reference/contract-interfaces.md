# Contract Interfaces

Bonsol hosts various libraries to help you interact with the Bonsol network. When interacting with Bonsol from a Solana program you will need to use the `bonsol-interface` crate.

## Using the crate in an on-chain program
```toml
bonsol-interface = { git = "https://github.com/anagrambuild/bonsol", branch = "main" , features = ["on-chain"] }
```

## Using the crate in an anchor program
```toml
bonsol-interface = { git = "https://github.com/anagrambuild/bonsol", branch = "main", features = ["anchor"] }
```

## Using the crate outside of a solana program
Off chain software should use the `bonsol-sdk` crate, which uses the `bonsol-interface` crate internally with off chain dependencies.
Nevertheless here is how you can use the `bonsol-interface` crate outside of a solana program.
```toml
bonsol-interface = { git = "https://github.com/anagrambuild/bonsol", branch = "main" }
```

## Usage 
See the [Interact with Bonsol Contracts/Programs](/docs/how-to/interact-with-contracts) section for more information on how to interact with Bonsol programs.



