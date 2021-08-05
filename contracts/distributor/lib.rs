// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

    /// deposit record.
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

    /// Event emitted when contract distribut the ALC to the user.
    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    /// Event emitted when contract distribut the ALC to the user.
    #[ink(event)]
    pub struct Distributed {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    /// Event emitted when contract finish to distribut all the ALC to the users.
    #[ink(event)]
    pub struct DistributedAccountCount {
        #[ink(topic)]
        count: u32,
    }

    impl Distributor {
        /// Create the new distributor with the ALC or aUsd's address.
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

        /// Ensure the caller is the operator of this contract.
        fn _only_operator(&self) {
            let sender = Self::env().caller();
            assert!(self.operator == sender, "Distributor: caller is not the operator");
        }

        /// Update or insert the deposti record.
        fn _upsert_deposit_record(&mut self, user:AccountId, amount:Balance) {
            let b = self.deposit_records.get(&user).copied().unwrap_or(0);
            let new_balance = b.checked_add(amount).expect("failed at _upsert_deposit_record the `distributor` contract");
            self.deposit_records.insert(user, new_balance);
        }

        /// Distribute the ALC to a user.
        fn _distribute_alc(&mut self, user:AccountId, amount:Balance) {
            let this = self.env().account_id();
            let balance: Balance = self.cash.balance_of(this);
            assert!(balance >= amount, "Distributor: _distribute_alc err");

            let ret:bool = self.cash.transfer(user, amount).is_ok();
            assert!(ret, "Distributor: _distribute_alc err");
            
            self.deposit_records.take(&user);    

            self.env().emit_event(Deposited {
                user: Some(user),
                amount,
            });
        }

        /// Get the operator's AccountId.
        #[ink(message)]
        pub fn operator(&self) -> AccountId {
            return self.operator;
        }

        /// Switch the operator who can call the function of this contract.
        #[ink(message)]
        pub fn transfer_operator(&mut self, new_operator:AccountId)  {
            self._only_operator();
            self.operator = new_operator;
        }

        /// Deposit the erc20 token just like aUsd.
        #[ink(message)]
        pub fn deposit_token(&mut self, amount:Balance) {
            let user:AccountId = self.env().caller();
            assert!(user != AccountId::from([0; 32]), "Distributor: distribute_alc err");
            assert!(amount > 0, "Distributor: deposit err");

            let balance: Balance = self.a_usd.balance_of(user);
            assert!(balance >= amount, "Distributor: deposit err");

            let this = self.env().account_id();
            let ret: bool = self.a_usd.transfer_from(user, this, amount).is_ok();
            assert!(ret, "Distributor: deposit err");

            self._upsert_deposit_record(user, amount);

            self.env().emit_event(Deposited {
                user: Some(user),
                amount,
            });
        }

        /// Deposit the coin of the chain.
        #[ink(message, payable)]
        pub fn deposit_coin(&mut self) {
            let caller = self.env().caller();
            assert!(caller != AccountId::from([0; 32]), "Distributor: distribute_alc err");

            let value = self.env().transferred_balance();
            assert!(value > 0, "Distributor: deposit coin err");

            self._upsert_deposit_record(caller, value);

            self.env().emit_event(Deposited {
                user: Some(caller),
                amount: value,
            });
        }

        /// Distribute the ALC with the deposit records.
        #[ink(message)]
        pub fn distribute_alc(&mut self, records:Vec<Record>) {
            self._only_operator();

            let a: usize = records.len();
            assert!(a > 0, "Distributor: distribute_alc err");

            for r in records {
                assert!(r.user != AccountId::from([0; 32]), "Distributor: distribute_alc err");
                assert!(r.amount > 0, "Distributor: distribute_alc err");

                self._distribute_alc(r.user, r.amount);
            }
        
            self.env().emit_event(DistributedAccountCount {
                count: a as u32,
            });
        }

        /// Get all the depositors.
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
}
