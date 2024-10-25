use soroban_sdk::{Env, String};
use crate::custom_token_metadata::{CustomTokenMetadata, CustomTokenUtils};

pub fn read_decimal(e: &Env) -> u32 {
    let util = CustomTokenUtils::new(e);
    util.get_metadata().decimal
}

pub fn read_name(e: &Env) -> String {
    let util = CustomTokenUtils::new(e);
    util.get_metadata().name
}

pub fn read_symbol(e: &Env) -> String {
    let util = CustomTokenUtils::new(e);
    util.get_metadata().symbol
}

pub fn write_metadata(e: &Env, metadata: CustomTokenMetadata) {
    let util = CustomTokenUtils::new(e);
    util.set_metadata(&metadata);
}
