
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
