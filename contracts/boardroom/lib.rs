#![cfg_attr(not(feature = "std"), no_std)]

pub use self::boardroom::Boardroom;
use ink_lang as ink;

#[ink::contract]
mod boardroom {
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
            Vec as StorageVec,
        },
        lazy::Lazy,
        traits::{PackedLayout, SpreadLayout},
    };

    use ink_env::call::FromAccountId;
    use ink_env::debug_println;
    use core::convert::TryInto;

    use util::Util;
    use asset::Asset;

    #[ink(event)]
    pub struct Staked {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    #[ink(event)]
    pub struct Withdrawn {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        amount: u128,
    }

    #[ink(event)]
    pub struct RewardPaid {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        reward: u128,
    }

    #[ink(event)]
    pub struct RewardAdded {
        #[ink(topic)]
        user: Option<AccountId>,
        #[ink(topic)]
        reward: u128,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct BoardSeat {
        pub last_snapshot_index: u128,
        pub reward_earned: u128,
    }

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct BoardSnapshot {
        pub time: u32,
        pub reward_received: u128,
        pub reward_per_share: u128,
    }

    #[ink(storage)]
    pub struct Boardroom {
        util: Lazy<Util>,
        cash: Lazy<Asset>,
        share: Lazy<Asset>,
        stake_total: u128,
        balances: StorageHashMap<AccountId, u128>,
        directors: StorageHashMap<AccountId, BoardSeat>,
        board_history: StorageVec<BoardSnapshot>,
    }

    impl Boardroom {
        #[ink(constructor)]
        pub fn new(cash_address:AccountId,
                   share_address:AccountId,
                   util_address:AccountId) -> Self {
            let share: Asset = FromAccountId::from_account_id(share_address);
            let cash: Asset = FromAccountId::from_account_id(cash_address);
            let util: Util = FromAccountId::from_account_id(util_address);

            let mut history: StorageVec<BoardSnapshot> = StorageVec::new();
            let genesis = BoardSnapshot {
                time: Self::env().block_number(),
                reward_received: 0,
                reward_per_share: 0,
            };
            history.push(genesis);

            Self {
                util: Lazy::new(util),
                cash: Lazy::new(cash),
                share: Lazy::new(share),
                stake_total: 0,
                balances: StorageHashMap::new(),
                directors: StorageHashMap::new(),
                board_history: history,
            }
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

            let latest_rps_sub: u128 = latest_rps.checked_sub(stored_rps).expect("");

            let balance: u128 = self.balance_of(director);
            let balance_mul: u128 = balance.checked_mul(latest_rps_sub).expect("");

            let one_unit: u128 = self.util.get_one_unit_with_decimal();
            let balance_mul_div: u128 = balance_mul.checked_div(one_unit).expect("");

            let seat = self._get_director_board_seat(director).unwrap();
            let earned: u128 = seat.reward_earned;
            let ret: u128 = balance_mul_div.checked_add(earned).expect("");
            return ret;
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
            let r = self._build_empty_board_seat();
            let exist = self.directors.contains_key(&account);
            if !exist {
                return Some(r)
            }

            return Some(self.directors.get(&account).unwrap().clone());
        }

        fn _update_seat(&mut self, account: AccountId, earned: u128, snap_shot_index: u128) {
            if let Some(seat) = self.directors.get_mut(&account) {
                seat.reward_earned = earned;
                seat.last_snapshot_index = snap_shot_index;
            }
        }

        fn _stake(&mut self, amount: u128) {
            let total:u128 = self.stake_total;
            self.stake_total = total.checked_add(amount).expect("");

            let sender = Self::env().caller();
            let balance = self.balance_of(sender);
            let value = balance.checked_add(amount).expect("");
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
            self.stake_total = total.checked_sub(amount).expect("");

            let balance = self.balance_of(sender);
            let value = balance.checked_sub(amount).expect("");
            self.balances.insert(sender, value);

            let ret:bool = self.share.transfer(sender, amount).is_ok();
            assert!(ret, "Boardroom: _withdraw share.transfer err");
        }

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

        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            return self.stake_total;
        }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            return self.balances.get(&account).copied().unwrap_or(0);
        }

        #[ink(message)]
        pub fn stake(&mut self, amount: u128) {
            let sender = Self::env().caller();
            self._update_reward(sender);
            assert!(amount > 0, "Boardroom: Cannot stake 0");
            self._stake(amount);
            self.env().emit_event(Staked {
                user: Some(sender),
                amount,
            });
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: u128) {
            let sender = Self::env().caller();
            self._director_exists();
            self._update_reward(sender);
            assert!(amount > 0, "Boardroom: Cannot withdraw 0");
            self._withdraw(amount);
            self.env().emit_event(Withdrawn {
                user: Some(sender),
                amount,
            });
        }

        #[ink(message)]
        pub fn exit(&mut self) {
            let sender = Self::env().caller();
            let balance = self.balance_of(sender);
            self.withdraw(balance);
            self.claim_reward();
        }

        #[ink(message)]
        pub fn claim_reward(&mut self) {
            let sender = Self::env().caller();
            let seat = self._get_director_board_seat(sender).unwrap();
            let reward: u128 = seat.reward_earned;
            if reward > 0 {
                let index = seat.last_snapshot_index;
                self._update_seat(sender, 0, index);

                let ret:bool = self.cash.transfer(sender, reward).is_ok();
                assert!(ret, "Boardroom: Cannot claim_reward cash.transfer err");

                self.env().emit_event(RewardPaid {
                    user: Some(sender),
                    reward,
                });
            }
        }

        #[ink(message)]
        pub fn allocate_seigniorage(&mut self, amount: u128) {
            assert!(amount > 0, "Boardroom: Cannot allocate 0");

            let total: u128 = self.total_supply();
            assert!(total > 0, "Boardroom: Cannot allocate when total_supply is 0");

            let one_unit_with_decimal: u128 = self.util.get_one_unit_with_decimal();
            let prev_rps: u128 = self.reward_per_share();
            let amount_mul: u128 = amount.checked_mul(one_unit_with_decimal).expect("");
            let amount_mul_div: u128 = amount_mul.checked_div(total).expect("");
            let next_rps: u128 = prev_rps.checked_add(amount_mul_div).expect("");

            let snapshot = BoardSnapshot {
                time: Self::env().block_number(),
                reward_received: amount,
                reward_per_share: next_rps,
            };
            self.board_history.push(snapshot);

            let sender = Self::env().caller();
            let this = self.env().account_id();
            let ret: bool = self.cash.transfer_from(sender, this, amount).is_ok();
            assert!(ret, "Boardroom: allocate_seigniorage transfer_from is err");

            self.env().emit_event(RewardAdded {
                user: Some(sender),
                reward: amount,
            });
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
