#![no_std]

mod admin;
mod allowance;
mod balance;
mod contract;
mod event;
mod metadata;
mod storage_types;
mod test;
mod custom_token_metadata;
mod erc_functions;

pub use crate::contract::TokenClient;
