#![cfg_attr(not(feature = "std"), no_std)]

pub use self::oracle::Oracle;
use ink_lang as ink;

#[ink::contract]
mod oracle {

    #[ink(storage)]
    pub struct Oracle {
        cash_price: u128,
        last_update_time_stamp: u32,
    }

    impl Oracle {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                cash_price: 0,
                last_update_time_stamp: 0,
            }
        }

        #[ink(message)]
        pub fn get_cash_price(&self) -> u128 {
            self.cash_price
        }

        #[ink(message)]
        pub fn update_cash_price(&mut self, price: u128, ts: u32) {
            assert!(ts - self.last_update_time_stamp > 1, "invalid time stamp");
            self.cash_price = price;
            self.last_update_time_stamp = ts;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            let Oracle = Oracle::default();
        }

        #[test]
        fn it_works() {
            let mut Oracle = Oracle::new(false);
        }
    }
}
