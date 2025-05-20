use crate::storage_types::{BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

pub fn read_balance(e: &Env, addr: Address) -> i128 {
    if let Some(balance) = e.storage().persistent().get::<Address, i128>(&addr) {
        e.storage()
            .persistent()
            .extend_ttl(&addr, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        balance
    } else {
        0
    }
}

fn write_balance(e: &Env, addr: Address, amount: i128) {
    e.storage().persistent().set(&addr, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&addr, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn receive_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    write_balance(e, addr, balance + amount);
}

pub fn spend_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    if balance < amount {
        panic!("insufficient balance");
    }
    write_balance(e, addr, balance - amount);
}
