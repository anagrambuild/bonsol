# Interact with Bonsol Contracts/Programs

In your solana program you can interact with bonsol programs by using the `anagram-bonsol-channel-interface` crate.

## Installing the crate

```bash
cargo add anagram-bonsol-channel-interface
```

## Using the crate in a standard solana program
### Crafting the TXN
In a standard solana program you will need to craft a transaction with at least the following accounts:

* Deployment Account
* Execution Request Account
* Callback Program Account
  * Mostly this will be the same as the program id

You can can ensure you have the correct accounts by using the provided derivation functions.

In rust as long as you know the IMAGE_ID and EXECUTION_REQEUST_ID you can use the following derivation functions to ensure you have the correct accounts.

```rust
use anagram_bonsol_channel_interface::{execution_address, deployment_address};

const IMAGE_ID: &str = "image_id"; //this image id will be much longer and provided by the on chain record or in the manifest of a built zkprogram it will differ for each zkprogram

let execution_id = "execution_id"; //This is allowed to be any string as long as its unique. UUIDs are a good choice.
let requester = your.pubkey(); //this is the pubkey of the requester
let deployment_account = deployment_address(image_id);
let execution_request_account = execution_address(requester, execution_id);
```
In typescript you can use the following derivation functions.
```typescript
import { deploymentAddress, executionAddress } from "@bonsol/bonsol-node";

const IMAGE_ID = "image_id"; //this image id will be much longer and provided by the on chain record or in the manifest of a built zkprogram it will differ for each zkprogram
const EXECUTION_ID = "execution_id"; //This is allowed to be any string as long as its unique. UUIDs are a good choice.
const REQUESTER = new PublicKey("your.pubkey"); //this is the pubkey of the requester
const DEPLOYMENT_ACCOUNT = deploymentAddress(IMAGE_ID);
const EXECUTION_REQUEST_ACCOUNT = executionAddress(REQUESTER, EXECUTION_ID);
```


### Creating an Execuition Request

To create an execuition request you will need to provide the following information:

```rust
use anagram_bonsol_channel_interface::{
    macros::{execute...},
   
};


fn your_program_entrypoint(program_id: &Pubkey, accounts: &[AccountInfo], input: Vec<u8>) -> Result<()> {
...
let your_ix_data_oject = ...;
let execution_id = "execution_id"; //This is allowed to be any string as long as its unique. UUIDs are a good choice.
let inputs = vec![Input::PublicData(input)]; //this is the input you want to pass to the program


execute!( 
  accounts[0], //just an example the requester account, must be a signer,
  None, //no signer seeds, if you are havine a pda be the requester then you will need to provide the seeds
  accounts, //execute will locate the accounts you need to pass to the program
  IMAGE_ID, //the image id of the program you want to execute
  , //the execution id of the program you want to execute

)?;
...
}
```

## Using the crate in a anchor program





