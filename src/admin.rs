use crate::storage_types::ADMIN_KEY;
use soroban_sdk::{Address, Env};

pub fn has_administrator(e: &Env) -> bool {
    e.storage().instance().has(&ADMIN_KEY)
}

pub fn read_administrator(e: &Env) -> Address {
    e.storage().instance().get(&ADMIN_KEY).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    e.storage().instance().set(&ADMIN_KEY, id);
}
