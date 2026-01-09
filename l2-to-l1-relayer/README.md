# L2-to-L1 Relayer Service

Automated relayer service that monitors L2 `sendBundleToL1` transactions and
executes them on L1 Sepolia after finalization.

## Overview

When users deposit/withdraw via Aave on the passkey wallet app:

1. **L2 Transaction**: User signs transaction calling
   `L2InteropCenter.sendBundleToL1()`
2. **~15 min wait**: L2-to-L1 message gets proven/finalized on L1
3. **L1 Execution**: This relayer automatically calls
   `L1InteropHandler.receiveInteropFromL2()` to execute the bundle

## Architecture

```
┌─────────────────┐
│  Passkey App    │
│  (L2 ZKSync OS) │
└────────┬────────┘
         │
         │ sendBundleToL1()
         │
         ▼
┌─────────────────┐
│ L2InteropCenter │──┐
│  (Emits event)  │  │
└─────────────────┘  │
                     │
         ┌───────────┘
         │
         ▼
┌─────────────────┐
│  This Relayer   │
│  (Monitors L2)  │
└────────┬────────┘
         │
         │ Wait 15 min
         │
         ▼
┌─────────────────┐
│L1InteropHandler │──► Aave on L1
│  (L1 Sepolia)   │
└─────────────────┘
```

## Setup

### 1. Install Dependencies

```bash
cd l2-to-l1-relayer
npm install
```

### 2. Configure Environment

The `.env` file is already configured with:

- Executor private key:
  `0x1cfcab2cf5ad255cb3387f7fdca2651a61377b334a2a3daa4af86eb476369105`
- L2 RPC: ZKSync OS Testnet
- L1 RPC: Sepolia
- L2InteropCenter: `0xc64315efbdcD90B71B0687E37ea741DE0E6cEFac`
- L1InteropHandler: `0xB0dD4151fdcCaAC990F473533C15BcF8CE10b1de`

**Important**: Make sure the executor address has ETH on **L1 Sepolia** to pay
gas fees!

### 3. Fund the Executor

The executor address derived from the private key needs Sepolia ETH:

```bash
# Get executor address (should be displayed when relayer starts)
# Then fund it with Sepolia ETH from a faucet
```

Faucets:

- <https://www.alchemy.com/faucets/ethereum-sepolia>
- <https://sepoliafaucet.com/>

## Running the Relayer

### Development Mode (with auto-reload)

```bash
npm run dev
```

### Production Mode

```bash
npm start
```

## How It Works

### 1. **Event Monitoring**

- Polls L2 every 30 seconds for `BundleSent` events
- Detects new `sendBundleToL1` transactions
- Adds them to an in-memory queue

### 2. **Waiting Period**

- Waits 15 minutes (configurable) for L2-to-L1 finalization
- Shows countdown timer for each pending message

### 3. **L1 Execution**

- After finalization period, calls `L1InteropHandler.receiveInteropFromL2()`
- Includes L2 transaction hash and block number
- Pays gas fees from executor account

### 4. **Status Tracking**

- **Pending**: Waiting for finalization period
- **Executing**: Currently submitting to L1
- **Executed**: Successfully completed on L1
- **Failed**: Execution failed (will retry if temporary error)

## Output Example

```
🤖 L2-to-L1 Relayer Service Starting...
📍 Executor Address: 0x18ac402d33706c303cD559FA6B2F2f14Fae75307
📍 L2 Interop Center: 0xc64315efbdcD90B71B0687E37ea741DE0E6cEFac
📍 L1 Interop Handler: 0xB0dD4151fdcCaAC990F473533C15BcF8CE10b1de
⏱️  Finalization Wait: 15 minutes

🔄 Starting monitoring loop...

🔍 Scanning L2 blocks 48560 to 48600...

📨 New L2-to-L1 message detected!
   Sender: 0xFc4db9ac6984dbfB0cEC1614F4D2D4F39Cf0b028
   L2 TX: 0xabc123...
   Message Hash: 0xdef456...
   ⏳ Scheduled for execution at: 3:45:00 PM

⏳ Pending messages:
   - 0xdef456... (14 min remaining)

[15 minutes later...]

🚀 Executing L2-to-L1 message on L1...
   Message Hash: 0xdef456...
   L2 TX: 0xabc123...
   ✅ L1 Execution TX: 0x789xyz...
   🔗 Etherscan: https://sepolia.etherscan.io/tx/0x789xyz...
   ✅ L1 execution confirmed in block 12345678

📊 Status: 0 pending | 0 executing | 1 executed | 0 failed
```

## Configuration

Edit `.env` to customize:

```bash
# Wait time before execution (default: 15 minutes)
FINALIZATION_WAIT=900000

# How often to check for new messages (default: 30 seconds)
POLL_INTERVAL=30000

# Use custom RPC endpoints
L2_RPC_URL=https://your-zksync-rpc.com
L1_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
```

## Troubleshooting

### "Insufficient funds" error

The executor needs Sepolia ETH. Fund the address shown on startup.

### "Nonce too low" error

The relayer automatically retries after 5 minutes.

### Messages not executing

1. Check that 15 minutes have passed since L2 transaction
2. Verify L1InteropHandler address is correct
3. Check executor has enough Sepolia ETH

### How to verify execution succeeded

1. Check relayer output for "✅ L1 execution confirmed"
2. Visit Etherscan link shown in output
3. Check Aave balance increased (refresh in passkey app)

## Production Deployment

For production, consider:

1. **Persistent Storage**: Store pending messages in a database (Redis,
   PostgreSQL)
2. **Multiple Relayers**: Run multiple instances for redundancy
3. **Better RPC**: Use paid RPC providers (Alchemy, Infura) for reliability
4. **Monitoring**: Add Prometheus metrics and alerting
5. **Gas Management**: Implement dynamic gas price adjustment
6. **Error Handling**: More sophisticated retry logic

## Security Notes

- Private key is stored in `.env` - **DO NOT commit to git**
- Executor only needs enough ETH for gas fees (no large amounts)
- Relayer cannot steal user funds (only executes their signed messages)
- Consider using a dedicated executor key for production

## License

MIT
