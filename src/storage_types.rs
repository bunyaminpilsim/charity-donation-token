use soroban_sdk::{contracttype, symbol_short, Address, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

pub const INSTANCE_BUMP_AMOUNT: u32 = 5088;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 5088;
pub const BALANCE_BUMP_AMOUNT: u32 = 5088;
pub const BALANCE_LIFETIME_THRESHOLD: u32 = 5088;

pub const ADMIN_KEY: Symbol = symbol_short!("Admin");
pub const FROZEN_PREFIX: Symbol = symbol_short!("Frozen");
