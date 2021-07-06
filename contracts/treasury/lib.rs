#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod treasury {
    use ink_env::call::FromAccountId;
    use ink_storage::{
        Lazy,
    };
    use ink_env::debug_println;

    use oracle::Oracle;
    use asset::Asset;

    #[ink(storage)]
    pub struct Treasury {
        bond_cap: u128,
        decimal: u128,
        cash_price_one: u128,
        accumulated_seigniorage: u128,
        ceiling_price: u128,

        cash: Lazy<Asset>,
        bond: Lazy<Asset>,
        oracle:  Lazy<Oracle>,
    }

    #[ink(event)]
    pub struct RedeemedBonds {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    #[ink(event)]
    pub struct BoughtBonds {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    impl Treasury {
        #[ink(constructor)]
        pub fn new(cash_address:AccountId,
                   bond_address: AccountId,
                   oracle_address: AccountId,
                   decimal: u128) -> Self {

            let cash: Asset = FromAccountId::from_account_id(cash_address);
            let bond: Asset = FromAccountId::from_account_id(bond_address);
            let oracle: Oracle = FromAccountId::from_account_id(oracle_address);

            let r = decimal.checked_div(100).expect("");
            let r = r.checked_mul(5).expect("");
            let ar = decimal.checked_add(r).expect("");

            let instance = Self {
                bond_cap: 0,
                decimal,
                cash_price_one: decimal,
                accumulated_seigniorage: 0,
                ceiling_price: ar,

                cash: Lazy::new(cash),
                bond: Lazy::new(bond),
                oracle: Lazy::new(oracle),
            };
            instance
        }

        fn _cash_balance_of_this(&self) -> u128 {
            let this = self.env().account_id();
            let b: u128 = self.cash.balance_of(this);
            return b;
        }

        fn _min(&self, a: u128, b: u128) -> u128 {
            if a < b {
                return a;
            }

            return b;
        }

        fn _circulating_supply(&self) -> u128 {
            let cash_supply: u128 = self.cash.total_supply();
            let r = cash_supply.checked_sub(self.accumulated_seigniorage).expect("");
            return r;
        }

        fn _update_conversion_limit(&mut self, cash_price: u128) {
            let percentage = self.cash_price_one.checked_sub(cash_price).expect("");

            let cap = self._circulating_supply().checked_mul(percentage).expect("");
            let b_cap = cap.checked_div(self.decimal).expect("");
            let bond_supply: u128 = self.bond.total_supply();

            self.bond_cap = b_cap.checked_sub(self._min(b_cap, bond_supply)).expect("");
        }

        #[ink(message)]
        pub fn buy_bonds(&mut self, amount: u128, target_price: u128) {
            debug_println!("enter buy_bonds");
            assert!(amount > 0, "Treasure: cannot purchase bonds with zero amount");

            let cash_price:u128 = self.oracle.get_cash_price();

            ink_env::debug_println!("cash_price is: {}", cash_price);

            assert!(cash_price <= target_price, "Treasure: cash price moved");
            assert!(cash_price < self.cash_price_one, "Treasure: cash_price not eligible for bond purchase");

            debug_println!("cash_price is valid");

            self._update_conversion_limit(cash_price);

            debug_println!("cash_price is valid2");

            let mul_value = self.bond_cap.checked_mul(cash_price).expect("");
            let div_value = mul_value.checked_div(self.decimal).expect("");
            let amount = self._min(amount, div_value);

            ink_env::debug_println!("amount is: {}", amount);

            assert!(amount > 0, "Treasure: amount exceeds bond cap");
            debug_println!("amount > 0");

            let mul_value = amount.checked_mul(self.decimal).expect("");
            let div_value = mul_value.checked_div(cash_price).expect("");

            let sender = Self::env().caller();
            let burn_ret:bool = self.cash.burn_from(sender, amount).is_ok();
            assert!(burn_ret, "Treasure: transfer ok");

            let mint:bool = self.bond.mint(sender, div_value).is_ok();
            assert!(mint, "Treasure: mint ok");

            debug_println!("transfer is over");

            self.env().emit_event(BoughtBonds {
                from: Some(sender),
                amount,
            });

            debug_println!("leave buy_bonds");
        }

        #[ink(message)]
        pub fn redeem_bonds(&mut self, amount: u128) {
            debug_println!("enter redeem_bonds");
            assert!(amount > 0, "Treasure: cannot redeem bonds with zero amount");

            let cash_price:u128 = self.oracle.get_cash_price();
            assert!(cash_price > self.ceiling_price, "Treasure: cashPrice not eligible for bond purchase");

            debug_println!("cash_price > self.ceiling_price");

            let b: u128 = self._cash_balance_of_this();
            assert!(b >= amount, "Treasure: treasury has no more budget");

            debug_println!("b >= amount");

            let sub_value = self.accumulated_seigniorage.checked_sub(self._min(self.accumulated_seigniorage, amount)).expect("");
            self.accumulated_seigniorage = sub_value;

            let sender = Self::env().caller();

            let burn_ret: bool = self.bond.burn_from(sender, amount).is_ok();
            assert!(burn_ret, "Treasure: transfer ok");

            let trans_ret: bool = self.cash.transfer(sender, amount).is_ok();
            assert!(trans_ret, "Treasure: transfer ok");

            self.env().emit_event(RedeemedBonds {
                from: Some(sender),
                amount,
            });
            debug_println!("leave redeem_bonds");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
        }

        #[test]
        fn it_works() {
        }
    }
}
