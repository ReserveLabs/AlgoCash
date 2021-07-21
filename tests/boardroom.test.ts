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

    // it("Assigns initial balance", async () => {
    //     const { contract, sender } = await setup();
    //     const result = await contract.query.balanceOf(sender.address);
    //     expect(result.output).to.equal(1000);
    // });

    // it("Transfer adds amount to destination account", async () => {
    //     const { contract, receiver } = await setup();

    //     await expect(() =>
    //         contract.tx.transfer(receiver.address, 7)
    //     ).to.changeTokenBalance(contract, receiver, 7);

    //     await expect(() =>
    //         contract.tx.transfer(receiver.address, 7)
    //     ).to.changeTokenBalances(contract, [contract.signer, receiver], [-7, 7]);
    // });

    // it("Transfer emits event", async () => {
    //     const { contract, sender, receiver } = await setup();

    //     await expect(contract.tx.transfer(receiver.address, 7))
    //         .to.emit(contract, "Transfer")
    //         .withArgs(sender.address, receiver.address, 7);
    // });

    // it("Can not transfer above the amount", async () => {
    //     const { contract, receiver } = await setup();

    //     await expect(contract.tx.transfer(receiver.address, 1007)).to.not.emit(
    //         contract,
    //         "Transfer"
    //     );
    // });

    // it("Can not transfer from empty account", async () => {
    //     const { contract, Alice, sender } = await setup();

    //     const emptyAccount = await getRandomSigner(Alice, "10 UNIT");

    //     await expect(
    //         contract.connect(emptyAccount).tx.transfer(sender.address, 7)
    //     ).to.not.emit(contract, "Transfer");
    // });
});
