use alloy::{primitives::Bytes, sol, sol_types::SolCall};

sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IPaymaster {
        function general(bytes calldata input) external nonpayable;
        function approvalBased(
            address _token,
            uint256 _minAllowance,
            bytes calldata _innerInput
        ) external nonpayable;
    }
}

pub fn generate_paymaster_input(inner_input: Option<Bytes>) -> Bytes {
    let inner_input = inner_input.unwrap_or_default();

    let general_call = IPaymaster::generalCall { input: inner_input };

    let encoded_bytes = general_call.abi_encode();

    encoded_bytes.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_paymaster_input_empty() -> eyre::Result<()> {
        let result = generate_paymaster_input(None);

        assert!(!result.is_empty(), "Result should not be empty");
        assert!(
            result.len() >= 4,
            "Result should have at least the function selector"
        );

        Ok(())
    }

    #[test]
    fn test_generate_paymaster_input_with_data() -> eyre::Result<()> {
        let input = Bytes::from_static(&[1, 2, 3, 4]);
        let result = generate_paymaster_input(Some(input));

        assert!(!result.is_empty(), "Result should not be empty");
        assert!(
            result.len() >= 4,
            "Result should have at least the function selector"
        );

        Ok(())
    }
}
