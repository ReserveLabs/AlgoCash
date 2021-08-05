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
mod treasury {
    use ink_env::call::FromAccountId;
    use ink_storage::{
        collections::HashMap,
        lazy::Lazy,
    };

    use util::Util;
    use oracle::Oracle;
    use asset::Asset;
    use boardroom::Boardroom;

    #[ink(storage)]
    pub struct Treasury {
        bond_cap: u128,
        accumulated_seigniorage: u128,

        room_address: AccountId,
        util:  Lazy<Util>,
        oracle:  Lazy<Oracle>,
        cash: Lazy<Asset>,
        bond: Lazy<Asset>,
        share: Lazy<Asset>,
        boardroom:  Lazy<Boardroom>,

        status: HashMap<(u32, AccountId), bool>,
    }

    /// Event emitted when an redeem_bonds occurs that user redeem the ALB.
    #[ink(event)]
    pub struct RedeemedBonds {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    /// Event emitted when an buy_bonds occurs that user buy the ALB.
    #[ink(event)]
    pub struct BoughtBonds {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    /// Event emitted when an allocate_seigniorage occurs that system allocate the reward.
    #[ink(event)]
    pub struct TreasuryFunded {
        #[ink(topic)]
        timestamp: u64,
        #[ink(topic)]
        seigniorage: u128,
    }

    /// Event emitted when an allocate_seigniorage occurs that system allocate the reward.
    #[ink(event)]
    pub struct BoardroomFunded {
        #[ink(topic)]
        timestamp: u64,
        #[ink(topic)]
        seigniorage: u128,
    }


    impl Treasury {
        /// Create new treasury
        #[ink(constructor)]
        pub fn new(util_address:AccountId,
                   cash_address:AccountId,
                   bond_address: AccountId,
                   share_address: AccountId,
                   oracle_address: AccountId,
                   boardroom_address: AccountId) -> Self {

            let util: Util = FromAccountId::from_account_id(util_address);
            let cash: Asset = FromAccountId::from_account_id(cash_address);
            let bond: Asset = FromAccountId::from_account_id(bond_address);
            let share: Asset = FromAccountId::from_account_id(share_address);
            let oracle: Oracle = FromAccountId::from_account_id(oracle_address);
            let boardroom: Boardroom = FromAccountId::from_account_id(boardroom_address);

            let instance = Self {
                bond_cap: 0,
                accumulated_seigniorage: 0,

                room_address: boardroom_address,
                util: Lazy::new(util),
                cash: Lazy::new(cash),
                bond: Lazy::new(bond),
                share: Lazy::new(share),
                oracle: Lazy::new(oracle),
                boardroom: Lazy::new(boardroom),
                status: HashMap::new(),
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
            let r = cash_supply.checked_sub(self.accumulated_seigniorage).expect("failed at _circulating_supply the `treasury` contract");
            return r;
        }

        fn _update_conversion_limit(&mut self, cash_price: u128) {
            let cash_price_one = self.util.get_one_unit_with_decimal();
            let percentage = cash_price_one.checked_sub(cash_price).expect("failed at _update_conversion_limit the `treasury` contract");

            let cap = self._circulating_supply().checked_mul(percentage).expect("failed at _update_conversion_limit the `treasury` contract");

            let decimal = self.util.get_decimal();
            let b_cap = cap.checked_div(decimal.into()).expect("failed at _update_conversion_limit the `treasury` contract");

            let bond_supply: u128 = self.bond.total_supply();

            self.bond_cap = b_cap.checked_sub(self.util.math_min(b_cap, bond_supply)).expect("failed at _update_conversion_limit the `treasury` contract");
        }

        fn _check_operator(&self) {
            let this = self.env().account_id();
            assert!(self.cash.operator() == this &&
                    self.bond.operator() == this &&
                    self.share.operator() == this &&
                    self.boardroom.operator() == this, "Treasury: need more permission");
        }

        fn _check_same_sender_rented(&self) {
            let block_num:u32 = Self::env().block_number();
            let sender = Self::env().caller();
            let rented:bool = self.status.get(&(block_num, sender)).copied().unwrap_or(false);
            assert!(!rented, "Boardroom: : _check_same_sender_rented err");
        }

        fn _update_sender_rented_status(&mut self) {
            let block_num:u32 = Self::env().block_number();
            let sender = Self::env().caller();
            self.status.insert((block_num, sender), true);
        }

        /// If the ALC's price < $1, we assume the ALC's price is 0.9. User may buy the ALB with the ALC's current price(0.9*0.9).
        /// When the ALC's price >= $1, user can redeem the ALB, system will transfer the ALC to user with the ALC's current price.
        #[ink(message)]
        pub fn buy_bonds(&mut self, amount: u128, target_price: u128) {
            self._check_operator();
            self._check_same_sender_rented();
            assert!(amount > 0, "Treasury: cannot purchase bonds with zero amount");

            // get ALC's price from oracle.
            let cash_price:u128 = self.oracle.get_cash_price();
            assert!(cash_price <= target_price, "Treasury: cash price moved");

            let cash_price_one = self.util.get_one_unit_with_decimal();
            assert!(cash_price < cash_price_one, "Treasury: cash_price not eligible for bond purchase");

            // Caculate the user's ALC amount which should be burnt.
            self._update_conversion_limit(cash_price);

            let mul_value = self.bond_cap.checked_mul(cash_price).expect("failed at buyBonds the `treasury` contract");

            let one_unit_with_decimal = self.util.get_one_unit_with_decimal();
            let div_value = mul_value.checked_div(one_unit_with_decimal).expect("failed at buyBonds the `treasury` contract");
            let amount = self.util.math_min(amount, div_value);

            assert!(amount > 0, "Treasury: amount exceeds bond cap");

            let mul_value = amount.checked_mul(one_unit_with_decimal).expect("failed at buyBonds the `treasury` contract");
            let div_value = mul_value.checked_div(cash_price).expect("failed at buyBonds the `treasury` contract");

            // Burn the user's ALC.
            let sender = Self::env().caller();
            let burn_ret:bool = self.cash.burn_from(sender, amount).is_ok();
            assert!(burn_ret, "Treasury: transfer ok");

            // Mint the ALB to user.
            let mint:bool = self.bond.mint(sender, div_value).is_ok();
            assert!(mint, "Treasury: mint ok");

            // Emit the event.
            self.env().emit_event(BoughtBonds {
                from: Some(sender),
                amount,
            });

            // ensure only buy_bonds once per block.
            self._update_sender_rented_status();
        }

        /// When the ALC's price >= $1.05, user can redeem the ALB, system will transfer the ALC to user with the ALC's current price.
        #[ink(message)]
        pub fn redeem_bonds(&mut self, amount: u128) {
            self._check_operator();
            self._check_same_sender_rented();
            assert!(amount > 0, "Treasury: cannot redeem bonds with zero amount");

            let cash_price:u128 = self.oracle.get_cash_price();
            let ceiling_price:u128 = self.util.get_ceiling_price();
            assert!(cash_price > ceiling_price, "Treasury: cashPrice not eligible for bond purchase");

            let b: u128 = self._cash_balance_of_this();
            assert!(b >= amount, "Treasury: treasury has no more budget");

            let sub_value = self.accumulated_seigniorage.checked_sub(self.util.math_min(self.accumulated_seigniorage, amount)).expect("failed at redeemBonds the `treasury` contract");
            self.accumulated_seigniorage = sub_value;

            // Burn the user's ALB.
            let sender = Self::env().caller();
            let burn_ret: bool = self.bond.burn_from(sender, amount).is_ok();
            assert!(burn_ret, "Treasury: transfer ok");

            // Transfer the ALC to user.
            let trans_ret: bool = self.cash.transfer(sender, amount).is_ok();
            assert!(trans_ret, "Treasury: transfer ok");

            // Emit the event.
            self.env().emit_event(RedeemedBonds {
                from: Some(sender),
                amount,
            });

            // ensure only redeem_bonds once per block.
            self._update_sender_rented_status();
        }

        /// Allocate the ALC to boardroom, the other is the reward of user who stake the ALS.
        #[ink(message)]
        pub fn allocate_seigniorage(&mut self) {
            self._check_operator();
            self._check_same_sender_rented();
            let cash_price:u128 = self.oracle.get_cash_price();
            let ceiling_price:u128 = self.util.get_ceiling_price();
            assert!(cash_price > ceiling_price, "Treasury: cashPrice not eligible for allocate_seigniorage");

            // circulating supply
            let cash_price_one = self.util.get_one_unit_with_decimal();
            let percentage:u128 = cash_price.checked_sub(cash_price_one).expect("failed at allocateSeigniorage the `treasury` contract");
            let seigniorage_mul:u128 = self._circulating_supply().checked_mul(percentage).expect("failed at allocateSeigniorage the `treasury` contract");
            let seigniorage:u128 = seigniorage_mul.checked_div(cash_price_one).expect("failed at allocateSeigniorage the `treasury` contract");

            assert!(seigniorage > 0, "seigniorage should above 0");    

            // mint the ALC.
            let this = self.env().account_id();
            let mint_ret:bool = self.cash.mint(this, seigniorage).is_ok();
            assert!(mint_ret, "Treasury: allocate_seigniorage mint err");

            let bond_total:u128 = self.bond.total_supply();
            let bond_total_sub:u128 = bond_total.checked_sub(self.accumulated_seigniorage).expect("failed at allocateSeigniorage the `treasury` contract");
            let treasury_reserve_ori = self.util.math_min(seigniorage, bond_total_sub);
            let mut treasury_reserve: u128 = 0;
            if treasury_reserve_ori > 0 {
                if treasury_reserve_ori == seigniorage {
                    let treasury_reserve_mul:u128 = treasury_reserve_ori.checked_mul(80).expect("failed at allocateSeigniorage the `treasury` contract");
                    treasury_reserve = treasury_reserve_mul.checked_div(100).expect("failed at allocateSeigniorage the `treasury` contract");
                }
                self.accumulated_seigniorage = self.accumulated_seigniorage.checked_add(treasury_reserve).expect("failed at allocateSeigniorage the `treasury` contract");
                self.env().emit_event(TreasuryFunded {
                    timestamp: Self::env().block_timestamp(),
                    seigniorage: treasury_reserve,
                });
            }

            // boardroom
            let boardroom_reserve:u128 = seigniorage.checked_sub(treasury_reserve).expect("failed at allocateSeigniorage the `treasury` contract");
            if boardroom_reserve > 0 {
                let ret:bool = self.cash.approve(self.room_address, boardroom_reserve).is_ok();
                assert!(ret, "Treasury: allocate_seigniorage approve err");

                // allocate the ALC as reward to user.
                self.boardroom.allocate_seigniorage(boardroom_reserve);
                self.env().emit_event(BoardroomFunded {
                    timestamp: Self::env().block_timestamp(),
                    seigniorage: treasury_reserve,
                });
            }
            self._update_sender_rented_status();
        }
    }
}
