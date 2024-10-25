#![no_std]

use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{is_authorized, write_authorization};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::custom_token_metadata::CustomTokenMetadata;
use crate::erc_functions::{exists, owner_of};
use crate::event;
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::storage_types::{
    INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK, INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

pub trait TokenTrait {
    fn initialize(e: Env, admin: Address, token_id: u32);

    fn allowance(e: Env, from: Address, spender: Address) -> i128;

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);

    fn balance(e: Env, id: Address) -> i128;

    fn spendable_balance(e: Env, id: Address) -> i128;

    fn authorized(e: Env, id: Address) -> bool;

    fn transfer(e: Env, from: Address, to: Address, amount: i128);

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn burn(e: Env, from: Address, amount: i128);

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);

    fn clawback(e: Env, from: Address, amount: i128);

    fn set_authorized(e: Env, id: Address, authorize: bool);

    fn mint(e: Env, token_id: u32, to: Address);

    fn set_admin(e: Env, new_admin: Address);

    fn get_admin(e: Env) -> Address;

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> String;

    fn symbol(e: Env) -> String;

    fn get_owners(e: Env) -> Map<u32, Address>;

    fn set_owners(e: Env, token_id: u32, owner: Address);

    fn set_token_uri(e: Env, token_id: u32, token_uri: String);

    fn require_minted(e: Env, token_id: u32) -> bool;
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

const OWNERS: Symbol = symbol_short!("OWNERS");
const URIS: Symbol = symbol_short!("URIS");
const APPROVALS: Symbol = symbol_short!("approvals");
const OWNED_TOKEN_COUNT: Symbol = symbol_short!("tCount");
const OPERATOR_APPROVAL: Symbol = symbol_short!("opApprov");

#[contract]
pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn initialize(e: Env, admin: Address, token_id: u32) {
        if has_administrator(&e) {
            panic!("already initialized")
        }

        write_administrator(&e, &admin);

        let admin = read_administrator(&e);

        log!(&e, "Admin {}", admin);

        let mut owners: Map<u32, Address> =
            e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        owners.set(token_id, admin);
        e.storage().instance().set(&OWNERS, &owners);

        log!(&e, "Done Initializing");

        // if decimal > u8::MAX.into() {
        //     panic!("Decimal must fit in a u8");
        // }

        // write_metadata(
        //     &e,
        //     CustomTokenMetadata {
        //         decimal,
        //         name,
        //         symbol,
        //         token_uri
        //     },
        // )
    }

    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        event::approve(&e, from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
        read_balance(&e, id)
    }

    fn spendable_balance(e: Env, id: Address) -> i128 {
        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
        read_balance(&e, id)
    }

    fn authorized(e: Env, id: Address) -> bool {
        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
        is_authorized(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        event::transfer(&e, from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        event::transfer(&e, from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        spend_balance(&e, from.clone(), amount);
        event::burn(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        event::burn(&e, from, amount)
    }

    fn clawback(e: Env, from: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        spend_balance(&e, from.clone(), amount);
        event::clawback(&e, admin, from, amount);
    }

    fn set_authorized(e: Env, id: Address, authorize: bool) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        write_authorization(&e, id.clone(), authorize);
        event::set_authorized(&e, admin, id, authorize);
    }

    fn require_minted(e: Env, token_id: u32) -> bool {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        if exists(&e, token_id, &owners) == true {
            return true;
        }
        return false;
    }

    fn mint(e: Env, token_id: u32, to: Address) {
        // SOL: require(to != address(0), "ERC721: mint to the zero address");
        // CHECK IF ADDRESS IS NUL ADDRESS in soroban

        // New Token id should be incremented by 1 and not injected as param.

        let mut owners: Map<u32, Address> =
            e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        log!(&e, "Owners {}", owners);

        if exists(&e, token_id, &owners) == true {
            panic!("Token already minted!");
        }
        log!(&e, "Token does not exists {}", token_id);

        let cloned_to = to.clone();

        owners.set(token_id, to);
        log!(&e, "Owners set locally {}", owners);

        e.storage().instance().set(&OWNERS, &owners);
        log!(&e, "Owners set instance {}", owners);

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
        event::mint(&e, &cloned_to, token_id);
    }

    fn get_owners(e: Env) -> Map<u32, Address> {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
        log!(&e, "Owners {}", owners);
        owners
    }

    fn set_owners(e: Env, token_id: u32, owner: Address) {
        let mut owners: Map<u32, Address> =
            e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        owners.set(token_id, owner);
        e.storage().instance().set(&OWNERS, &owners);
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );

        write_administrator(&e, &new_admin);
        event::set_admin(&e, admin, new_admin);
    }

    fn get_admin(e: Env) -> Address {
        let admin = read_administrator(&e);
        admin
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }

    fn set_token_uri(e: Env, token_id: u32, token_uri: String) {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));

        if exists(&e, token_id, &owners) == false {
            panic!("ERC721URIStorage: URI set of nonexistent token");
        }

        let mut token_uris: Map<u32, String> =
            e.storage().instance().get(&URIS).unwrap_or(Map::new(&e));
        token_uris.set(token_id, token_uri);

        e.storage().instance().set(&URIS, &token_uris);
        e.storage().instance().bump(
            INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
            INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK,
        );
    }
}

//STEPS TO MINT:
// 1. Initialize with admin
//

// ------------> CURRENT CONTRACT ID = CCO42P3BKGQTZGVHWTK5EUS4R7OR7RHUUPH3PHQJKXLGJZP34INQJMIT --------------------

// soroban contract build --profile release-with-logs

// soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id b
// soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm \
//     --source nalnir \
//     --network standalone

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CCC7BOTWWO5LDI7Z2DOLGGSRWYHBG5IBIR7FI4F36JNS6DLBBZM2DNMN \
//     --source juico \
//     --network standalone \
//     -- \
//     initialize \
//     --admin GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4 \
//     --token_id 1111

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CAKDU2QQU7YYDYZWHRCRE23IJNANUACGMSIEAN74ATSUQIHU4AERSAYF \
//     --source juico \
//     --network standalone \
//     -- \
//     mint \
//     --token_id 4 \
//     --to GA3YIJVTHQIH3BXKQHUHAYHBZ7Z5NYPPWIXXT3OHVQO5YE3RKT5ASAFC

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CCYALRIXFRJXI4YRQCMN4YMNQZALNPAZLEGGJSAIP6I2CELT33L6WF4D \
//     --source juico \
//     --network standalone \
//     -- \
//     set_owners \
//     --token_id 4 \
//     --owner GA3YIJVTHQIH3BXKQHUHAYHBZ7Z5NYPPWIXXT3OHVQO5YE3RKT5ASAFC

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CAKDU2QQU7YYDYZWHRCRE23IJNANUACGMSIEAN74ATSUQIHU4AERSAYF \
//     --source juico \
//     --network standalone \
//     -- \
//     get_owners

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id b \
//     -- \
//     test

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     balance \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     initialize \
//     --admin GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4 \
//     --decimal 4 \
//     --name test \
//     --symbol TST \
//     --token_uri www.test.com

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     set_admin \
//     --new_admin GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     authorized \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     balance \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     set_authorized \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4
