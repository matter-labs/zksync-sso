
import { SmartAccount, utils, types, EIP712Signer } from "zksync-ethers";
import { deployFactory } from "./AccountAbstraction"
import { Contract, ethers } from "ethers";
import { promises } from "fs";
import { it } from "mocha";
import { deployContract, getWallet } from "./utils";
import { assert } from "chai";

describe("Spend limit validation", function () {

    // era test node location
    const eraTestNodeRichKey = "0x3d3cbc973389cb26f657686445bcc75662b415b656078503592ac8c1abb8810e"
    const wallet = getWallet(eraTestNodeRichKey);

    it("should deploy module", async () => {
        const passkeyModuleAddress = await deployContract("SessionPasskeySpendLimitModule", [], { wallet });
        assert(passkeyModuleAddress != null, "No module deployed")
    });

    it("should deploy verifier", async () => {
        const expensiveVerifier = await deployContract("P256VerifierExpensive", [], { wallet });
        assert(expensiveVerifier != null, "No verifier deployed")
    });

    it.only("should deploy implemention", async () => {
        const accountImpl = await deployContract("ClaveAccount", [], { wallet });
        assert(accountImpl != null, "No account impl deployed")
    });

    it("should deploy proxy", async () => {
        const accountImpl = await deployContract("ClaveAccount", [], { wallet });
        const accountImplAddress = await accountImpl.getAddress();

        const proxyAccount = await deployContract("ERC7579Account", [accountImplAddress], { wallet });
        assert(proxyAccount != null, "No account proxy deployed")
    });

    it("should deploy account with module", async () => {
        const passkeyModuleContract= await deployContract("SessionPasskeySpendLimitModule", [], { wallet });

        const expensiveVerifierContract = await deployContract("P256VerifierExpensive", [], { wallet });

        const accountImpl = await deployContract("ClaveAccount", [], { wallet });
        const accountImplAddress = await accountImpl.getAddress();
        const proxyAccount = await deployContract("ERC7579Account", [accountImplAddress], { wallet });

        const aaFactory = await deployFactory("AAFactory", wallet)

        // this is a binary object formatted by @simplewebauthn that contains the alg type and public key
        const publicKeyEs256Bytes = new Uint8Array([
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 167, 69,
            109, 166, 67, 163, 110, 143, 71, 60, 77, 232, 220, 7,
            121, 156, 141, 24, 71, 28, 210, 116, 124, 90, 115, 166,
            213, 190, 89, 4, 216, 128, 34, 88, 32, 193, 67, 151,
            85, 245, 24, 139, 246, 220, 204, 228, 76, 247, 65, 179,
            235, 81, 41, 196, 37, 216, 117, 201, 244, 128, 8, 73,
            37, 195, 20, 194, 9
        ]);
        const passkeyAddress = await passkeyModuleContract.getAddress()
        const p265VerifyAddress = await expensiveVerifierContract.getAddress();

        console.log("deployment inputs", publicKeyEs256Bytes, p265VerifyAddress, [passkeyAddress])
        const passkeyTx = await aaFactory.deployProxy7579Account(
            ethers.ZeroHash,
        );
        console.log("what is this", passkeyTx)
        assert(passkeyTx != null, "new account setup failed")

        await passkeyTx.wait();

        assert(passkeyTx != null, "new account deployment failed")
    });

    xit("should set spend limit via module", async () => {
        const passkeyModuleAddress = await deployContract("SessionPasskeySpendLimitModule", [], { wallet });

        const expensiveVerifier = await deployContract("P256VerifierExpensive", [], { wallet });

        const aaFactory = await deployFactory("AAFactory", wallet)

        // this is a binary object formatted by @simplewebauthn that contains the alg type and public key
        const publicKeyEs256Bytes = new Uint8Array([
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 167, 69,
            109, 166, 67, 163, 110, 143, 71, 60, 77, 232, 220, 7,
            121, 156, 141, 24, 71, 28, 210, 116, 124, 90, 115, 166,
            213, 190, 89, 4, 216, 128, 34, 88, 32, 193, 67, 151,
            85, 245, 24, 139, 246, 220, 204, 228, 76, 247, 65, 179,
            235, 81, 41, 196, 37, 216, 117, 201, 244, 128, 8, 73,
            37, 195, 20, 194, 9
        ]);

        const passkeyTx = await aaFactory.deployProxy7579Account(
            ethers.ZeroHash,
            publicKeyEs256Bytes,
            await expensiveVerifier.getAddress(),
            [], // TODO: Make this REAL module and data byte array
        );
        await passkeyTx.wait();

        // send signed transaction to update spend limit on module
        const authenticatorData = 'SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MFAAAABQ'
        const clientData = 'eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiZFhPM3ctdWdycS00SkdkZUJLNDFsZFk1V2lNd0ZORDkiLCJvcmlnaW4iOiJodHRwOi8vbG9jYWxob3N0OjUxNzMiLCJjcm9zc09yaWdpbiI6ZmFsc2UsIm90aGVyX2tleXNfY2FuX2JlX2FkZGVkX2hlcmUiOiJkbyBub3QgY29tcGFyZSBjbGllbnREYXRhSlNPTiBhZ2FpbnN0IGEgdGVtcGxhdGUuIFNlZSBodHRwczovL2dvby5nbC95YWJQZXgifQ'
        const b64SignedChallange = 'MEUCIQCYrSUCR_QUPAhvRNUVfYiJC2JlOKuqf4gx7i129n9QxgIgaY19A9vAAObuTQNs5_V9kZFizwRpUFpiRVW_dglpR2A'

        // get ABI of smart account
        const aaArtifact = JSON.parse(await promises.readFile('artifacts-zk/src/ERC7579Account.sol/ERC7579Account.json', 'utf8'))

        // compare both to see what it's doing
        const smartAccountModule = new Contract(await passkeyModuleAddress.getAddress(), ["function setSpendingLimits(TokenConfig[])"], wallet)
        const smartAccount = new SmartAccount({ address: createdAccount, secret: b64SignedChallange });


        // execute setSpendLimit within the account contract:
        // send a transaction with the sender being the smart account address,
        // and the transaction call data with the setSpendLimit function
        // with the signature of the passkey
        console.log('5555');
        let tx = await smartAccountModule.populateTransaction["setSpendingLimits"](wallet.address);
        let aaTx = {
            ...tx,
            from: createdAccount,
            gasLimit: 1000000,
            gasPrice: await wallet.provider.getGasPrice(),
            chainId: (await wallet.provider.getNetwork()).chainId,
            nonce: await wallet.provider.getTransactionCount(createdAccount),
            type: 113,
            customData: {
                gasPerPubdata: utils.DEFAULT_GAS_PER_PUBDATA_LIMIT,
            } as types.Eip712Meta,
            value: 0,
        }
        console.log('6666');


        const signedTxHash = EIP712Signer.getSignedDigest(aaTx);
        const customSignature = ethers.getBytes(
            ethers.joinSignature(
                wallet._signingKey().signDigest(signedTxHash)
            )
        );

        aaTx.customData = {
            ...aaTx.customData,
            customSignature
        };

        const serializedTx = utils.serialize(aaTx);
        console.log('7777');
        let txResponse = await provider.sendTransaction(serializedTx);
        const populatedSmartAccountTx = await smartAccount.populateTransaction({
            type: utils.EIP712_TX_TYPE,
            customData: {
                customSignature: b64SignedChallange,
            },

            /*
            selector: "setSpendLimit"
            publicKey: publicKeyEs256Bytes,
            limit: 1000,
            token: "0xAe045DE5638162fa134807Cb558E15A3F5A7F853",
            */
        });
        console.log('8888');

        // a transaction that adds a passkey
    })
})