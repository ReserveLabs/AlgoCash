# AlgoCash
Algo Cash specifically uses an algorithmic approach to manage the supply of tokens according to a predetermined logic. The algorithm is in charge of balancing stablecoin supply to fluctuating demand, ensuring that the token price remains relatively stable.

## Tokens
### ALC - Algo Cash
Algo Cash tokens are designed to be used as a medium of exchange. The built-in stability mechanism expands and contracts their supply, maintaining their peg to the aUSD token.

### ALB - Algo Bonds
Algo Bonds are minted and redeemed to incentivize changes in the Algo Cash supply. Bonds are always on sale to Algo Cash holders, although purchases are expected to be made at a price below 1 Algo Cash. At any given time, holders are able to exchange their bonds to Algo Cash tokens in the Algo Cash Treasury. Upon redemption, they are able to convert 1 Algo Bond to 1 Algo Cash, earning them a premium on their previous bond purchases.

Bonds do not have expiration dates. All holders are able to convert their bonds to Algo Cash tokens, as long as the Treasury has a positive ALC balance.

### ALS - Algo Shares
Algo Shares loosely represent the value of the Algo Cash network. Increased demand for Algo Cash results in new Algo Cash tokens to be minted and distributed to Algo Share holders, provided that the Treasury is sufficiently full.

Holders of Algo Share tokens can claim a pro-rata share of Algo Cash tokens accumulated to the Boardroom contract.

## Pools
### Treasury
The Algo Cash Treasury exists to enable bond-to-cash redemptions. Bonds redeemed via the Treasury automatically returns the user an equal number of Algo Cash, provided that: 1) the oracle price of Algo Cash is above 1 aUSD, and 2) the Treasury contract has a positive balance of Algo Cash.

Disallowing redemptions when the Algo Cash price is below 1 aUSD prevents bond holders from prematurely cutting their losses and creating unnecessary downward pressure on the price of ALC.

### Boardroom
The Boardroom allows Algo Share holders to claim excess Algo Cash minted by the protocol. Holders of Algo Shares can stake their Shares to the Boardroom contract, which by doing so, they can claim a pro-rata share of Algo Cash tokens assigned to the Boardroom.

## Code Structure

![image](https://user-images.githubusercontent.com/77781754/123919774-dde5ce80-d9b7-11eb-8426-57c312b3756a.png)

## Deployment

In Polkadotjs-ui, use uplaod & deploy to deploy the asset, oracle and treasure contacts. With asset contract, we can create Cash (ALC) and Bond (ALB) token instance. In treasure contract, we set ALC, ALB, oracle and decimal (10000000000) as paras. 

## Usage
### compile
1:  git clone https://github.com/ReserveLabs/AlgoCash.git
2:  cd AlgoCash
2:  npm install   
3:  npx redspot compile

### deploy
1: npx redspot run scripts/deploy.ts

### test
1: npx redspot test tests/distributor.test.ts   
2: npx redspot test tests/boardroom.test.ts   
3: npx redspot test tests/treasury.test.ts

