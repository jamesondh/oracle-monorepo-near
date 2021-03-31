use super::*;
use near_sdk::{Promise};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::Serialize;

/// Price per 1 byte of storage from mainnet config after `0.18` release and protocol version `42`.
/// It's 10 times lower than the genesis price.
pub const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountStorageBalance {
    total: U128,
    available: U128,
}

pub trait StorageManager {
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> AccountStorageBalance;

    fn storage_withdraw(&mut self, amount: U128) -> AccountStorageBalance;

    fn storage_minimum_balance(&self) -> U128;

    fn storage_balance_of(&self, account_id: ValidAccountId) -> AccountStorageBalance;
}

fn assert_one_yocto() {
    assert_eq!(
        env::attached_deposit(),
        1,
        "Requires attached deposit of exactly 1 yoctoNEAR"
    )
}

#[near_bindgen]
impl StorageManager for Contract {

    #[payable]
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> AccountStorageBalance {
        let amount = env::attached_deposit();
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());

        let mut balance = self.accounts.get(&account_id).unwrap_or(0);
        balance += amount;
        self.accounts.insert(&account_id, &balance);
        AccountStorageBalance {
            total: balance.into(),
            available: balance.into(),
        }
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: U128) -> AccountStorageBalance {
        assert_one_yocto();
        let amount: Balance = amount.into();
        assert_eq!(
            amount,
            self.storage_minimum_balance().0,
            "The withdrawal amount should be the exact storage minimum balance"
        );
        let account_id = env::predecessor_account_id();
        if let Some(balance) = self.accounts.remove(&account_id) {
            if balance > 0 {
                env::panic(b"The account has positive token balance");
            } else {
                Promise::new(account_id).transfer(amount + 1);
                AccountStorageBalance {
                    total: 0.into(),
                    available: 0.into(),
                }
            }
        } else {
            env::panic(b"The account is not registered");
        }
    }

    fn storage_minimum_balance(&self) -> U128 {
        U128(0)
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> AccountStorageBalance {
        if let Some(balance) = self.accounts.get(account_id.as_ref()) {
            AccountStorageBalance {
                total: self.storage_minimum_balance(),
                available: if balance > 0 {
                    0.into()
                } else {
                    self.storage_minimum_balance()
                },
            }
        } else {
            AccountStorageBalance {
                total: 0.into(),
                available: 0.into(),
            }
        }
    }
}