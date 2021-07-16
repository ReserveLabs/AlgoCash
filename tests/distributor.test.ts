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
        const Bob = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"; // Bob Address
        const Dave = "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy"; // Dave Address

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

        api.tx.balances.transfer(Dave, '10000000000')

        return { Alice, Bob, Dave, alc, ausd, distributor };
    }

    it("Distributor operator", async () => {
        console.log("begin test");
        const { Alice, Bob, Dave, alc, ausd, distributor } = await setup();
        const decimal = 10000000000;    
        await ausd.tx.mint(Bob, 100*decimal);
        await ausd.tx.mint(Dave, 100*decimal);

        const bob_result = await ausd.query.balanceOf(Bob);
        expect(bob_result.output).to.equal(100*decimal);

        const dave_result = await ausd.query.balanceOf(Dave);
        expect(dave_result.output).to.equal(100*decimal);
    
        await alc.mint(distributor.address, 10000*decimal);
        const distributor_balance_result = await alc.query.balanceOf(distributor.address);
        expect(distributor_balance_result.output).to.equal(10000*decimal);

        const ausd_bob = ausd.connect(Bob);
        await ausd_bob.tx.approve(distributor.address, 100*decimal);

        const ausd_dave = ausd_bob.connect(Dave);
        await ausd.tx.approve(distributor.address, 100*decimal);    
        
        console.log("bob deposit");
        await distributor.tx.deposit(Bob, 100*decimal);
        console.log("after bob deposit");
        const bob_ausd_result = await ausd.query.balanceOf(Bob);
        console.log("after bob deposit2");
        expect(bob_ausd_result.output).to.equal(0);

        console.log("dave deposit");
        await distributor.tx.deposit(Dave, 100*decimal);
        const dave_ausd_result = await ausd.query.balanceOf(Dave);
        expect(dave_ausd_result.output).to.equal(0);   

        const distributor_result = await ausd.query.balanceOf(distributor.address);
        expect(distributor_result.output).to.equal(200*decimal);

        console.log("distribute  Alc");
        await distributor.tx.distributeAlc([{user:Bob, amount:500*decimal}, {user:Dave, amount:500*decimal}]);

        const bob_alc_result = await alc.query.balanceOf(Bob);
        expect(bob_alc_result.output).to.equal(500*decimal);

        const dave_alc_result = await alc.query.balanceOf(Dave);
        expect(dave_alc_result.output).to.equal(500*decimal);

        const distributor_alc_result = await alc.query.balanceOf(distributor.address);
        expect(distributor_alc_result.output).to.equal(9000*decimal);
    });
});
