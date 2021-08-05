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

pub use self::boardroom::Boardroom;
use ink_lang as ink;

#[ink::contract]
mod boardroom {
    use ink_prelude::vec::Vec;

    use ink_storage::{
        collections::{
            HashMap,
            Vec as StorageVec,
        },
        lazy::Lazy,
        traits::{PackedLayout, SpreadLayout},
    };
    
    use ink_env::call::FromAccountId;
    use core::convert::TryInto;

    use util::Util;
    use asset::Asset;

    /// Event emitted when a stake occurs that user Stake the ALS.
    #[ink(event)]
    pub struct Staked {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    /// Event emitted when an withdraw occurs that user withdraw the ALS which is staked before.
    #[ink(event)]
    pub struct Withdrawn {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    /// Event emitted when an claim_reward occurs that user claim the reward which is produced by staked ALS.
    #[ink(event)]
    pub struct RewardPaid {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        reward: u128,
    }

    /// Event emitted when an allocate_seigniorage occurs that treasury allocate the reward.
    #[ink(event)]
    pub struct RewardAdded {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        reward: u128,
    }

    /// BoardSeat record the reward should paid to user.
    /// When treasury allocate the reward, boardroom generate the new BoardSeat for user who staked the ALS. 
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct BoardSeat {
        pub last_snapshot_index: u128,
        pub reward_earned: u128,
    }

    /// BoardSnapshot record the reward per ALS.
    /// When treasury allocate the reward, boardroom generate the new BoardSnapshot.
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct BoardSnapshot {
        pub time: u64,
        pub reward_received: u128,
        pub reward_per_share: u128,
    }

    #[ink(storage)]
    pub struct Boardroom {
        util: Lazy<Util>,
        cash: Lazy<Asset>,
        share: Lazy<Asset>,
        stake_total: u128,
        balances: HashMap<AccountId, u128>,
        directors: HashMap<AccountId, BoardSeat>,
        board_history: StorageVec<BoardSnapshot>,

        operator: AccountId,
        status: HashMap<(u32, AccountId), bool>,
    }

    impl Boardroom {
        /// Creates a new boardroom contract with the contract's addresses of ALS, ALC, util.
        #[ink(constructor)]
        pub fn new(cash_address:AccountId,
                   share_address:AccountId,
                   util_address:AccountId) -> Self {
            let share: Asset = FromAccountId::from_account_id(share_address);
            let cash: Asset = FromAccountId::from_account_id(cash_address);
            let util: Util = FromAccountId::from_account_id(util_address);
            let sender = Self::env().caller();
            let mut history: StorageVec<BoardSnapshot> = StorageVec::new();
            let genesis = BoardSnapshot {
                time: Self::env().block_timestamp(),
                reward_received: 0,
                reward_per_share: 0,
            };
            history.push(genesis);

            Self {
                util: Lazy::new(util),
                cash: Lazy::new(cash),
                share: Lazy::new(share),
                stake_total: 0,
                balances: HashMap::new(),
                directors: HashMap::new(),
                board_history: history,
                operator: sender,
                status: HashMap::new(),
            }
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

        fn _director_exists(&self) {
            let sender = Self::env().caller();
            let b:u128 = self.balance_of(sender);
            assert!(b > 0, "Boardroom: : The director does not exist");
        }

        fn _update_reward(&mut self, director:AccountId) {
            assert_ne!(director, AccountId::from([0; 32]));
            let earned = self._earned(director);
            let index = self.latest_snapshot_index();
            self._update_seat(director, earned, index);
        }

        fn _earned(&self, director:AccountId) -> u128 {
            let latest_rps: u128 = self._get_latest_snapshot().reward_per_share;
            let stored_rps: u128 = self._get_last_snapshot_of(director).reward_per_share;

            let latest_rps_sub: u128 = latest_rps.checked_sub(stored_rps).expect("failed at _earned the `boardroom` contract");

            let balance: u128 = self.balance_of(director);
            let balance_mul: u128 = balance.checked_mul(latest_rps_sub).expect("failed at _earned the `boardroom` contract");

            let one_unit: u128 = self.util.get_one_unit_with_decimal();
            let balance_mul_div: u128 = balance_mul.checked_div(one_unit).expect("failed at _earned the `boardroom` contract");

            let seat = self._get_director_board_seat(director).unwrap();
            let earned: u128 = seat.reward_earned;
            let ret: u128 = balance_mul_div.checked_add(earned).expect("failed at _earned the `boardroom` contract");
            return ret;
        }

        /// Get the snapshots for the rewards.
        #[ink(message)]
        pub fn get_snapshots(&self) -> Vec<BoardSnapshot> {
            let mut records:Vec<BoardSnapshot> = Vec::new();
            for history in self.board_history.iter() {
                let r = BoardSnapshot {
                    time: history.time,
                    reward_received: history.reward_received,
                    reward_per_share: history.reward_per_share,
                };
                records.push(r);
            }
            return records;
        }

        /// Get the everyone's rewards detail.
        #[ink(message)]
        pub fn get_seats(&self) -> Vec<BoardSeat> {
            let mut records:Vec<BoardSeat> = Vec::new();
            for (_, value) in self.directors.iter() {
                let r = BoardSeat {
                    last_snapshot_index: value.last_snapshot_index,
                    reward_earned: value.reward_earned,
                };
                records.push(r);
            }
            return records;
        }

        fn _get_latest_snapshot(&self) -> BoardSnapshot {
            let index = self.latest_snapshot_index();
            return self.board_history[index.try_into().unwrap()].clone();
        }

        fn _get_last_snapshot_of(&self, director:AccountId) -> BoardSnapshot {
            let index = self.get_last_snapshot_index_of(director);
            return self.board_history[index.try_into().unwrap()].clone();
        }

        fn _build_empty_board_seat(&self) -> BoardSeat {
            BoardSeat {
                last_snapshot_index: 0,
                reward_earned: 0,
            }
        }

        fn _get_director_board_seat(&self, account: AccountId) -> Option<BoardSeat> {
            let exist = self.directors.contains_key(&account);
            if !exist {
                let r = self._build_empty_board_seat();
                return Some(r)
            }

            return Some(self.directors.get(&account).unwrap().clone());
        }

        fn _update_seat(&mut self, account: AccountId, earned: u128, snap_shot_index: u128) {
            let exist = self.directors.contains_key(&account);
            if !exist {
                let mut r = self._build_empty_board_seat();
                r.reward_earned = earned;
                r.last_snapshot_index = snap_shot_index;
                self.directors.insert(account, r);
                return;
            }

            if let Some(seat) = self.directors.get_mut(&account) {
                seat.reward_earned = earned;
                seat.last_snapshot_index = snap_shot_index;
            }
        }

        fn _stake(&mut self, amount: u128) {
            let total:u128 = self.stake_total;
            self.stake_total = total.checked_add(amount).expect("failed at _stake the `boardroom` contract");

            let sender = Self::env().caller();
            let balance = self.balance_of(sender);
            let value = balance.checked_add(amount).expect("failed at _stake the `boardroom` contract");
            self.balances.insert(sender, value);

            let this = self.env().account_id();
            let ret:bool = self.share.transfer_from(sender, this, amount).is_ok();
            assert!(ret, "Boardroom: _withdraw share.transfer err");
        }

        fn _withdraw(&mut self, amount: u128) {
            let sender = Self::env().caller();
            let balance = self.balance_of(sender);
            assert!(balance >= amount, "Boardroom: withdraw request greater than staked amount");

            let total:u128 = self.stake_total;
            self.stake_total = total.checked_sub(amount).expect("failed at _withdraw the `boardroom` contract");

            let balance = self.balance_of(sender);
            let value = balance.checked_sub(amount).expect("failed at _withdraw the `boardroom` contract");
            self.balances.insert(sender, value);

            let ret:bool = self.share.transfer(sender, amount).is_ok();
            assert!(ret, "Boardroom: _withdraw share.transfer err");
        }

        fn _only_operator(&self) {
            let sender = Self::env().caller();
            assert!(self.operator == sender, "Boardroom: caller is not the operator");
        }

        /// Get the operator who can operate this contract.
        #[ink(message)]
        pub fn operator(&self) -> AccountId {
            return self.operator;
        }

        /// Switch the operator of this contract.
        #[ink(message)]
        pub fn transfer_operator(&mut self, new_operator:AccountId)  {
            self._only_operator();
            self.operator = new_operator;
        }

        /// Get the reward(ALC) amount per ALS.
        #[ink(message)]
        pub fn reward_per_share(&self) -> u128 {
            let index = self.latest_snapshot_index();
            return self.board_history[index.try_into().unwrap()].reward_per_share;
        }

        #[ink(message)]
        pub fn get_last_snapshot_index_of(&self, director:AccountId) -> u128 {
            return self._get_director_board_seat(director).unwrap().last_snapshot_index;
        }

        #[ink(message)]
        pub fn latest_snapshot_index(&self) -> u128 {
            let len:u128 = self.board_history.len().into();
            return len - 1;
        }

        /// The total amount user stake the ALC.
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            return self.stake_total;
        }

        /// User's stake the ALS's amount.
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            return self.balances.get(&account).copied().unwrap_or(0);
        }

        /// User stake the ALS for the ALC reward. 
        #[ink(message)]
        pub fn stake(&mut self, amount: u128) {
            self._check_same_sender_rented();

            // Calculate the reward.
            let sender = Self::env().caller();
            self._update_reward(sender);
            assert!(amount > 0, "Boardroom: Cannot stake 0");

            // Stake the ALS
            self._stake(amount);

            // Emit the event.
            self.env().emit_event(Staked {
                user: Some(sender),
                amount,
            });

            // Ensure stake once per block.
            self._update_sender_rented_status();
        }

        /// Withdraw the ALS user staked.
        #[ink(message)]
        pub fn withdraw(&mut self, amount: u128) {
            self._check_same_sender_rented();
            self._director_exists();

            // Calculate the ALC reward.
            let sender = Self::env().caller();
            self._update_reward(sender);
            assert!(amount > 0, "Boardroom: Cannot withdraw 0");

            // Withdraw the ALS.
            self._withdraw(amount);

            // Emit the event
            self.env().emit_event(Withdrawn {
                user: Some(sender),
                amount,
            });
            // Ensure stake once per block.
            self._update_sender_rented_status();
        }

        /// User exit system, system will return back the ALS, ALC reward.
        #[ink(message)]
        pub fn exit(&mut self) {
            let sender = Self::env().caller();
            let balance = self.balance_of(sender);
            self.withdraw(balance);
            self.claim_reward();
        }

        /// User claim the ALC reward.
        #[ink(message)]
        pub fn claim_reward(&mut self) {
            let sender = Self::env().caller();
            // Caculate the reward.
            self._update_reward(sender);
            let seat = self._get_director_board_seat(sender).unwrap();
            let reward: u128 = seat.reward_earned;
            if reward > 0 {
                let index = seat.last_snapshot_index;
                self._update_seat(sender, 0, index);
                
                // Return back the ALC reward to user.
                let ret:bool = self.cash.transfer(sender, reward).is_ok();
                assert!(ret, "Boardroom: Cannot claim_reward cash.transfer err");

                // Emit the event.
                self.env().emit_event(RewardPaid {
                    user: Some(sender),
                    reward,
                });
            }
        }

        /// Allocate the ALC reward. Called by treasury.
        #[ink(message)]
        pub fn allocate_seigniorage(&mut self, amount: u128) {
            self._only_operator();
            self._check_same_sender_rented();
            assert!(amount > 0, "Boardroom: Cannot allocate 0");

            let total: u128 = self.total_supply();
            assert!(total > 0, "Boardroom: Cannot allocate when total_supply is 0");

            let one_unit_with_decimal: u128 = self.util.get_one_unit_with_decimal();
            let prev_rps: u128 = self.reward_per_share();
            let amount_mul: u128 = amount.checked_mul(one_unit_with_decimal).expect("failed at allocateSeigniorage the `boardroom` contract");
            let amount_mul_div: u128 = amount_mul.checked_div(total).expect("failed at allocateSeigniorage the `boardroom` contract");
            let next_rps: u128 = prev_rps.checked_add(amount_mul_div).expect("failed at allocateSeigniorage the `boardroom` contract");

            // Update the reward per ALS.
            let snapshot = BoardSnapshot {
                time: Self::env().block_timestamp(),
                reward_received: amount,
                reward_per_share: next_rps,
            };
            self.board_history.push(snapshot);

            let sender = Self::env().caller();
            let this = self.env().account_id();

            // transfer the ALC from treasury to this.
            let ret: bool = self.cash.transfer_from(sender, this, amount).is_ok();
            assert!(ret, "Boardroom: allocate_seigniorage transfer_from is err");

            // Emit the event.
            self.env().emit_event(RewardAdded {
                user: Some(sender),
                reward: amount,
            });
            // Ensure stake once per block.
            self._update_sender_rented_status();
        }
    }
}
