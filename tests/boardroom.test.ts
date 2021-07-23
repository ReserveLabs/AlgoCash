import { expect } from "chai";
import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

describe("boardroom", () => {
    after(() => {
        return api.disconnect();
    });

    async function setup() {
        await api.isReady

        const signerAddresses = await getAddresses();
        const Alice = signerAddresses[0];
        const Bob = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"; // Bob Address
        
        const alcFactory = await getContractFactory('asset', Alice);
        const tsForAlc = (new Date().getTime()).toString();
        const alc = await alcFactory.deploy('new', '0', 'ALC', 'ALC', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForAlc
        });

        const albFactory = await getContractFactory("asset", Alice);
        const tsForAlb = (new Date().getTime()).toString();
        const alb = await albFactory.deploy('new', '0', 'ALB', 'ALB', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForAlb
        });

        const alsFactory = await getContractFactory("asset", Alice);
        const tsForAls = (new Date().getTime()).toString();
        const als = await alsFactory.deploy('new', '0', 'ALS', 'ALS', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForAls
        });

        const orcaleFactory = await getContractFactory('oracle', Alice);
        const tsForOracle = (new Date().getTime()).toString();
        const oracle = await orcaleFactory.deploy('new', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForOracle
        });

        const utilFactory = await getContractFactory('util', Alice);
        const tsForUtil = (new Date().getTime()).toString();
        const util = await utilFactory.deploy('new', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForUtil
        });

        const boardroomFactory = await getContractFactory('boardroom', Alice);
        const tsForBoardroom = (new Date().getTime()).toString();
        const boardroom = await boardroomFactory.deploy('new', alc.address, als.address, util.address, {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForBoardroom
        });

        const treasuryFactory = await getContractFactory('treasury', Alice);
        const tsForTreasury = (new Date().getTime()).toString();
        const treasury = await treasuryFactory.deploy('new', util.address, alc.address, alb.address, als.address, oracle.address, boardroom.address, {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForTreasury
        });

        return { Alice, Bob, alc, alb, als, oracle, boardroom, treasury };
    }

    it("Stake als, mint alc, withdraw als, claim alc reward", async () => {
        const { Alice, Bob, alc, alb, als, oracle, boardroom, treasury } = await setup();
        
        const decimal = 10000000000;

        console.log("mint als to alice");
        await als.tx.mint(Alice, 100*decimal);
        await als.tx.approve(boardroom.address, 100*decimal);
        const alice_als_allowance = await als.query.allowance(Alice, boardroom.address);
        expect(alice_als_allowance.output).to.equal(100*decimal);

        console.log("mint alc to treasury");
        await alc.tx.mint(treasury.address, 10000*decimal);
        const treasury_alc_balance = await alc.query.balanceOf(treasury.address);
        expect(treasury_alc_balance.output).to.equal(10000*decimal);

        console.log("transfer operator for alc");
        await alc.tx.transferOperator(treasury.address);
        const alc_operator = await alc.query.operator();
        expect(alc_operator.output).to.equal(treasury.address);

        console.log("transfer operator for alb");
        await alb.tx.transferOperator(treasury.address);
        const alb_operator = await alb.query.operator();
        expect(alb_operator.output).to.equal(treasury.address);

        console.log("transfer operator for als");
        await als.tx.transferOperator(treasury.address);
        const als_operator = await als.query.operator();
        expect(als_operator.output).to.equal(treasury.address);

        console.log("transfer operator for boardroom");
        await boardroom.tx.transferOperator(treasury.address);
        const boardroom_operator = await boardroom.query.operator();
        expect(boardroom_operator.output).to.equal(treasury.address);

        console.log("alice stake als");
        await boardroom.tx.stake(100*decimal);    
        const alice_als_stake_balance = await als.query.balanceOf(Alice);
        expect(alice_als_stake_balance.output).to.equal(0);
        const boardroom_alc_stake_balance = await als.query.balanceOf(boardroom.address);
        expect(boardroom_alc_stake_balance.output).to.equal(100*decimal);

        console.log("update cash price to 1.1");
        await oracle.tx.updateCashPrice(11000000000, 123);
        const b_price = await oracle.query.getCashPrice();
        expect(b_price.output).to.equal(11000000000);

        console.log("allocate seigniorage");
        await treasury.tx.allocateSeigniorage();

        console.log("withdraw als");
        await boardroom.tx.withdraw(100*decimal); 
        const alice_als_balance = await als.query.balanceOf(Alice);
        expect(alice_als_balance.output).to.equal(100*decimal);
        const boardroom_als_balance = await als.query.balanceOf(boardroom.address);
        expect(boardroom_als_balance.output).to.equal(0);

        console.log("claim reward: alc");
        await boardroom.tx.claimReward(); 
        const alice_alc_balance = await alc.query.balanceOf(Alice);
        expect(alice_alc_balance.output).to.equal(1000*decimal);
    });
});
