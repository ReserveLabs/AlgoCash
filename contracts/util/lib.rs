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

pub use self::util::Util;
use ink_lang as ink;

#[ink::contract]
mod util {

    #[ink(storage)]
    pub struct Util {
        decimal: u32,
        one_unit_with_decimal: u128,
    }

    impl Util {
        #[ink(constructor)]
        pub fn new(decimal:u32) -> Self {
            let u:u128 = 10;
            let ret:u128 = u.checked_pow(decimal).expect("failed at new the `util` contract");
            Self {
                decimal,
                one_unit_with_decimal:ret,
            }
        }

        #[ink(message)]
        pub fn get_decimal(&self) -> u32 {
            return self.decimal;
        }

        #[ink(message)]
        pub fn get_one_unit_with_decimal(&self) -> u128 {
            return self.one_unit_with_decimal;
        }

        #[ink(message)]
        pub fn get_ceiling_price(&self) -> u128 {
            let r = self.one_unit_with_decimal.checked_div(100).expect("failed at new the `getCeilingPrice` contract");
            let r = r.checked_mul(5).expect("failed at getCeilingPrice the `util` contract");
            let ar = self.one_unit_with_decimal.checked_add(r).expect("failed at getCeilingPrice the `util` contract");
            return ar
        }

        #[ink(message)]
        pub fn math_min(&self, a: u128, b: u128) -> u128 {
            if a < b {
                return a;
            }

            return b;
        }
    }
}
