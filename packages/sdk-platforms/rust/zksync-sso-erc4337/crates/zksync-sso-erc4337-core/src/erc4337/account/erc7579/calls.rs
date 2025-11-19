use crate::erc4337::account::erc7579::contract::{
    account::IERC7579Account::executeCall,
    execution::Execution as ContractExecution,
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, U256},
    sol,
    sol_types::{SolCall, SolType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub target: Address,
    pub value: U256,
    pub call_data: Bytes,
}

impl From<Execution> for ContractExecution {
    fn from(execution: Execution) -> Self {
        ContractExecution {
            target: execution.target,
            value: execution.value,
            callData: execution.call_data,
        }
    }
}

pub fn encode_execution_call_data(
    mode: FixedBytes<32>,
    execution_call_data: Bytes,
) -> Bytes {
    executeCall { mode, executionCalldata: execution_call_data }
        .abi_encode()
        .into()
}

pub fn encoded_call_with_target_and_data(
    target: Address,
    data: Bytes,
) -> Bytes {
    encoded_call_data(target, Some(data), None)
}

pub fn encoded_call_data(
    target: Address,
    data: Option<Bytes>,
    value: Option<U256>,
) -> Bytes {
    let call = {
        let value = value.unwrap_or(U256::from(0));
        let call_data = data.unwrap_or_default();
        Execution { target, value, call_data }
    };

    let calls = vec![call];
    encode_calls(calls).into()
}

pub fn encode_calls(calls: Vec<Execution>) -> Vec<u8> {
    if calls.clone().len() == 1 {
        single_call(&calls[0])
    } else {
        multi_call(calls)
    }
}

fn single_call(call: &Execution) -> Vec<u8> {
    let mode = mode_code_single();
    let execution_calldata: Bytes = [
        call.target.as_slice(),
        call.value
            .as_le_bytes()
            .iter()
            .rev()
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
        call.call_data.to_vec().as_slice(),
    ]
    .concat()
    .into();

    executeCall { mode, executionCalldata: execution_calldata }.abi_encode()
}

fn multi_call(calls: Vec<Execution>) -> Vec<u8> {
    ERC7579AccountExecute::new(calls).encode()
}

sol! {
    struct ExecutionParams {
        ContractExecution[] executions;
    }
}

struct ERC7579AccountExecute(executeCall);

impl ERC7579AccountExecute {
    pub fn new(executions: Vec<Execution>) -> Self {
        let mode = mode_code();
        let execution_calldata = {
            let exectution_params = ExecutionParams {
                executions: executions.into_iter().map(Into::into).collect(),
            };
            let execution_bytes =
                <ExecutionParams as SolType>::abi_encode_params(
                    &exectution_params,
                );
            Bytes::from(execution_bytes)
        };
        Self(executeCall { mode, executionCalldata: execution_calldata })
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

fn mode_code_single() -> FixedBytes<32> {
    FixedBytes::from([0u8; 32])
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{U256, address, hex};

    #[test]
    fn test_encode_call() -> eyre::Result<()> {
        let expected_encoded_calls = {
            let expected_encoded_calls_hex = "e9ae5c53000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000034d5b7e333f346c92b6f6355ac62cc3f0ffa882bc30000000000000000000000000000000000000000000000000000000000000001000000000000000000000000";
            hex::decode(expected_encoded_calls_hex)?
        };

        let target = address!("0xd5b7e333f346c92b6f6355ac62cc3f0ffa882bc3");

        let calls = vec![Execution {
            target,
            value: U256::from(1),
            call_data: Bytes::default(),
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

    #[test]
    fn test_encode_calls() -> eyre::Result<()> {
        let expected_encoded_calls = {
            let expected_encoded_calls_hex = "e9ae5c5301000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000d5b7e333f346c92b6f6355ac62cc3f0ffa882bc3000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d5b7e333f346c92b6f6355ac62cc3f0ffa882bc3000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000";
            hex::decode(expected_encoded_calls_hex)?
        };

        let target = address!("0xd5b7e333f346c92b6f6355ac62cc3f0ffa882bc3");

        let calls = vec![
            Execution {
                target,
                value: U256::from(1),
                call_data: Bytes::default(),
            },
            Execution {
                target,
                value: U256::from(2),
                call_data: Bytes::default(),
            },
        ];

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
