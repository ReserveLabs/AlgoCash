#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod distributor {
    use ink_prelude::{
        vec::Vec,
    };

    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
        },
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };
    use ink_env::call::FromAccountId;
    use asset::Asset;
    use tokenstub::TokenStub;

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Record {
        pub user: AccountId,
        pub amount: Balance,
    }

    #[ink(storage)]
    pub struct Distributor {
        cash: Lazy<Asset>,
        a_usd: Lazy<TokenStub>,

        operator: AccountId,
        deposit_records: StorageHashMap<AccountId, Balance>,
    }

    impl Distributor {
        #[ink(constructor)]
        pub fn new( cash_address:AccountId,
                    a_usd_address: AccountId) -> Self {
            let cash: Asset = FromAccountId::from_account_id(cash_address);
            let a_usd: TokenStub = FromAccountId::from_account_id(a_usd_address);
            let sender = Self::env().caller();

            let instance = Self {
                cash: Lazy::new(cash),
                a_usd: Lazy::new(a_usd),
                operator: sender,
                deposit_records: StorageHashMap::new(),
            };
            instance
        }

        fn _only_operator(&self) {
            let sender = Self::env().caller();
            assert!(self.operator == sender, "Distributor: caller is not the operator");
        }

        fn _upsert_deposit_record(&mut self, user:AccountId, amount:Balance) {
            let b = self.deposit_records.get(&user).copied().unwrap_or(0);
            let new_balance = b.checked_add(amount).expect("");
            self.deposit_records.insert(user, new_balance);
        }

        fn _distribute_alc(&mut self, user:AccountId, amount:Balance) {
            let this = self.env().account_id();
            let balance: u128 = self.cash.balance_of(this);
            assert!(balance >= amount, "Distributor: _distribute_alc err");

            let ret:bool = self.cash.transfer(user, amount).is_ok();
            assert!(ret, "Distributor: _distribute_alc err");
        }

        #[ink(message)]
        pub fn operator(&self) -> AccountId {
            return self.operator;
        }

        #[ink(message)]
        pub fn transfer_operator(&mut self, new_operator:AccountId)  {
            self._only_operator();
            self.operator = new_operator;
        }

        #[ink(message)]
        pub fn deposit(&mut self, user:AccountId, amount:Balance) {
            assert!(user != AccountId::from([0; 32]), "Distributor: deposit err");
            assert!(amount > 0, "Distributor: deposit err");

            let balance: u128 = self.a_usd.balance_of(user);
            assert!(balance >= amount, "Distributor: deposit err");

            let this = self.env().account_id();
            let ret: bool = self.a_usd.transfer_from(user, this, amount).is_ok();
            assert!(!ret, "Distributor: deposit err");

            self._upsert_deposit_record(user, amount);
        }

        #[ink(message)]
        pub fn distribute_alc(&mut self, records:Vec<Record>) {
            self._only_operator();

            for r in records {
                assert!(r.user != AccountId::from([0; 32]), "Distributor: distribute_alc err");
                assert!(r.amount > 0, "Distributor: distribute_alc err");

                self._distribute_alc(r.user, r.amount);
            }
        }

        #[ink(message)]
        pub fn get_all_depositor(&self) -> Vec<Record> {
            let mut records:Vec<Record> = Vec::new();
            for (key, value) in self.deposit_records.iter() {
                let r = Record {
                    user: *key,
                    amount: *value,
                };
                records.push(r);
            }
            return records;
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
        }

        #[ink::test]
        fn it_works() {
        }
    }
}
