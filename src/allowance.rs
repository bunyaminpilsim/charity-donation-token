use crate::storage_types::AllowanceValue;
use soroban_sdk::{Address, Env};

pub fn read_allowance(e: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = (from.clone(), spender.clone());
    if let Some(allowance) = e
        .storage()
        .temporary()
        .get::<(Address, Address), AllowanceValue>(&key)
    {
        if allowance.expiration_ledger < e.ledger().sequence() {
            AllowanceValue {
                amount: 0,
                expiration_ledger: allowance.expiration_ledger,
            }
        } else {
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
}

pub fn write_allowance(
    e: &Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let allowance = AllowanceValue {
        amount,
        expiration_ledger,
    };

    if amount > 0 && expiration_ledger < e.ledger().sequence() {
        panic!("expiration_ledger is less than ledger seq when amount > 0")
    }

    let key = (from.clone(), spender.clone());
    e.storage().temporary().set(&key, &allowance);

    if amount > 0 {
        let live_for = expiration_ledger
            .checked_sub(e.ledger().sequence())
            .unwrap();
        e.storage().temporary().extend_ttl(&key, live_for, live_for)
    }
}

pub fn spend_allowance(e: &Env, from: Address, spender: Address, amount: i128) {
    let allowance = read_allowance(e, from.clone(), spender.clone());
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    write_allowance(
        e,
        from,
        spender,
        allowance.amount - amount,
        allowance.expiration_ledger,
    );
}
