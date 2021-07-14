/*
*   the script of the deployment for all the contracts.
*   1) deploy the util and return the accountid.
*   2) deploy the orcale and return the accountid.
*   3) deploy and asset contract and instantiate the ALC and return the accountid.
*   4) instantiate the ALB and return the accountid.
*   5) instantiate the ALS and return the accountid.
*   6) mock the aUsd for testing by instantiate the aUsd and return the accountid.
*   7) deploy the distributor and return the accountid.
*   8) deploy the boardroom and return the accountid.
*   9) deploy the treasury and return the accountid.
*/

import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

async function run() {
    await api.isReady;

    // The redspot signer supports passing in an address. If you want to use  substrate uri, you can do it like this:
    // const signer = createSigner(keyring.createFromUri("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"));
    // Or get the configured account from redspot config:
    // const signer = (await getSigners())[0]
    const signer = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"; // Alice Address
    const balance = await api.query.system.account(signer);

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the util: ');
    const utilFactory = await getContractFactory('util', signer);
    const tsForUtil = (new Date().getTime()).toString();
    const util = await utilFactory.deploy('new', '10', {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForUtil
    });
    console.log(
        'Deploy util successfully. The contract address: ',
        util.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the oracle: ');
    const orcaleFactory = await getContractFactory('oracle', signer);
    const tsForOracle = (new Date().getTime()).toString();
    const oracle = await orcaleFactory.deploy('new', {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForOracle
    });

    console.log(
        'Deploy oracle successfully. The contract address: ',
        oracle.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the ALC: ');
    const alcFactory = await getContractFactory('asset', signer);
    const tsForAlc = (new Date().getTime()).toString();
    const alc = await alcFactory.deploy('new', '0', 'ALC', 'ALC', '10', {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForAlc
    });

    console.log(
        'Deploy ALC successfully. The contract address: ',
        alc.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the ALB: ');
    const albFactory = await getContractFactory('asset', signer);
    const tsForAlb = (new Date().getTime()).toString();
    const alb = await albFactory.deploy('new', '0', 'ALB', 'ALB', '10', {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForAlb
    });

    console.log(
        'Deploy ALB successfully. The contract address: ',
        alb.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the ALS: ');
    const alsFactory = await getContractFactory('asset', signer);
    const tsForAls = (new Date().getTime()).toString();
    const als = await alsFactory.deploy('new', '0', 'ALS', 'ALS', '10', {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForAls
    });

    console.log(
        'Deploy ALS successfully. The contract address: ',
        als.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the mocked AUSD: ');
    const ausdFactory = await getContractFactory('asset', signer);
    const tsForAusd = (new Date().getTime()).toString();
    const ausd = await ausdFactory.deploy('new', '100000', 'AUSD', 'AUSD', '10', {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForAusd
    });

    console.log(
        'Deploy AUSD successfully. The contract address: ',
        ausd.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the distributor: ');
    const distributorFactory = await getContractFactory('distributor', signer);
    const tsForDistributor = (new Date().getTime()).toString();
    const distributor = await distributorFactory.deploy('new', alc.address, ausd.address, {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForDistributor
    });

    console.log(
        'Deploy distributor successfully. The contract address: ',
        distributor.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the boardroom: ');
    const boardroomFactory = await getContractFactory('boardroom', signer);
    const tsForBoardroom = (new Date().getTime()).toString();
    const boardroom = await boardroomFactory.deploy('new', alc.address, als.address, util.address, {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForBoardroom
    });

    console.log(
        'Deploy boardroom successfully. The contract address: ',
        boardroom.address.toString()
    );

    console.log('-----------------------------------');
    console.log('Balance: ', balance.toHuman());
    console.log('deploy the treasury: ');
    const treasuryFactory = await getContractFactory('treasury', signer);
    const tsForTreasury = (new Date().getTime()).toString();
    const treasury = await treasuryFactory.deploy('new', util.address, alc.address, alb.address, als.address, oracle.address, boardroom.address, {
        gasLimit: "400000000000",
        value: "1000000000000",
        salt: tsForTreasury
    });

    console.log(
        'Deploy treasury successfully. The contract address: ',
        treasury.address.toString()
    );

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
