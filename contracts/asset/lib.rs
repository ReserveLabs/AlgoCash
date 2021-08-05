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

pub use self::asset::Asset;
use ink_lang as ink;

#[ink::contract]
mod asset {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        lazy::Lazy,
    };

    use ink_prelude::string::String;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    pub struct Asset {
        /// Name of the token
        name: Option<String>,
        /// Symbol of the token
        symbol: Option<String>,
        /// Decimals of the token
        decimals: Option<u8>,
        /// Total token supply.
        total_supply: Lazy<Balance>,
        /// Mapping from owner to number of owned token.
        balances: StorageHashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// Owner of the contract
        operator: AccountId,
        /// from another account.
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        /// Retuured if the value is invalid
        InvalidValue,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Asset {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance,
                   name: Option<String>,
                   symbol: Option<String>,
                   decimals: Option<u8>,) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, initial_supply);
            let instance = Self {
                name,
                symbol,
                decimals,
                total_supply: Lazy::new(initial_supply),
                balances,
                allowances: StorageHashMap::new(),
                operator: caller,
            };
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
            instance
        }

        fn _only_operator(&self) {
            let sender = Self::env().caller();
            assert!(self.operator == sender, "Asset: caller is not the operator");
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

        /// Returns the name.
        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            self.name.clone()
        }

        /// Returns the symbol.
        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            self.symbol.clone()
        }

        /// Returns the decimals.
        #[ink(message)]
        pub fn decimals(&self) -> Option<u8> {
            self.decimals
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).copied().unwrap_or(0)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(from, to, value)?;
            let r = allowance.checked_sub(value).expect("failed at transferFrom the `asset` contract");
            self.allowances.insert((from, caller), r);
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }
            let r = from_balance.checked_sub(value).expect("failed at transferFromTo the `asset` contract");
            self.balances.insert(from, r);
            let to_balance = self.balance_of(to);
            let ar = to_balance.checked_add(value).expect("failed at transferFromTo the `asset` contract");
            self.balances.insert(to, ar);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }

        /// Mint `value` amount of tokens to account `to`.
        /// # Errors
        ///
        /// Returns `InvalidValue` error if balance increase failed
        #[ink(message)]
        pub fn mint(
            &mut self,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            self._only_operator();
            let balance_before = self.balance_of(to);
            let ar = balance_before.checked_add(value).expect("failed at mint the `asset` contract");
            self.balances.insert(to, ar);
            let balance_after = self.balance_of(to);
            if balance_after < balance_before {
                return Err(Error::InvalidValue)
            }
            let ts = self.total_supply();
            let ar = ts.checked_add(value).expect("failed at mint the `asset` contract");
            self.total_supply = Lazy::new(ar);
            self.env().emit_event(Transfer {
                from: Some(self.operator),
                to: Some(to),
                value,
            });
            Ok(())
        }

        /// Burn `value` amount of tokens from the caller's account.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn burn(
            &mut self,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let from_balance = self.balance_of(caller);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            let sr = from_balance.checked_sub(value).expect("failed at burn the `asset` contract");
            self.balances.insert(caller, sr);

            let ts = self.total_supply();
            let sr = ts.checked_sub(value).expect("failed at burn the `asset` contract");
            self.total_supply = Lazy::new(sr);
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(self.operator),
                value,
            });
            Ok(())
        }

        /// Burn `value` amount of tokens from the from's account.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the from's account balance.
        #[ink(message)]
        pub fn burn_from(
            &mut self,
            from: AccountId,
            value: Balance,
        ) -> Result<()> {
            self._only_operator();
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            let sr = allowance.checked_sub(value).expect("failed at burnFrom the `asset` contract");
            self.allowances.insert((from, caller), sr);

            let sr = from_balance.checked_sub(value).expect("failed at burnFrom the `asset` contract");
            self.balances.insert(from, sr);

            let ts = self.total_supply();
            let sr = ts.checked_sub(value).expect("failed at burnFrom the `asset` contract");
            self.total_supply = Lazy::new(sr);
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(self.operator),
                value,
            });
            Ok(())
        }
    }

    /// Unit tests.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;

        /// The default constructor does its job.
        #[ink::test]
        fn new_works() {
            // Constructor works.
            let erc20 = Asset::new(100, Some("test".to_string()), Some("test".to_string()), Some(10));
            assert!(erc20.decimals().unwrap_or(0) == 10);
            assert!(erc20.name().unwrap_or("".to_string()) == "test");
            assert!(erc20.symbol().unwrap_or("".to_string()) == "test");    
            assert!(erc20.total_supply() == 100); 
            
            let account = AccountId::from([0x01; 32]);
            assert_eq!(erc20.balance_of(account), 100);
        }

        #[ink::test]
        fn transfer_works() {
            // Constructor works.
            let mut erc20 = Asset::new(100, Some("test".to_string()), Some("test".to_string()), Some(10));
            
            // Transfer event triggered during initial construction.
            let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let account = AccountId::from([0x01; 32]);
            assert_eq!(erc20.transfer(accounts.bob, 10), Ok(()));
            assert_eq!(erc20.balance_of(accounts.bob), 10);
            assert_eq!(erc20.balance_of(account), 90);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            // Constructor works.
            let mut erc20 = Asset::new(100, Some("test".to_string()), Some("test".to_string()), Some(10));
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(erc20.balance_of(accounts.bob), 0);
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob fails to transfers 10 tokens to Eve.
            assert_eq!(
                erc20.transfer(accounts.eve, 10),
                Err(Error::InsufficientBalance)
            );
            
            let account = AccountId::from([0x01; 32]);
            assert_eq!(erc20.balance_of(account), 100);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.eve), 0);
        }

        #[ink::test]
        fn transfer_from_works() {
            // Constructor works.
            let mut erc20 = Asset::new(100, Some("test".to_string()), Some("test".to_string()), Some(10));
            
            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            // Bob fails to transfer tokens owned by Alice.
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10),
                Err(Error::InsufficientAllowance)
            );                
            
            assert_eq!(erc20.transfer(accounts.bob, 10), Ok(()));

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call.
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            let account = AccountId::from([0x01; 32]);

            // Alice approves Bob for token transfers on her behalf.
            assert_eq!(erc20.approve(account, 10), Ok(()));

            assert_eq!(erc20.balance_of(accounts.bob), 10);

            assert_eq!(erc20.allowance(accounts.bob, account), 10);

            
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                                    .unwrap_or([0x0; 32].into());
            
            // Create call.
            let mut data1 =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data1.push_arg(&account);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                account,
                callee,
                1000000,
                1000000,
                data1,
            );

            // Bob transfers tokens from Alice to Eve.
            assert_eq!(erc20.transfer_from(accounts.bob, accounts.eve, 10), Ok(()));

            // Eve owns tokens.
            assert_eq!(erc20.balance_of(accounts.eve), 10);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
        }
    }

    /// For calculating the event topic hash.
    struct PrefixedValue<'a, 'b, T> {
        pub prefix: &'a [u8],
        pub value: &'b T,
    }

    impl<X> scale::Encode for PrefixedValue<'_, '_, X>
        where
            X: scale::Encode,
    {
        #[inline]
        fn size_hint(&self) -> usize {
            self.prefix.size_hint() + self.value.size_hint()
        }

        #[inline]
        fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
            self.prefix.encode_to(dest);
            self.value.encode_to(dest);
        }
    }
}
