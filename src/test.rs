#![cfg(test)]
extern crate std;

use crate::{contract::Token, TokenClient};
use soroban_sdk::{
    log, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    Address, Env, IntoVal, String, Symbol, TryIntoVal, Val, Vec,
};
use soroban_token_sdk::metadata::TokenMetadata;
use std::string::ToString;

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &7, &"DonationToken".into_val(e), &"DNT".into_val(e));
    token
}

#[test]
fn test_donation_and_freeze() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let donor1 = Address::generate(&e);
    let recipient = Address::generate(&e);
    let token = create_token(&e, &admin);

    // Admin kontrolü
    assert!(token.has_administrator());
    assert_eq!(token.read_administrator(), admin);

    token.mint(&donor1, &1000);
    assert_eq!(token.balance(&donor1), 1000);

    token.transfer(&donor1, &recipient, &500);
    assert_eq!(token.balance(&donor1), 500);
    assert_eq!(token.balance(&recipient), 500);

    // Freeze account ve yetkilendirme
    token.freeze_account(&donor1);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&e, "freeze_account"),
                    (donor1.clone(),).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Freeze olayını doğrula
    let events = e.events().all();
    log!(&e, "Freeze events: {:?}", events);
    let freeze_event_found = events.iter().any(|event| {
        let (contract, topics, data): (Address, Vec<Val>, Val) = event;
        let topics_match = topics.len() == 3
            && topics
                .get(0)
                .map(|t| {
                    t.try_into_val(&e)
                        .map(|t_sym: Symbol| {
                            t_sym.to_string() == String::from_str(&e, "frz_acct").to_string()
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            && topics
                .get(1)
                .map(|t| {
                    t.try_into_val(&e)
                        .map(|t_addr: Address| t_addr == admin)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            && topics
                .get(2)
                .map(|t| {
                    t.try_into_val(&e)
                        .map(|t_addr: Address| t_addr == donor1)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
        log!(
            &e,
            "Checking freeze event: contract={:?}, topics={:?}, data={:?}, match={}",
            contract,
            topics,
            data,
            topics_match
        );
        contract == token.address && topics_match && data.is_void()
    });
    assert!(freeze_event_found, "No matching frz_acct event found");

    // Unfreeze account ve yetkilendirme
    token.unfreeze_account(&donor1);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&e, "unfreeze_account"),
                    (donor1.clone(),).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Unfreeze olayını doğrula
    let events = e.events().all();
    log!(&e, "Unfreeze events: {:?}", events);
    let unfreeze_event_found = events.iter().any(|event| {
        let (contract, topics, data): (Address, Vec<Val>, Val) = event;
        let topics_match = topics.len() == 3
            && topics
                .get(0)
                .map(|t| {
                    t.try_into_val(&e)
                        .map(|t_sym: Symbol| {
                            t_sym.to_string() == String::from_str(&e, "unfrz_acc").to_string()
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            && topics
                .get(1)
                .map(|t| {
                    t.try_into_val(&e)
                        .map(|t_addr: Address| t_addr == admin)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
            && topics
                .get(2)
                .map(|t| {
                    t.try_into_val(&e)
                        .map(|t_addr: Address| t_addr == donor1)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
        log!(
            &e,
            "Checking unfreeze event: contract={:?}, topics={:?}, data={:?}, match={}",
            contract,
            topics,
            data,
            topics_match
        );
        contract == token.address && topics_match && data.is_void()
    });
    assert!(unfreeze_event_found, "No matching unfrz_acc event found");

    token.transfer(&donor1, &recipient, &100);
    assert_eq!(token.balance(&donor1), 400);
    assert_eq!(token.balance(&recipient), 600);

    assert_eq!(token.get_donation_balance(&donor1), 400);
}

#[test]
#[should_panic(expected = "account is frozen and cannot transfer tokens")]
fn test_frozen_account_transfer() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let donor = Address::generate(&e);
    let recipient = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.mint(&donor, &1000);
    token.freeze_account(&donor);
    token.transfer(&donor, &recipient, &100);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn test_insufficient_balance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let donor = Address::generate(&e);
    let recipient = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.mint(&donor, &1000);
    token.transfer(&donor, &recipient, &1001);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_already_initialized() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.initialize(
        &admin,
        &7,
        &"DonationToken".into_val(&e),
        &"DNT".into_val(&e),
    );
}

#[test]
fn test_admin_operations() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let new_admin = Address::generate(&e);
    let token = create_token(&e, &admin);

    // Başlangıç admin kontrolü
    assert!(token.has_administrator());
    assert_eq!(token.read_administrator(), admin);

    // Yeni admin ata (doğrudan yazma)
    token.write_administrator(&new_admin);
    assert_eq!(token.read_administrator(), new_admin);

    // Admin değiştirme (set_admin)
    token.set_admin(&new_admin);
    assert_eq!(token.read_administrator(), new_admin);

    // Event'leri al ve kontrol et
    let events = e.events().all();
    log!(&e, "Admin events: {:?}", events);

    // Beklenen topic: [set_admin, new_admin]
    let expected_topic = Vec::<Val>::from_array(
        &e,
        [
            symbol_short!("set_admin").into_val(&e),
            new_admin.clone().into_val(&e),
        ],
    );

    let mut set_admin_event_found = false;
    for event in events.iter() {
        let (contract, topics, data): (Address, Vec<Val>, Val) = event;
        log!(
            &e,
            "Checking set_admin event: contract={:?}, topics={:?}, data={:?}, expected_topics={:?}",
            contract,
            topics,
            data,
            expected_topic
        );
        if contract == token.address
            && topics.len() == expected_topic.len()
            && topics.iter().zip(expected_topic.iter()).all(|(t, et)| {
                t.try_into_val(&e)
                    .map(|t_val: Symbol| {
                        et.try_into_val(&e)
                            .map(|et_val: Symbol| t_val == et_val)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
                    || t.try_into_val(&e)
                        .map(|t_val: Address| {
                            et.try_into_val(&e)
                                .map(|et_val: Address| t_val == et_val)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
            })
            && data
                .try_into_val(&e)
                .map(|addr: Address| addr == new_admin)
                .unwrap_or(false)
        {
            set_admin_event_found = true;
            break;
        }
    }
    assert!(set_admin_event_found, "No matching set_admin event found");
}

#[test]
fn test_allowance_operations() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let donor = Address::generate(&e);
    let spender = Address::generate(&e);
    let recipient = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.mint(&donor, &1000);
    assert_eq!(token.balance(&donor), 1000);

    // Allowance onayı
    let amount = 300_i128;
    let expiration_ledger = e.ledger().sequence() + 1000;
    token.approve(&donor, &spender, &amount, &expiration_ledger);
    assert_eq!(token.allowance(&donor, &spender), amount);

    // Approve olayını doğrula
    let events = e.events().all();
    log!(&e, "Allowance events: {:?}", events);
    assert!(events.iter().any(|event| {
        let (contract, topics, data): (Address, Vec<Val>, Val) = event;
        contract == token.address
            && topics
                == Vec::<Val>::from_array(
                    &e,
                    [
                        symbol_short!("approve").into_val(&e),
                        donor.clone().into_val(&e),
                        spender.clone().into_val(&e),
                    ],
                )
            && data
                .try_into_val(&e)
                .map(|vec: Vec<Val>| {
                    vec.get(0)
                        .and_then(|v| v.try_into_val(&e).ok())
                        .map(|amt: i128| amt == amount)
                        .unwrap_or(false)
                        && vec
                            .get(1)
                            .and_then(|v| v.try_into_val(&e).ok())
                            .map(|exp: u32| exp == expiration_ledger)
                            .unwrap_or(false)
                })
                .unwrap_or(false)
    }));

    // Allowance ile transfer
    token.transfer_from(&spender, &donor, &recipient, &200);
    assert_eq!(token.balance(&donor), 800);
    assert_eq!(token.balance(&recipient), 200);
    assert_eq!(token.allowance(&donor, &spender), 100);
}

#[test]
fn test_burn_operations() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let donor = Address::generate(&e);
    let spender = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.mint(&donor, &1000);
    assert_eq!(token.balance(&donor), 1000);

    // Doğrudan yakma
    token.burn(&donor, &300);
    assert_eq!(token.balance(&donor), 700);

    // Allowance ile yakma için onay
    let amount = 200_i128;
    let expiration_ledger = e.ledger().sequence() + 1000;
    token.approve(&donor, &spender, &amount, &expiration_ledger);

    // Allowance ile yakma
    token.burn_from(&spender, &donor, &150);
    assert_eq!(token.balance(&donor), 550);
    assert_eq!(token.allowance(&donor, &spender), 50);

    // Yakma olaylarını doğrula
    let events = e.events().all();
    log!(&e, "Burn events: {:?}", events);
    assert!(events.iter().any(|event| {
        let (contract, topics, data): (Address, Vec<Val>, Val) = event;
        contract == token.address
            && topics
                == Vec::<Val>::from_array(
                    &e,
                    [
                        symbol_short!("burn").into_val(&e),
                        donor.clone().into_val(&e),
                    ],
                )
            && data
                .try_into_val(&e)
                .map(|amt: i128| amt == 300)
                .unwrap_or(false)
    }));
    assert!(events.iter().any(|event| {
        let (contract, topics, data): (Address, Vec<Val>, Val) = event;
        contract == token.address
            && topics
                == Vec::<Val>::from_array(
                    &e,
                    [
                        symbol_short!("burn").into_val(&e),
                        donor.clone().into_val(&e),
                    ],
                )
            && data
                .try_into_val(&e)
                .map(|amt: i128| amt == 150)
                .unwrap_or(false)
    }));
}

#[test]
fn test_metadata() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let token = create_token(&e, &admin);

    // Metadata kontrolü
    assert_eq!(token.decimals(), 7);
    assert_eq!(token.name(), String::from_str(&e, "DonationToken"));
    assert_eq!(token.symbol(), String::from_str(&e, "DNT"));

    // Metadata güncelleme
    token.write_metadata(&TokenMetadata {
        decimal: 8,
        name: String::from_str(&e, "UpdatedToken"),
        symbol: String::from_str(&e, "UTK"),
    });
    assert_eq!(token.decimals(), 8);
    assert_eq!(token.name(), String::from_str(&e, "UpdatedToken"));
    assert_eq!(token.symbol(), String::from_str(&e, "UTK"));
}
