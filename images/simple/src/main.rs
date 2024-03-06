
use std::str::from_utf8;
use gjson::Kind;
use risc0_zkvm::guest::env;

fn main() {
    let mut data = Vec::new();
    env::read_slice(&mut data);
    let st = from_utf8(&data).unwrap();
    let valid = gjson::valid(st);
    if valid {
        let val = gjson::get(st, "attestation");
        if val.kind() == Kind::String {
            env::commit(&1);
            return;
        }
    }
    env::commit(&0);
}