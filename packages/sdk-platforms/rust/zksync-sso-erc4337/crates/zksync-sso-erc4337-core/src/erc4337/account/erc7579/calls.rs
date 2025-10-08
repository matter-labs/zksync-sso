use crate::erc4337::account::erc7579::{Execution, executeCall};
use alloy::{
    primitives::{Bytes, FixedBytes},
    sol,
    sol_types::{SolCall, SolType},
};

pub fn encode_calls(calls: Vec<Execution>) -> Vec<u8> {
    ERC7579AccountExecute::new(calls).encode()
}

sol! {
    struct ExecutionParams {
        Execution[] executions;
    }
}

struct ERC7579AccountExecute(executeCall);

impl ERC7579AccountExecute {
    pub fn new(executions: Vec<Execution>) -> Self {
        let mode = mode_code();
        let execution = {
            let exectution_params = ExecutionParams { executions };
            let execution_bytes =
                <ExecutionParams as SolType>::abi_encode_params(
                    &exectution_params,
                );
            Bytes::from(execution_bytes)
        };
        Self(executeCall { mode, execution })
    }

    pub fn encode(&self) -> Vec<u8> {
        executeCall::abi_encode(&self.0)
    }
}

fn mode_code() -> FixedBytes<32> {
    let mut code = [0u8; 32];
    code[0] = 1;
    FixedBytes::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{U256, address, hex};

    #[test]
    fn test_encode_calls() -> eyre::Result<()> {
        let expected_encoded_calls = {
            let expected_encoded_calls_hex = "0xe9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000d5b7e333f346c92b6f6355ac62cc3f0ffa882bc3000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000";
            hex::decode(expected_encoded_calls_hex)?
        };

        let target = address!("0xd5b7e333f346c92b6f6355ac62cc3f0ffa882bc3");

        let calls = vec![Execution {
            target,
            value: U256::from(1),
            data: Bytes::default(),
        }];

        let encoded = encode_calls(calls);

        eyre::ensure!(
            encoded == expected_encoded_calls,
            "Encoded calls do not match expected. Expected: {}, Actual: {}",
            hex::encode(expected_encoded_calls),
            hex::encode(encoded)
        );

        Ok(())
    }
}
