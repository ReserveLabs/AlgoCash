import { expect } from "chai";
import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

// BOB: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
// ALICE: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
// DAVE: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy
// EVE: 5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw

describe("ERC20", () => {
    after(() => {
        return api.disconnect();
    });

    async function setup() {
        await api.isReady

        const signerAddresses = await getAddresses();
        const Alice = signerAddresses[0];
        const Bob = signerAddresses[2];
        const Dave = signerAddresses[5];
        const Eve = signerAddresses[6];

        const sender = await getRandomSigner(Alice, "100000000000000");

        const alcFactory = await getContractFactory("asset", sender.address);
        const tsForAlc = (new Date().getTime()).toString();
        const alc = await alcFactory.deploy('new', '0', 'ALC', 'ALC', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForAlc
        });

        const ausdFactory = await getContractFactory("asset", sender.address);
        const tsForAusd = (new Date().getTime()).toString();
        const ausd = await ausdFactory.deploy('new', '0', 'AUSD', 'AUSD', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForAusd
        });

        const distributorFactory = await getContractFactory("distributor", sender.address);
        const distributor = await distributorFactory.deploy("new", alc.address, ausd.address);

        const assetAbi = artifacts.readArtifact("asset");
        const distributorAbi = artifacts.readArtifact("distributor");
        
        const receiver = await getRandomSigner();

        return { sender, alcFactory, ausdFactory, distributorFactory, alc, ausd, distributor, assetAbi, distributorAbi, receiver, Alice, Bob, Dave, Eve };
    }

    it("Distributor operator", async () => {
        const { distributor, Alice } = await setup();
        const result = await distributor.query.operator();
        expect(result.output).to.equal(Alice);
    });

    it("Transfer operator account to Bob", async () => {
        const { distributor, Alice, Bob } = await setup();

        await expect(() =>
            distributor.tx.transferOperator(Bob)
        ).to.changeTokenBalance(contract, receiver, 7);

        await expect(() =>
            contract.tx.transfer(receiver.address, 7)
        ).to.changeTokenBalances(contract, [contract.signer, receiver], [-7, 7]);
    });

    it("Transfer emits event", async () => {
        const { contract, sender, receiver } = await setup();

        await expect(contract.tx.transfer(receiver.address, 7))
            .to.emit(contract, "Transfer")
            .withArgs(sender.address, receiver.address, 7);
    });

    it("Can not transfer above the amount", async () => {
        const { contract, receiver } = await setup();

        await expect(contract.tx.transfer(receiver.address, 1007)).to.not.emit(
            contract,
            "Transfer"
        );
    });

    it("Can not transfer from empty account", async () => {
        const { contract, Alice, sender } = await setup();

        const emptyAccount = await getRandomSigner(Alice, "10 UNIT");

        await expect(
            contract.connect(emptyAccount).tx.transfer(sender.address, 7)
        ).to.not.emit(contract, "Transfer");
    });
});
