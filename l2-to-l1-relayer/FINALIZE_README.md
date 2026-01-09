# Finalize Your L2-to-L1 Transaction

## ✅ Transaction Ready to Finalize

Your transaction has been successfully verified and is ready for L1 execution!

**L2 Transaction**:
`0xb20b44bf8761c6592d3e367b0633ccbf038c123d7e40b9b623095abeb2273de4`

**Status**:

- ✅ L2-to-L1 logs found
- ✅ Merkle proof obtained (Batch: 43760, Index: 0)
- ✅ Transaction finalized and ready for L1 execution
- ⏳ Waiting for executor to have Sepolia ETH

## 🚨 Action Required: Fund Executor

The executor address needs Sepolia ETH to pay for gas:

**Executor Address**: `0x18ac402d33706c303cD559FA6B2F2f14Fae75307`

### Get Sepolia ETH

1. **Alchemy Faucet** (Recommended):

   - Visit: <https://www.alchemy.com/faucets/ethereum-sepolia>
   - Enter address: `0x18ac402d33706c303cD559FA6B2F2f14Fae75307`
   - Receive ~0.1 ETH

2. **Alternative Faucet**:
   - Visit: <https://sepoliafaucet.com/>
   - Follow instructions to get test ETH

**How much?**: 0.01 ETH is enough for one execution (~0.006 ETH gas cost)

## 🚀 Execute After Funding

Once you have Sepolia ETH in the executor address, run:

```bash
cd /Users/ra/Work/ZkSync/ZKSync-SSO/l2-to-l1-relayer
node finalize-tx.js
```

You should see:

```
🚀 Finalizing L2-to-L1 Message

📍 L2 TX: 0xb20b44bf8761c6592d3e367b0633ccbf038c123d7e40b9b623095abeb2273de4
📍 L1 Interop Handler: 0xB0dD4151fdcCaAC990F473533C15BcF8CE10b1de

💼 Executor: 0x18ac402d33706c303cD559FA6B2F2f14Fae75307

📜 Step 1: Getting receipt with L2-to-L1 logs...
   ✅ Found 1 L2-to-L1 log(s)

🔐 Step 2: Fetching Merkle proof...
   ✅ Proof obtained
   Batch: 43760, Index: 0

📦 Step 3: Building execution parameters...
   Chain ID: 8022833
   Batch: 43760
   Message Index: 0
   Sender: 0x4337084d9e255ff0702461cf8895ce9e3b5ff108

🔗 Step 4: Executing on L1...
   Calling L1InteropHandler.receiveInteropFromL2()...
   ✅ L1 TX submitted: 0x...
   🔗 Etherscan: https://sepolia.etherscan.io/tx/0x...

⏳ Step 5: Waiting for L1 confirmation...
   ✅ L1 execution confirmed in block ...

🎉 Success! L2-to-L1 message executed on L1
```

## 📊 What Happens on L1

When the transaction executes:

1. **Shadow Account Deployment**: Your shadow account on L1 gets deployed (if
   first time)

   - Address: Deterministic based on your L2 account

2. **Bundle Execution**: Shadow account executes the Aave deposit:

   - Calls Aave WETH Gateway
   - Deposits 0.01 ETH into Aave Pool
   - Receives aWETH tokens

3. **Check Aave Balance**: After execution, check your aave balance in the
   wallet app

## 🔍 Monitoring

### Check Executor Balance

```bash
cast balance 0x18ac402d33706c303cD559FA6B2F2f14Fae75307 --rpc-url https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU
```

### View L2 Transaction

```
https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev/tx/0xb20b44bf8761c6592d3e367b0633ccbf038c123d7e40b9b623095abeb2273de4
```

### View L1 Execution

After running `finalize-tx.js`, you'll get an Etherscan link like:

```
https://sepolia.etherscan.io/tx/0x...
```

## ❓ Troubleshooting

### Error: "gas required exceeds allowance (0)"

**Cause**: Executor has no Sepolia ETH

**Solution**: Fund the executor address (see above)

### Error: "Could not get L2-to-L1 proof"

**Cause**: Transaction not finalized yet (needs ~15 minutes from L2 transaction
time)

**Solution**: Wait and try again later

### Error: "Execution reverted"

**Possible causes**:

1. Message already executed (check Etherscan for previous execution)
2. Invalid proof (should not happen - proof was verified)
3. Contract issue (check Etherscan for revert reason)

## 📁 Files

- `finalize-tx.js` - Script to execute L2-to-L1 message on L1
- `index.js` - Full relayer service (for automated execution)
- `test-tx.js` - Test script to verify transaction structure

## 🎯 Next Steps

1. ✅ Fund executor address with Sepolia ETH
2. ✅ Run `node finalize-tx.js`
3. ✅ Wait for L1 confirmation
4. ✅ Check Aave balance in wallet app

---

**Summary**: Your L2 transaction is finalized and ready. Just need to add
Sepolia ETH to the executor address, then run the finalize script!
