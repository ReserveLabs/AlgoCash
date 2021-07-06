#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod boardroom {

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        lazy::Lazy,
        traits::{PackedLayout, SpreadLayout},
    };

    use ink_env::call::FromAccountId;
    use ink_env::debug_println;

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
        pub time: u128,
        pub reward_received: u128,
        pub reward_per_share: u128,
    }

    #[ink(storage)]
    pub struct Boardroom {
        cash: Lazy<Asset>,
        share: Lazy<Asset>,
        stake_total: u128,
        balances: StorageHashMap<AccountId, u128>,
        directors: StorageHashMap<AccountId, BoardSeat>,
    }

    impl Boardroom {
        #[ink(constructor)]
        pub fn new(cash_address:AccountId, share_address:AccountId) -> Self {
            let share: Asset = FromAccountId::from_account_id(share_address);
            let cash: Asset = FromAccountId::from_account_id(cash_address);
            Self {
                cash: Lazy::new(cash),
                share: Lazy::new(share),
                stake_total: 0,
                balances: StorageHashMap::new(),
                directors: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            return self.stake_total;
        }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            return self.balances.get(&account).copied().unwrap_or(0);
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

        fn _update_earned(&mut self, account: AccountId, earned: u128) {
            if let Some(seat) = self.directors.get_mut(&account) {
                seat.reward_earned = earned;
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
        pub fn stake(&mut self, amount: u128) {
            assert!(amount > 0, "Boardroom: Cannot stake 0");
            self._stake(amount);
            let sender = Self::env().caller();
            self.env().emit_event(Staked {
                user: Some(sender),
                amount,
            });
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: u128) {
            assert!(amount > 0, "Boardroom: Cannot withdraw 0");
            self._withdraw(amount);
            let sender = Self::env().caller();
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
            let reward: u128 = self._get_director_board_seat(sender).unwrap().reward_earned;
            if reward > 0 {
                self._update_earned(sender, 0);

                let ret:bool = self.cash.transfer(sender, reward).is_ok();
                assert!(ret, "Boardroom: Cannot claim_reward cash.transfer err");

                self.env().emit_event(RewardPaid {
                    user: Some(sender),
                    reward,
                });
            }
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
