#![no_main]


use json::parse;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let data: String = env::read();
    let data = parse(&data).unwrap();
    let val = data["attestation"].as_str();
    if val.is_none() {
        env::commit(&0);
    }
    env::commit(&1);
}