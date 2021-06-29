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


## Stabilization Mechanism
The Algo Cash protocol is designed to guarantee Algo Cash tokens to be exchanged at a value of a single US dollar, with the stabilizer (in-protocol stability mechanism) in charge of matching the supply of Algo Cash to their demand.

Every 24 hours, the time-weighted average of the ALC-aUSD exchange rate is read from the DEX Aggregator in Polkaot, which is then fed into the Algo Cash protocol to be referenced by its stability mechanism.

The stabilization mechanism is triggered whenever the price of Algo Cash is observed to be above / below (1+ε) aUSD, where ε is a parameter that defines the range of price stability for the ALC token. In inilization, ε is set to be 0.05.

### Contractionary Policy
At any point in time, Algo Bonds can be bought from the protocol in exchange for Algo Cash. Purchase of Bonds are performed at an algorithmically set price. With a Algo Cash oracle price of P aUSD, bonds are sold off at a price of P ALC (effective price being P^2 aUSD), promising bond holders a premium when redeemed. Purchased bonds can be converted to Algo Cash, insofar as the preconditions are met and the Treasury is not empty.

### Expansionary Policy
If the price of Algo Cash is observed to be higher than (1+ε) aUSDT, the system mints totalSupply *(oraclePrice-1) number of new Algo Cash tokens. The issued Algo Cash is either deposited to the Treasury or the Boardroom, depending on the Algo Cash balance of the Treasury.

If the Treasury has a balance above 1,000 Algo Cash, then it is logical to assume that either all bonds have been already redeemed, or no bond holder is currently willing to perform a redemption.Either way, this signals that the demand for bond redemption do not exist at this time, and thus the freshly minted Algo Cash is given to the Boardroom contract.

However, if the Treasury has a balance of below 1,000 Algo Cash, then it is assumed that there will be additional demand for bond-to-cash redemption. Therefore, the issued Algo Cash is routed to the Treasury so that Bond holders can exercise redemptions.

## Usage
1:  git clone https://github.com/ReserveLabs/AlgoCash.git   
2:  npm install   
3:  npx redspot compile



