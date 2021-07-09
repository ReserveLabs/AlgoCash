#![cfg_attr(not(feature = "std"), no_std)]

pub use self::tokenstub::TokenStub;
use ink_lang as ink;

#[ink::contract]
mod tokenstub {
    use ink_prelude::string::String;

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

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct TokenStub {}

    impl TokenStub {
        #[ink(constructor)]
        pub fn new(
            _initial_supply: Balance,
            _name: Option<String>,
            _symbol: Option<String>,
            _decimals: Option<u8>,
        ) -> Self {
            unimplemented!()
        }

        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            unimplemented!()
        }

        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            unimplemented!()
        }

        #[ink(message)]
        pub fn decimals(&self) -> Option<u8> {
            unimplemented!()
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        pub fn balance_of(&self, _owner: AccountId) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        pub fn transfer(&mut self, _to: AccountId, _value: Balance) -> Result<()> {
            unimplemented!()
        }

        #[ink(message)]
        pub fn allowance(&self, _owner: AccountId, _spender: AccountId) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            _from: AccountId,
            _to: AccountId,
            _value: Balance,
        ) -> Result<()> {
            unimplemented!()
        }

        #[ink(message)]
        pub fn approve(&mut self, _spender: AccountId, _value: Balance) -> Result<()> {
            unimplemented!()
        }
    }
}
