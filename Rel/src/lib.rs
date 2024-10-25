#![no_std]

mod storage_types;
use crate::storage_types::{
    INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK, INSTANCE_BUMP_AMOUNT_LOW_WATERMARK,
};

mod erc_functions;
use crate::erc_functions::exists;

mod event;

mod admin;
use crate::admin::{has_administrator, read_administrator, write_administrator};

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, panic_with_error, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
};

// mod erc721 {
//     soroban_sdk::contractimport!(
//         file = "../token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
//     );
// }

#[contract]
pub struct PetalDocuments;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    TokenNotMinted = 1,
    DocumentSigningsIsEmpty = 2,
    NotASigner = 3,
    AlreadySigned = 4,
    SignerDoesNotExist = 5,
    DocumentHashesIsEmpty = 6,
    DocumentHashesDoesNotMatchTokenHash = 7,
    HashNotFound = 8,
    DeadlinesIsEmpty = 9,
    DeadlinePassed = 10,
    DeadlineNotFound = 11,
    SignatureExpired = 12,
    TokenAlreadyMinted = 13,
    TokenDoesNotExist = 14,
    SignersListEmpty = 15,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum SignatureStatus {
    NotASigner,
    Rejected,
    Signed,
    Waiting,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct SignedMessage {
    pub deadline: u64,
    pub description: String,
    pub document_hash: String,
    pub document_uri: String,
    pub signer: Address,
    pub status: SignatureStatus,
    pub token_id: u32,
    pub nonce: u32,
}

const OWNERS: Symbol = symbol_short!("OWNERS");
const URIS: Symbol = symbol_short!("URIS");

const NONCES: Symbol = symbol_short!("NONCES");
const T2DHASH: Symbol = symbol_short!("T2DHASH");
const DEADLINES: Symbol = symbol_short!("DEADLINES");
const DOCSIGN: Symbol = symbol_short!("DOCSIGN");
const CREACTION_FEE: Symbol = symbol_short!("crea_fee");

const TEST: Symbol = symbol_short!("TEST");

#[contractimpl]
impl PetalDocuments {
    pub fn init(e: Env, admin: Address, token_id: u32) {
        if has_administrator(&e) {
            panic!("already initialized")
        }

        write_administrator(&e, &admin);
    }

    pub fn sign_document(
        e: Env,
        document_hash: String,
        signer: Address,
        status: SignatureStatus,
        token_id: u32,
    ) -> Map<u32, Map<Address, SignatureStatus>> {
        // let client = erc721::Client::new(&e, &erc721_address);
        // let is_token_minted: bool = client.require_minted(&payload.token_id);
        let is_token_minted: bool = Self::require_minted(&e, token_id);
        if is_token_minted == false {
            panic_with_error!(&e, Error::TokenNotMinted)
        }
        let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e
            .storage()
            .persistent()
            .get(&DOCSIGN)
            .unwrap_or(Map::new(&e));
        if doc_signings.is_empty() {
            panic_with_error!(&e, Error::DocumentSigningsIsEmpty)
        }

        let clone_signer = signer.clone();
        let all_signings = doc_signings.get(token_id);
        let signer_status: SignatureStatus = match all_signings {
            Some(signing) => {
                let is_signer = signing.get(signer);
                match is_signer {
                    Some(signer) => {
                        if signer == SignatureStatus::NotASigner {
                            panic_with_error!(&e, Error::NotASigner)
                        } else if signer == SignatureStatus::Signed {
                            panic_with_error!(&e, Error::AlreadySigned)
                        }
                        signer
                    }
                    None => {
                        panic_with_error!(&e, Error::SignerDoesNotExist)
                    }
                }
            }
            None => {
                panic_with_error!(&e, Error::DocumentSigningsIsEmpty)
            }
        };

        let token_to_doc_hashes: Map<u32, String> = e
            .storage()
            .persistent()
            .get(&T2DHASH)
            .unwrap_or(Map::new(&e));
        if token_to_doc_hashes.is_empty() {
            panic_with_error!(&e, Error::DocumentHashesIsEmpty)
        }

        let doc_hash = token_to_doc_hashes.get(token_id);
        let matched_hash = match doc_hash {
            Some(hash) => {
                if (hash != document_hash) {
                    panic_with_error!(&e, Error::DocumentHashesDoesNotMatchTokenHash)
                }
                hash
            }
            None => {
                panic_with_error!(&e, Error::HashNotFound)
            }
        };

        let doc_signing_deadlines: Map<u32, u64> = e
            .storage()
            .persistent()
            .get(&DEADLINES)
            .unwrap_or(Map::new(&e));
        if doc_signing_deadlines.is_empty() {
            panic_with_error!(&e, Error::DeadlinesIsEmpty)
        }
        let deadlines: Option<u64> = doc_signing_deadlines.get(token_id);
        let deadline: u64 = match deadlines {
            Some(v) => {
                if e.ledger().timestamp() > v {
                    panic_with_error!(&e, Error::DeadlinePassed)
                }
                v
            }
            None => {
                panic_with_error!(&e, Error::DeadlineNotFound)
            }
        };

        let clone_signer_2 = clone_signer.clone();
        Self::verify_signer(&e, clone_signer, token_id);

        if e.ledger().timestamp() > deadline {
            panic_with_error!(&e, Error::SignatureExpired)
        };

        let clone_signer_3 = clone_signer_2.clone();
        let clone_signer_4 = clone_signer_3.clone();
        let mut signature_nonces: Map<Address, u32> = e
            .storage()
            .persistent()
            .get(&NONCES)
            .unwrap_or(Map::new(&e));
        let last_nonce = signature_nonces.get(clone_signer_4).unwrap_or(0);
        if signature_nonces.is_empty() {
            signature_nonces.set(clone_signer_2, last_nonce);
        } else {
            signature_nonces.set(clone_signer_2, last_nonce + 1);
        }
        let status_copy = status.clone();
        // doc_signings.get(token_id).unwrap().set(clone_signer_3, status);
        let mut inner_signings: Map<Address, SignatureStatus> = doc_signings.get(token_id).unwrap();
        // inner_signings.set(clone_signer_3, SignatureStatus::Signed);
        inner_signings.set(clone_signer_3, status);
        doc_signings.set(token_id, inner_signings);

        e.storage().persistent().set(&DOCSIGN, &doc_signings);
        // e.storage().persistent().bump(34560);

        doc_signings
    }

    fn verify_signer(e: &Env, signer: Address, token_id: u32) {
        signer.require_auth();

        let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e
            .storage()
            .persistent()
            .get(&DOCSIGN)
            .unwrap_or(Map::new(&e));
        let mut inner_doc_signings: Map<Address, SignatureStatus> =
            doc_signings.get(token_id).unwrap();
        let mut current_signature_status: SignatureStatus = inner_doc_signings.get(signer).unwrap();

        if (current_signature_status != SignatureStatus::Waiting) {
            panic_with_error!(&e, Error::AlreadySigned)
        }
    }

    pub fn safe_mint(
        e: Env,
        to: Address,
        token_id: u32,
        meta_uri: String,
        signers: Vec<Address>,
        document_hash: String,
        deadline: u64,
    ) -> u32 {
        // IMPLEMENT THIS LIKE IN SOLIDITY PETAL DOCUMENTS CONTRACT
        //		require(
        // 	msg.value >= creationFee || owner() == msg.sender,
        // 	'Creation fee not met'
        // );

        if signers.is_empty() {
            panic_with_error!(&e, Error::SignersListEmpty)
        }
        // let client = erc721::Client::new(&e, &erc721_address);
        // client.mint(&token_id, &to);
        // client.set_token_uri(&token_id, &meta_uri);

        Self::mint(&e, token_id, to);
        Self::set_token_uri(&e, token_id, meta_uri);

        let mut token_to_doc_hashes: Map<u32, String> = e
            .storage()
            .persistent()
            .get(&T2DHASH)
            .unwrap_or(Map::new(&e));
        token_to_doc_hashes.set(token_id, document_hash);

        let mut doc_signing_deadlines: Map<u32, u64> = e
            .storage()
            .persistent()
            .get(&DEADLINES)
            .unwrap_or(Map::new(&e));
        doc_signing_deadlines.set(token_id, deadline);

        let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e
            .storage()
            .persistent()
            .get(&DOCSIGN)
            .unwrap_or(Map::new(&e));
        let mut inner_doc_signings: Map<Address, SignatureStatus> = Map::new(&e);

        for signer in signers.iter() {
            inner_doc_signings.set(signer, SignatureStatus::Waiting);
        }
        doc_signings.set(token_id, inner_doc_signings);

        e.storage().persistent().set(&T2DHASH, &token_to_doc_hashes);
        e.storage()
            .persistent()
            .set(&DEADLINES, &doc_signing_deadlines);
        e.storage().persistent().set(&DOCSIGN, &doc_signings);

        // e.storage().persistent().bump(INSTANCE_BUMP_AMOUNT_LOW_WATERMARK, INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK);
        token_id
    }

    fn mint(e: &Env, token_id: u32, to: Address) {
        // New Token id should be incremented by 1 and not injected as param.

        let mut owners: Map<u32, Address> = e
            .storage()
            .persistent()
            .get(&OWNERS)
            .unwrap_or(Map::new(&e));
        if exists(&e, token_id, &owners) == true {
            panic_with_error!(&e, Error::TokenAlreadyMinted)
        }
        let cloned_to = to.clone();

        owners.set(token_id, to);
        log!(&e, "Owners set locally {}", owners);

        e.storage().persistent().set(&OWNERS, &owners);
        log!(&e, "Owners set instance {}", owners);

        // e.storage().persistent().bump(INSTANCE_BUMP_AMOUNT_LOW_WATERMARK, INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK);
        event::mint(&e, &cloned_to, token_id);
    }

    fn set_token_uri(e: &Env, token_id: u32, token_uri: String) {
        let owners: Map<u32, Address> = e
            .storage()
            .persistent()
            .get(&OWNERS)
            .unwrap_or(Map::new(&e));

        if exists(&e, token_id, &owners) == false {
            panic_with_error!(&e, Error::TokenDoesNotExist)
        }

        let mut token_uris: Map<u32, String> =
            e.storage().persistent().get(&URIS).unwrap_or(Map::new(&e));
        token_uris.set(token_id, token_uri);

        e.storage().persistent().set(&URIS, &token_uris);
        // e.storage().persistent().bump(INSTANCE_BUMP_AMOUNT_LOW_WATERMARK, INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK);
    }

    fn require_minted(e: &Env, token_id: u32) -> bool {
        let owners: Map<u32, Address> = e
            .storage()
            .persistent()
            .get(&OWNERS)
            .unwrap_or(Map::new(&e));
        if exists(&e, token_id, &owners) == true {
            return true;
        }
        return false;
    }

    pub fn set_test_int(e: Env) {
        let test_int: u32 = e.storage().persistent().get(&TEST).unwrap_or(0);
        let bump: u32 = test_int + 1;
        e.storage().persistent().set(&TEST, &bump);
    }

    pub fn get_test_int(e: Env) -> u32 {
        let test_int: u32 = e.storage().persistent().get(&TEST).unwrap_or(0);
        test_int
    }

    pub fn get_admin(e: Env) -> Address {
        let admin = read_administrator(&e);
        admin
    }

    pub fn get_nonces(e: Env, user: Address) -> u32 {
        let nonces: Map<Address, u32> = e
            .storage()
            .persistent()
            .get(&NONCES)
            .unwrap_or(Map::new(&e));
        let user_nonce = nonces.get(user).unwrap_or(0);
        // e.storage().persistent().bump(INSTANCE_BUMP_AMOUNT_LOW_WATERMARK, INSTANCE_BUMP_AMOUNT_HIGH_WATERMARK);
        user_nonce
    }

    pub fn get_owners(e: Env) -> Map<u32, Address> {
        let owners: Map<u32, Address> = e
            .storage()
            .persistent()
            .get(&OWNERS)
            .unwrap_or(Map::new(&e));
        owners
    }

    pub fn get_token_uris(e: Env) -> Map<u32, String> {
        let token_uris: Map<u32, String> =
            e.storage().persistent().get(&URIS).unwrap_or(Map::new(&e));
        token_uris
    }

    pub fn get_token_uri(e: Env, doc_id: u32) -> String {
        let token_uris: Map<u32, String> =
            e.storage().persistent().get(&URIS).unwrap_or(Map::new(&e));
        let token_uri = token_uris.get(doc_id).unwrap();
        token_uri
    }

    pub fn get_td_hashes(e: Env) -> Map<u32, String> {
        let token_to_doc_hashes: Map<u32, String> = e
            .storage()
            .persistent()
            .get(&T2DHASH)
            .unwrap_or(Map::new(&e));
        token_to_doc_hashes
    }

    pub fn get_deadlines(e: Env) -> Map<u32, u64> {
        let deadlines: Map<u32, u64> = e
            .storage()
            .persistent()
            .get(&DEADLINES)
            .unwrap_or(Map::new(&e));
        deadlines
    }

    pub fn get_documents(e: Env) -> Map<u32, Map<Address, SignatureStatus>> {
        let doc_signings: Map<u32, Map<Address, SignatureStatus>> = e
            .storage()
            .persistent()
            .get(&DOCSIGN)
            .unwrap_or(Map::new(&e));
        doc_signings
    }

    pub fn get_document(e: Env, doc_id: u32) -> Map<Address, SignatureStatus> {
        let doc_signings: Map<u32, Map<Address, SignatureStatus>> = e
            .storage()
            .persistent()
            .get(&DOCSIGN)
            .unwrap_or(Map::new(&e));
        let document = doc_signings.get(doc_id).unwrap_or(Map::new(&e));
        document
    }

    // pub fn add_extra_signers(e: Env, signers: Vec<Address>, doc_id: u32) {

    //     if signers.is_empty() {
    //         panic_with_error!(&e, Error::SignersListEmpty)
    //     }

    //     let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e
    //     .storage()
    //     .persistent()
    //     .get(&DOCSIGN)
    //     .unwrap_or(Map::new(&e));
    //     if doc_signings.is_empty() {
    //         panic_with_error!(&e, Error::DocumentSigningsIsEmpty)
    //     }

    //     let mut current_signers: Map<Address, SignatureStatus> = doc_signings.get(doc_id).unwrap_or(Map::new(&e));
    //     if current_signers.is_empty() {
    //         panic_with_error!(&e, Error::SignersListEmpty)
    //     }

    //     for (signer) in signers.iter() {
    //         let is_signer: SignatureStatus = current_signers.get(signer).unwrap_or_else(pa)

    //     }

    // }
}

// ------------> FUTURENET CONTRACT ID = CB6Y74MX2VRQ7C7ITKZM4SOAZOR7MQ3SX2QBJLXP63V43YCYNT46QKMG --------------------

// FUTURENET IDENTITY (juico) = GCA4YH7TOW2WUXPZ476I5EFKVLTQFMPXW7UG3GJ7BJTLXZAK226GTATI

// docker run --rm -it \
// -p 8000:8000 \
// --name stellar \
// stellar/quickstart:soroban-dev@sha256:ed57f7a7683e3568ae401f5c6e93341a9f77d8ad41191bf752944d7898981e0c \
// --futurenet \
// --enable-soroban-rpc

// soroban contract deploy \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --source juico \
// --network futurenet

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CCX7XGMKI6MERSDZXMUTHHWTPNKRBMFKSKR2ZA4DQ6WCAUSNZVZEVJZG \
//     --source juico \
//     --network futurenet \
//     -- \
//     get_document \
//     --doc_id 30

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CCTF7CMIEWTS6B2DLJL3QRBDD6JBRQCMXP6IBFTPQNR6K4DCSAIAPWGH \
//     --source juico \
//     --network futurenet \
//     -- \
//     get_documents

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CB7CEPSI2VGBKU63WGGOJ73EG2BEZJICYSGCPMGQK25DJKTBGR2GRV2N \
//     --source juico \
//     --network futurenet \
//     -- \
//     sign_document \
//         --user GB4ZLIQWAWNH3VKEFD2LXCYL4WYHYOGRG333457ZRYSANSQM3AFPCX7E \
//         --document_hash "hash1" \
//         --signer GBRVKHUULGOAU2ADSZZKFH2DZBZF2S4PXVEMSE23PPTFZDST464RDHIM \
//         --status "Signed" \
//         --token_id 1

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CB7CEPSI2VGBKU63WGGOJ73EG2BEZJICYSGCPMGQK25DJKTBGR2GRV2N \
//     --source juico \
//     --network futurenet \
//     -- \
//     safe_mint \
//     --to GDOB4GMX45VENP4YMUQMH4ZJ6KJZTERQVOASFTXC7OMOZM5EFKPFU4X5 \
//     --token_id 1 \
//     --meta_uri "test1" \
//     --signers '["GBRVKHUULGOAU2ADSZZKFH2DZBZF2S4PXVEMSE23PPTFZDST464RDHIM"]' \
//     --deadline 1691923773\
//     --document_hash "hash1"

// RUST_LOG=trace soroban contract restore --id CCPRKL4P77TK6ZCD2TI74ZKNCIAZFXG52QMPP4F7AV32U5NDBHRD6PRK --source juico --network futurenet --key-xdr AAAAFA==
// soroban contract restore --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm --source juico --network futurenet
// RUST_LOG=trace soroban contract invoke --id CCPRKL4P77TK6ZCD2TI74ZKNCIAZFXG52QMPP4F7AV32U5NDBHRD6PRK --source juico --network futurenet -- -h
// soroban contract fetch --id CCPRKL4P77TK6ZCD2TI74ZKNCIAZFXG52QMPP4F7AV32U5NDBHRD6PRK --network futurenet -o /tmp/swap.wasme
