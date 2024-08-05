

use risc0_zkvm::{guest::{env, sha::Impl},sha::{Digest, Sha256}};

fn main() {
    let kp = Vec::new();
    env::read_slice(&mut kp);

    let ivmsg = Vec::new();
    env::read_slice(&mut ivmsg);
    let ivmsg = String::from_utf8(ivmsg).unwrap();
   
    //todo

    env::read_slice(&mut [0u8; 32]);
    let digest = Impl::hash_bytes(
        &[
            publici1.as_bytes(),
            privatei2.as_bytes(),
        ].concat(),
    );
    env::commit_slice(digest.as_bytes());
    env::commit_slice(&[res]);
}
