import { expect } from "chai";
import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

// BOB: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty

describe("treasury", () => {
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

    it("buy and redeem bonds", async () => {
        const { Alice, alc, alb, als, oracle, boardroom, treasury } = await setup();

        const decimal = 10000000000;

        console.log("mint alc to treasury: ", treasury.address.toString());
        await alc.tx.mint(treasury.address, 10000*decimal);
        const treasury_alc_balance = await alc.query.balanceOf(treasury.address);
        expect(treasury_alc_balance.output).to.equal(10000*decimal);

        console.log("mint alc to alice");
        await alc.tx.mint(Alice, 100*decimal);
        await alc.tx.approve(treasury.address, 100*decimal);
        const alice_alc_balance = await alc.query.balanceOf(Alice);
        expect(alice_alc_balance.output).to.equal(100*decimal);

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

        console.log("update cash price to 0.9");
        await oracle.tx.updateCashPrice(9000000000, 123);
        const b_price = await oracle.query.getCashPrice();
        expect(b_price.output).to.equal(9000000000);

        console.log("alice buy bonds");
        await treasury.tx.buyBonds(100*decimal, 1*decimal);
        const alc_balance = await alc.query.balanceOf(Alice);
        expect(alc_balance.output).to.equal(0);

        const alb_balance = await alb.query.balanceOf(Alice);
        expect(alb_balance.output).to.equal(1111111111111);

        console.log("alb approve to treasury");
        await alb.tx.approve(treasury.address, 1111111111111);

        console.log("update cash price to 1.1");
        await oracle.tx.updateCashPrice(11000000000, 234);
        const l_price = await oracle.query.getCashPrice();
        expect(l_price.output).to.equal(11000000000);

        console.log("alice redeem bonds");
        await treasury.tx.redeemBonds(1111111111111);
        const final_balance = await alc.query.balanceOf(Alice);
        expect(final_balance.output).to.equal(1111111111111);
    });
});
