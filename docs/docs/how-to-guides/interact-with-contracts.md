# Interact with Bonsol Contracts/Programs

In your solana program you can interact with bonsol programs by using the `bonsol-interface` crate.

## Installing the crate

```bash
cargo add bonsol-interface
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
use bonsol_interface::{execution_address, deployment_address};

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
* The requester
* The image id
* The execution id
* The inputs
* The tip
* The expiry
* The execution config
* The callback config

Here is an example of how to create an execution request, taken from the PowPoW example.

```rust 
use anagram_channel_interface::instructions::{
    execute_v1, CallbackConfig, ExecutionConfig, Input,
};
...
execute_v1(
            ctx.accounts.miner.key, // requester
            MINE_IMAGE_ID, // image id
            &args.current_req_id, // execution id
            vec![ // inputs
                Input::public(pkbytes.to_vec()),
                Input::public(args.num.to_vec()),
            ],
            args.tip, // tip
            slot + 100, // expiry
            ExecutionConfig { // execution config
                verify_input_hash: true, // verify input hash - if true bonsol will ensure that the first output of the execution is the input hash
                input_hash: Some(input_hash.to_bytes().to_vec()), // input hash - if verify_input_hash is true this must be provided
                forward_output: true, // forward output - if true the output of the execution will be forwarded to the callback program
            },
            Some(CallbackConfig {
                program_id: crate::id(), // callback program id
                instruction_prefix: vec![0], // callback instruction prefix, this can be anything, but is used to allow the callback program to selecthe right instruction
                extra_accounts: vec![ // extra accounts to pass to the callback program, the prover will pass these accounts in the transaction so they can be used by the callback program
                    AccountMeta::new_readonly(ctx.accounts.pow_config.key(), false),
                    AccountMeta::new(ctx.accounts.pow_mint_log.key(), false),
                    AccountMeta::new(ctx.accounts.mint.key(), false),
                    AccountMeta::new(ctx.accounts.token_account.key(), false),
                    AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                ],
            }),
        )
```
Here is an example of how to verify a callback from bonsol, taken from the PowPoW example.
```rust
use anagram_bonsol_interface::callback::handle_callback;
...
pub fn your_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  data: &[u8],
) -> ProgramResult {
  ...
 let (prefix, data_stripped) = data.split_at(1); // this will depend on your program for anchor the ix prefix is 8 bytes and is called the discriminator for raw solana programs this could be anything.
 let epub = // some way of getting the execution account public key
 let output = handle_callback(
  epub,  // the execution account public key
  accounts, // the accounts that were passed in the transaction
  data_stripped, // the data that was passed in the transaction without the instruction prefix
 )?;

 //output can be used to do anything you want with the output of the execution, if the callback method errors then the transaction will still succeed in order to pay the prover but the 
 ...
```

## Using the crate in a anchor program
Bonsol has anchor integration and allows you to use anchor accounts structs with bonsol types.

```rust
use anagram_bonsol_interface::anchor::{
    Bonsol, DeployV1Account, ExecutionRequestV1Account,
};

#[derive(Accounts)]
#[instruction(args: MineTokenArgs)]
pub struct MineToken<'info> {
    #[account(
        seeds = [b"powconfig"],
        bump
    )]
    pub pow_config: Account<'info, PoWConfig>,
    ...
    pub bonsol_program: Program<'info, Bonsol>,
    pub execution_request: Account<'info, ExecutionRequestV1Account<'info>>,
    pub deployment_account: Account<'info, DeployV1Account<'info>>,
    pub system_program: Program<'info, System>,
    ...
}
```


