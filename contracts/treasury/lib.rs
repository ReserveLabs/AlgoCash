#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod treasury {
    use ink_env::call::FromAccountId;
    use ink_storage::{
        Lazy,
    };
    use ink_env::debug_println;

    use util::Util;
    use oracle::Oracle;
    use asset::Asset;
    use boardroom::Boardroom;

    #[ink(storage)]
    pub struct Treasury {
        bond_cap: u128,
        accumulated_seigniorage: u128,

        util:  Lazy<Util>,
        oracle:  Lazy<Oracle>,
        cash: Lazy<Asset>,
        bond: Lazy<Asset>,
        boardroom:  Lazy<Boardroom>,
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
        pub fn new(util_address:AccountId,
                   cash_address:AccountId,
                   bond_address: AccountId,
                   oracle_address: AccountId,
                   boardroom_address: AccountId) -> Self {

            let util: Util = FromAccountId::from_account_id(util_address);
            let cash: Asset = FromAccountId::from_account_id(cash_address);
            let bond: Asset = FromAccountId::from_account_id(bond_address);
            let oracle: Oracle = FromAccountId::from_account_id(oracle_address);
            let boardroom: Boardroom = FromAccountId::from_account_id(boardroom_address);

            let instance = Self {
                bond_cap: 0,
                accumulated_seigniorage: 0,

                util: Lazy::new(util),
                cash: Lazy::new(cash),
                bond: Lazy::new(bond),
                oracle: Lazy::new(oracle),
                boardroom: Lazy::new(boardroom),
            };
            instance
        }

        fn _cash_balance_of_this(&self) -> u128 {
            let this = self.env().account_id();
            let b: u128 = self.cash.balance_of(this);
            return b;
        }

        fn _circulating_supply(&self) -> u128 {
            let cash_supply: u128 = self.cash.total_supply();
            let r = cash_supply.checked_sub(self.accumulated_seigniorage).expect("");
            return r;
        }

        fn _update_conversion_limit(&mut self, cash_price: u128) {
            let cash_price_one = self.util.get_one_unit_with_decimal();
            let percentage = cash_price_one.checked_sub(cash_price).expect("");

            let cap = self._circulating_supply().checked_mul(percentage).expect("");

            let decimal = self.util.get_decimal();
            let b_cap = cap.checked_div(decimal.into()).expect("");

            let bond_supply: u128 = self.bond.total_supply();

            self.bond_cap = b_cap.checked_sub(self.util.math_min(b_cap, bond_supply)).expect("");
        }

        #[ink(message)]
        pub fn buy_bonds(&mut self, amount: u128, target_price: u128) {
            debug_println!("enter buy_bonds");
            assert!(amount > 0, "Treasure: cannot purchase bonds with zero amount");

            let cash_price:u128 = self.oracle.get_cash_price();

            ink_env::debug_println!("cash_price is: {}", cash_price);

            assert!(cash_price <= target_price, "Treasure: cash price moved");

            let cash_price_one = self.util.get_one_unit_with_decimal();
            assert!(cash_price < cash_price_one, "Treasure: cash_price not eligible for bond purchase");

            debug_println!("cash_price is valid");

            self._update_conversion_limit(cash_price);

            debug_println!("cash_price is valid2");

            let mul_value = self.bond_cap.checked_mul(cash_price).expect("");

            let decimal = self.util.get_decimal();
            let div_value = mul_value.checked_div(decimal.into()).expect("");
            let amount = self.util.math_min(amount, div_value);

            ink_env::debug_println!("amount is: {}", amount);

            assert!(amount > 0, "Treasure: amount exceeds bond cap");
            debug_println!("amount > 0");

            let mul_value = amount.checked_mul(decimal.into()).expect("");
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
            let ceiling_price:u128 = self.util.get_ceiling_price();
            assert!(cash_price > ceiling_price, "Treasure: cashPrice not eligible for bond purchase");

            debug_println!("cash_price > ceiling_price");

            let b: u128 = self._cash_balance_of_this();
            assert!(b >= amount, "Treasure: treasury has no more budget");

            debug_println!("b >= amount");

            let sub_value = self.accumulated_seigniorage.checked_sub(self.util.math_min(self.accumulated_seigniorage, amount)).expect("");
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

        #[ink(message)]
        pub fn allocate_seigniorage(&mut self) {
        //     _updateCashPrice();
        //     uint256 cashPrice = _getCashPrice(seigniorageOracle);
        //     if (cashPrice <= getCeilingPrice()) {
        //         return; // just advance epoch instead revert
        //     }
        //
        //     // circulating supply
        //     uint256 percentage = cashPrice.sub(cashPriceOne);
        //     uint256 seigniorage = circulatingSupply().mul(percentage).div(1e18);
        //     IBasisAsset(cash).mint(address(this), seigniorage);
        //
        //     // ======================== BIP-3
        //     uint256 fundReserve = seigniorage.mul(fundAllocationRate).div(100);
        //     if (fundReserve > 0) {
        //         IERC20(cash).safeApprove(fund, fundReserve);
        //         ISimpleERCFund(fund).deposit(
        //             cash,
        //             fundReserve,
        //             'Treasury: Seigniorage Allocation'
        //         );
        //         emit ContributionPoolFunded(now, fundReserve);
        //     }
        //
        //     seigniorage = seigniorage.sub(fundReserve);
        //
        //     // ======================== BIP-4
        //     uint256 treasuryReserve =
        //         Math.min(
        //             seigniorage,
        //             IERC20(bond).totalSupply().sub(accumulatedSeigniorage)
        //         );
        //     if (treasuryReserve > 0) {
        //         if (treasuryReserve == seigniorage) {
        //             treasuryReserve = treasuryReserve.mul(80).div(100);
        //         }
        //         accumulatedSeigniorage = accumulatedSeigniorage.add(
        //             treasuryReserve
        //         );
        //         emit TreasuryFunded(now, treasuryReserve);
        //     }
        //
        //     // boardroom
        //     uint256 boardroomReserve = seigniorage.sub(treasuryReserve);
        //     if (boardroomReserve > 0) {
        //         IERC20(cash).safeApprove(boardroom, boardroomReserve);
        //         IBoardroom(boardroom).allocateSeigniorage(boardroomReserve);
        //         emit BoardroomFunded(now, boardroomReserve);
        //     }
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
