#![no_std]

use soroban_sdk::{contracttype, symbol_short, unwrap::UnwrapOptimized, Env, String, Symbol};

const METADATA_KEY: Symbol = symbol_short!("METADATA");

#[derive(Clone)]
#[contracttype]
pub struct CustomTokenMetadata {
    pub decimal: u32,
    pub name: String,
    pub symbol: String,
    pub token_uri: String
}

#[derive(Clone)]
pub struct CustomTokenUtils(Env);

impl CustomTokenUtils {
    #[inline(always)]
    pub fn new(env: &Env) -> CustomTokenUtils {
        CustomTokenUtils(env.clone())
    }

    #[inline(always)]
    pub fn set_metadata(&self, metadata: &CustomTokenMetadata) {
        self.0.storage().persistent().set(&METADATA_KEY, metadata);
    }

    #[inline(always)]
    pub fn get_metadata(&self) -> CustomTokenMetadata {
        self.0
            .storage()
            .persistent()
            .get(&METADATA_KEY)
            .unwrap_optimized()
    }
}