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

pub use self::oracle::Oracle;
use ink_lang as ink;

#[ink::contract]
mod oracle {

    #[ink(storage)]
    pub struct Oracle {
        operator: AccountId,
        cash_price: u128,
        last_update_time_stamp: u32,
    }

    impl Oracle {
        /// Create the new oracle which support the ALC's price.
        #[ink(constructor)]
        pub fn new() -> Self {
            let sender = Self::env().caller();
            Self {
                operator: sender,
                cash_price: 0,
                last_update_time_stamp: 0,
            }
        }

        /// Ensure the caller is operator.
        fn _only_operator(&self) {
            let sender = Self::env().caller();
            assert!(self.operator == sender, "Distributor: caller is not the operator");
        }

        /// Switch the operator who can call the function of this contract. 
        #[ink(message)]
        pub fn transfer_operator(&mut self, new_operator:AccountId)  {
            self._only_operator();
            self.operator = new_operator;
        }

        /// Get the operator.
        #[ink(message)]
        pub fn operator(&self) -> AccountId {
            return self.operator;
        }

        /// Get the ALC's price.
        #[ink(message)]
        pub fn get_cash_price(&self) -> u128 {
            self.cash_price
        }

        /// Update the ALC's price.
        #[ink(message)]
        pub fn update_cash_price(&mut self, price: u128, ts: u32) {
            self._only_operator();
            
            assert!(ts - self.last_update_time_stamp > 1, "invalid time stamp");
            self.cash_price = price;
            self.last_update_time_stamp = ts;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn update_and_get_cash_works() {
            let mut oracle = Oracle::new();
        
            assert_eq!(oracle.get_cash_price(), 0);
            oracle.update_cash_price(123, 123);
            assert_eq!(oracle.get_cash_price(), 123);
        }
    }
}
