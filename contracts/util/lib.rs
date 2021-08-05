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

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            // let util = Util::new(10);
            // assert_eq!(util.get_decimal(), 10);

            // assert_eq!(util.get_one_unit_with_decimal(), 10000000000);
            // assert_eq!(util.get_ceiling_price(), 10500000000);
        }
    }
}
