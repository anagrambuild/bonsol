# 乃ㄖ几丂ㄖㄥ-ZK Plumbing on solana
![alt text](image.png)
As you can see from the above copy pasted AI image 乃ㄖ几丂ㄖㄥ is the hearts desire of Solana silly dragon developers. It is a ZK "co processor" (fancy word for a server that listens to a blochain and does stuff the chain cant do and then puts the results of those heavy lift operations on said blockchain).

乃ㄖ几丂ㄖㄥ currently allows you to run any risc0 image (program thingy, we will call this a zkprogram from now on) and verify it on solana. This works by allowing a user to create and `ExecutionRequest` with an image id, public inputs, any program callbacks(other solana programs can be called upon a valid proof) and a tip. A prover will act fast to `Claim` and will make a commitment produce a proof and submit the proof by the slot deadline with the `Status` . The 乃ㄖ几丂ㄖㄥ on chain solana program will then verify the proof and if it is valid will pay the prover the tip minus a small fee for the zkprogram creator and some "hand waive" auction or time based mechanic to incentivise provers to provide quality service. If the prover doesnt produce a proof in time, their claim will be invalidated and another prover can claim the execution request and try to produce a proof. Up until the slot deadline. 
```mermaid
sequenceDiagram
    participant User
    participant Solana
    participant Prover1
    participant Prover1
    participant PrivateInputServer


    User->>Solana: Create ExecutionRequest
    note right of User: Includes image ID, public inputs, callbacks, and tip

    Solana->>Prover1: Notify of ExecutionRequest
    Prover1-->>Solana: Claim ExecutionRequest at slot x
    Solana->>Prover2: Notify of ExecutionRequest
    Prover2-->>Solana: Claim ExecutionRequest at slot x+3

    Prover1->>PrivateInputServer: Request Private Inputs with claim proof
    PrivateInputServer-->>Prover1: Send Private Inputs
    Prover1->>Prover1: Generate proof
    Prover1->>Solana: Submit proof

    Prover2->>PrivateInputServer: Request Private Inputs with claim proof
    PrivateInputServer-->>Prover2: Send Private Inputs

    Prover2->>Prover2: Generate proof
    alt Proof is valid and submitted before the slot deadline
        Solana->>Prover1: Pay tip minus fees
        Solana->>Solana: Distribute fees to zkprogram creator
    else Proof is invalid or not submitted in time
        Solana->>Solana: Invalidate claim
        
        
        Prover2->>Prover2: Generate proof
        Prover2->>Solana: Submit proof (Status)
        alt Proof is valid and submitted before the slot deadline
            Solana->>Prover1: Pay tip minus fees
            Solana->>Solana: Distribute fees to zkprogram creator
        end
    end
```


#### Claim???
Wait dude why does the prover have to claim before they can start to prove? Well I have a simple answer but you wont like it. Because in the future I want to allow the private inputs to be encrypted so that only the claim winner can have access to the private inputs and I literally dont even know where to start with Homomorphic Encryption inside a ZKvm. FHE is not some silver bullet okay .... The claim mechaism allows "fair" access to the tip. Claims are recorded in order of their slot and multiple provers can have claims. The prover that produces a valid proof within their promised claim window will be paid the tip. The private input server MUST only allow the provers that have submitted a claim the execution request to access the private inputs. They do this by making sure the private input request is signed by the same Key that submitted the claim.

#### "hand waive" auction or time based mechanic to incentivise provers to provide quality service
This is a sarcastic way of saying that im still figuring this out but I think it will be something like.

Users tip decreases in value from the time a prover claims it. At the slot the prover claims the execution request the tip is at its highest value. And it decreases in value until the slot deadline.

Not producing a proof will result in a loss of claim and a different prover can submit. If the prover submits a proof that is invalid they get some of their stake for that execution slashed. 

I would love feedback on if this is a good idea or not. I think it will incentivise provers to act fast and produce a proof quickly and the proof stake slashing will incentivise provers to produce quality proofs.

### Public Inputs
Public inputs come in a few forms. 乃ㄖ几丂ㄖㄥ will allow the following types of public inputs.
* `PUBLIC_DATA` - A byte array that is passed in the execution request. 
* `PUBLIC_ACCOUNT` - The data contents of a solana account that is passed in the execution request. The prover will pull this account data from the solana blockchain and use it as a public input.
* `PUBLIC_URL` - A url that the prover will pull data from and use as a public input. (this is for large inputs that are too large to fit in a solana account. make sure you tip your provers people as they will check the size of the data they need to pull and if the tip is not worth the time and resources they will not claim the execution request. Its important the server hosting the data returns the `content-length` header so the prover can check the size of the data before they claim the execution request.

### Private Inputs
If the zkprogram has private inputs then simultaneosly the user will send private inputs to a server hosted by the zkprogram creator. I know what your thinking!! Why would I trust the zkprogram creator with my private inputs? Well until I get time to implement a MPC encryption scheme in 乃ㄖ几丂ㄖㄥ that allows private inputs to be handled without another third party your locus of trust is the zkprogram creator. After all you are trusting that they wrote the program you want. If you dont trust them then dont use their program. 

In the most likley scenario, you as the developer will be the zkprogram creator and you will be the one hosting the server that handles the private inputs. The server that handles the private inputs is extremly simple and require nothing more than function as a service level compute. Meanwhile the provers are doing the heavy lifting and will compete on speed to claim and produce the proof. The prover servers will likley be "chonky bois" with "blazing fast" gpus and internet connections.

## On writing 乃ㄖ几丂ㄖㄥ flavored zkprograms
In risc0 there are the concepts of `guest` and `host`. From the guest perspective it has no notion of private/public inputs. It is simply a program that takes some inputs and produces some outputs. The host is the program that will call the guest and provide the inputs and recieve the outputs. The host is the 乃ㄖ几丂ㄖㄥ relayer and in order for it to properly secure the zk execution its of the upmost importance that the zkprogram commit to a Digest (the akkkkkktually correct term for a hash) of all the inputs. 
This is done like this

```rust

use gjson::Kind;
use risc0_zkvm::{guest::{env, sha::Impl},sha::{Digest, Sha256}};
//Its a good Idea to make a Type for your inputs
type Input = (String, String, String);
//The output type must be a tuple with the first element being the digest of the inputs
type Output = (Digest, bool);
fn main() {
    // The Bonsol Host will send your inputs in a tuple in the order they were declared on chain
    let (
        publici1,  //json
        publici2,  //path
        privatei2, //value
    ): Input = env::read();
    let valid = gjson::valid(&publici1);
    let mut res = false;
    if valid {
        let val = gjson::get(&publici1, &publici2);
        if val.kind() == Kind::String && val.str() == privatei2 {
            res = true;
        }
    }
    let digest = Impl::hash_bytes(
        &[
            publici1.as_bytes(),
            publici2.as_bytes(),
            privatei2.as_bytes(),
        ].concat(),
    );
    //Type argument here is optional but hepls enforce the type of the output
    env::commit::<Output>(&(*digest, res));
}

```
Specifically 

```rust

use risc0_zkvm::{guest::{env, sha::Impl},sha::{Digest, Sha256}};
...
let digest = Impl::hash_bytes(
        &[
            publici1.as_bytes(),
            publici2.as_bytes(),
            privatei2.as_bytes(),
        ].concat(),
    );
//commit as the first output, this will be in the journal
env::commit::<Output>(&(*digest, ...));

```

## Development Status
* Proving and verifying a proof: works
* Enforcing input digest as output of circit: in progress
* Execution Request: works
* Claiming: in progress
* Private Inputs: in progress
* Fee distribution: basic


## Local Development
You will need to run a few components.

1. Solana validator localnet
2. 乃ㄖ几丂ㄖㄥ on chain program
3. 乃ㄖ几丂ㄖㄥ off chain relay
4. 乃ㄖ几丂ㄖㄥ zkprogram compiler 
5. (Soon) Private input server

```bash
#Compiles the 乃ㄖ几丂ㄖㄥ on chain program and starts a localnet with the program loaded
./validator.sh 

#Compiles the 乃ㄖ几丂ㄖㄥ off chain relay and starts it
./run-relay.sh

#Compiles any local 乃ㄖ几丂ㄖㄥ zkprograms
./build-images.sh 
```

Note, currently you need docker installed and the Risc0 zkey, Verification key and .dat file present at the root of the relay directory.  
See https://github.com/risc0/risc0/tree/e69db8038011dc2e4020d3899cc1c7b40dd3d637/compact_proof for instructions.
This will change once the all rust prover is working, see relay/risc0 file (the commented out blobs)
