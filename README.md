# AlgoCash
Algo Cash specifically uses an algorithmic approach to manage the supply of tokens according to a predetermined logic. The algorithm is in charge of balancing stablecoin supply to fluctuating demand, ensuring that the token price remains relatively stable.


## Code Structure

![image](https://user-images.githubusercontent.com/77781754/123919774-dde5ce80-d9b7-11eb-8426-57c312b3756a.png)

In this project, the 3 most important contracts are asset, treasure and boradroom.

### Asset

From asset contract, we can instantiate 3 tokens which would be used in the project, ALC, ALB and ALS.

### Treasure

The Treasury contract handles bond（ALC） purchases and redemptions

The Algo Cash Treasury exists to enable bond-to-cash redemptions. Bonds redeemed via the Treasury automatically returns the user an equal number of ALC, provided that: 1) the oracle price of ALC is above 1 USD (could be set to any price of any asset), and 2) the Treasury contract has a positive balance of ALC.

Disallowing redemptions when the ALC price is below 1 USD prevents bond holders from prematurely cutting their losses and creating unnecessary downward pressure on the price of ALC.

### Boardroom

The Boardroom contract handles dividend claims from Share holders

The Boardroom allows ALS holders to claim excess ALC minted by the protocol. Holders of ALS can stake their Shares to the Boardroom contract, which by doing so, they can claim a pro-rata share of ALC assigned to the Boardroom.


## Deployment

In Polkadotjs-ui, we use uplaod & deploy to deploy the asset, oracle and treasure contacts. With asset contract, we can create Cash (ALC) and Bond (ALB) token instance. In treasure contract, we set ALC, ALB, oracle and decimal (10000000000) as paras. 

## Instruction
### Build Dependencies
Ubuntu/Debian:   
sudo apt update   
sudo apt install -y git clang curl libssl-dev llvm libudev-dev   

macOS:      
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"   
brew update   
brew install openssl   

### Rust Developer Environment
#### Automated getsubstrate.io Script   
curl https://getsubstrate.io -sSf | bash -s -- --fast

#### Manual Rust Configuration      
curl https://sh.rustup.rs -sSf | sh   
source ~/.cargo/env   

rustup default stable   
rustup update   
rustup update nightly   
rustup target add wasm32-unknown-unknown --toolchain nightly

#### toolchain
rustup component add rust-src --toolchain nightly   
rustup target add wasm32-unknown-unknown --toolchain nightly   

#### ink!
##### install binaryen
Ubuntu/Debian:   
sudo apt install binaryen  

macOS:   
brew install binaryen   

##### cargo-contract
cargo install --force cargo-contract

### compile
1:  git clone https://github.com/ReserveLabs/AlgoCash.git   
2:  cd AlgoCash   
2:  npm install      
3:  npx redspot compile   

### deploy
1: npx redspot run scripts/deploy.ts

### test
```
cargo install europa --git https://github.com/patractlabs/europa --locked --force
europa --tmp
```
1: npx redspot test tests/distributor.test.ts   
2: npx redspot test tests/boardroom.test.ts   
3: npx redspot test tests/treasury.test.ts

## Docker
wget dl.veim.cn/download/algocash/europa-algocash.tar.gz

gunzip europa-algocash.tar.gz

docker load -i europa-algocash.tar 

$ docker images

| REPOSITORY | TAG | IMAGE ID | CREATED |SIZE|
| ------------- | ------------- | ------------- |------------- |------------- |
| europa-algocash | 1.0.0 | bc54f6339fb1 |14 hours ago |   4.39GB |


docker run -d --name europa-node europa-algocash:1.0.0 europa -d database

docker exec -it europa-node bash

cd github/AlgoCash/

test cases:

1: npx redspot test tests/distributor.test.ts

2: npx redspot test tests/boardroom.test.ts

3: npx redspot test tests/treasury.test.ts
