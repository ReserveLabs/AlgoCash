# AlgoCash
An algorithmic stablecoin

## Contract

### Treasure

     #[ink(event)]
     pub struct RedeemedBonds {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        amount: u8,
      
     }
      
     #[ink(event)]
     pub struct BoughtBonds {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        amount: u8,
      
     }
     
     #[ink(event)]
     pub struct TreasuryFunded {
        #[ink(topic)]
        timestamp: u8,
        #[ink(topic)]
        seigniorage: u8,
      
     }

     #[ink(event)]
     pub struct BoardroomFunded {
        #[ink(topic)]
        timestamp: u8,
        #[ink(topic)]
        seigniorage: u8,
      
     }

### Boardroom

     #[ink(event)]
     pub struct Staked {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: u8,
      
     }
     
     #[ink(event)]
     pub struct Withdrawn {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: u8,
      
     }
     
     #[ink(event)]
     pub struct RewardPaid {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        reward: u8,
      
     }
     
     #[ink(event)]
     pub struct RewardAdded {
        #[ink(topic)]
        reward: u8,
             
     }
  
  ### Token Distributor
  
     #[ink(event)]
     pub struct RewardAdded {
        #[ink(topic)]
        reward: u8,
             
     }
     
     #[ink(event)]
     pub struct Staked {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: u8,
      
     }
  
     #[ink(event)]
     pub struct Withdrawn {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: u8,
      
     }
     
     
     #[ink(event)]
     pub struct RewardPaid {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        reward: u8,
      
     }
