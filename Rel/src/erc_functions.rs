use soroban_sdk::{Address, Env, Map, log};

pub fn owner_of(e: &Env, token_id: u32, owners: &Map<u32, Address>) -> Address {
    owners.get(token_id).expect("Address does not exist for given token id").clone()
}

pub fn exists(e: &Env, token_id: u32, owners: &Map<u32, Address>) -> bool {
    let address = owners.get(token_id);
    match address {
        Some(v) => {
            true
        },
        None => {
            false
        }
    }
}