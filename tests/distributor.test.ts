import { expect } from "chai";
import { artifacts, network, patract } from "redspot";

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

// BOB: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty

describe("ERC20", () => {
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

        const ausdFactory = await getContractFactory("asset", Alice);
        const tsForAusd = (new Date().getTime()).toString();
        const ausd = await ausdFactory.deploy('new', '0', 'AUSD', 'AUSD', '10', {
            gasLimit: "400000000000",
            value: "1000000000000",
            salt: tsForAusd
        });

        const distributorFactory = await getContractFactory("distributor", Alice);
        const distributor = await distributorFactory.deploy("new", alc.address, ausd.address);

        return { Alice, Bob, alc, ausd, distributor };
    }

    it("Distributor deposit ausd", async () => {
        console.log("begin test");
        const { Alice, Bob, alc, ausd, distributor } = await setup();
        const decimal = 10000000000;    
        await ausd.tx.mint(Alice, 100*decimal);
        await ausd.tx.mint(Bob, 100*decimal);

        const alice_result = await ausd.query.balanceOf(Alice);
        expect(alice_result.output).to.equal(100*decimal);

        const bob_result = await ausd.query.balanceOf(Bob);
        expect(bob_result.output).to.equal(100*decimal);
    
        await alc.mint(distributor.address, 10000*decimal);
        const distributor_balance_result = await alc.query.balanceOf(distributor.address);
        expect(distributor_balance_result.output).to.equal(10000*decimal);

        ausd.tx.approve(distributor.address, 100*decimal);

        const ausd_bob = ausd.connect(Bob);
        await ausd_bob.tx.approve(distributor.address, 100*decimal);

        console.log("alice deposit");
        await distributor.tx.depositToken(100*decimal);
        console.log("alice deposit finished");
        const alice_ausd_result = await ausd.query.balanceOf(Alice);
        expect(alice_ausd_result.output).to.equal(0);

        console.log("bob deposit");
        const distributor_bob = distributor.connect(Bob);
        await distributor_bob.tx.depositToken(100*decimal);
        console.log("bob deposit finished");
        const bob_ausd_result = await ausd.query.balanceOf(Bob);
        expect(bob_ausd_result.output).to.equal(0);

        const distributor_result = await ausd.query.balanceOf(distributor.address);
        expect(distributor_result.output).to.equal(200*decimal);

        console.log("distribute  Alc");
        await distributor.tx.distributeAlc([{user:Bob, amount:500*decimal}, {user:Alice, amount:500*decimal}]);
        console.log("distribute  Alc finished");

        const alice_alc_result = await alc.query.balanceOf(Alice);
        expect(alice_alc_result.output).to.equal(500*decimal);

        const bob_alc_result = await alc.query.balanceOf(Bob);
        expect(bob_alc_result.output).to.equal(500*decimal);

        const distributor_alc_result = await alc.query.balanceOf(distributor.address);
        expect(distributor_alc_result.output).to.equal(9000*decimal);
    });
});
