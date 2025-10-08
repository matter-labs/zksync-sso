use super::{
    Bundler,
    config::BundlerConfig,
    models::{estimate::Estimate, receipt::UserOperationReceipt},
};
use crate::{
    erc4337::user_operation::UserOperationV08,
    jsonrpc::{JSONRPCResponse, Request, Response},
};
use alloy::{
    primitives::Address,
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
};

#[derive(Clone)]
pub struct BundlerClient {
    client: reqwest::Client,
    config: BundlerConfig,
}

impl BundlerClient {
    pub fn new(config: BundlerConfig) -> Self {
        Self { client: reqwest::Client::new(), config }
    }

    pub async fn send_user_operation(
        &self,
        entry_point_address: alloy::primitives::Address,
        user_op: AlloyPackedUserOperation,
    ) -> eyre::Result<String> {
        let bundler_url = self.config.url().clone();

        let user_op_value = serde_json::to_value(&user_op)?;

        println!("\nuser_op_value: {}", user_op_value);

        let entry_point_address_str = entry_point_address.to_string();

        let entry_point_addr_param = entry_point_address_str.into();

        println!("\nentry_point_addr_param: {}", entry_point_addr_param);

        let params = vec![user_op_value, entry_point_addr_param];

        println!("\nparams: {:#?}", params);

        let send_body = crate::jsonrpc::Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "eth_sendUserOperation".into(),
            params,
        };

        let response = self
            .client
            .post(bundler_url.as_str())
            .json(&send_body)
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("response_text: {:?}", response_text);

        let raw_payload =
            serde_json::from_str::<JSONRPCResponse<String>>(&response_text)?;
        println!("raw_payload: {:?}", raw_payload);

        let response: Response<String> = raw_payload.into();

        let user_operation_hash = response?;

        Ok(user_operation_hash.unwrap())
    }

    pub async fn estimate_user_operation_gas(
        &self,
        entry_point_address: Address,
        user_op: UserOperationV08,
    ) -> eyre::Result<Estimate> {
        println!("user_op: {:?}", user_op);

        let bundler_url = self.config.url().clone();

        use crate::jsonrpc::{JSONRPCResponse, Request, Response};
        use serde_json;

        let value = serde_json::to_value(&user_op).unwrap();

        let params: Vec<serde_json::Value> =
            vec![value, entry_point_address.to_string().into()];

        let req_body = Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "eth_estimateUserOperationGas".into(),
            params,
        };
        println!("req_body: {:?}", serde_json::to_string(&req_body)?);

        let post = self
            .client
            .post(bundler_url.as_str())
            .json(&req_body)
            .send()
            .await?;
        println!("eth_estimateUserOperationGas post: {:?}", post);
        let res = post.text().await?;
        println!("eth_estimateUserOperationGas res: {:?}", res);
        let v = serde_json::from_str::<JSONRPCResponse<Estimate>>(&res)?;

        println!("eth_estimateUserOperationGas json: {:?}", v);

        let response: Response<Estimate> = v.into();

        let response_estimate = response?;

        Ok(response_estimate.unwrap())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl Bundler for BundlerClient {
    async fn get_user_operation_receipt(
        &self,
        hash: String,
    ) -> eyre::Result<Option<UserOperationReceipt>> {
        let bundler_url = self.config.url().clone();

        let hash_value = serde_json::to_value(&hash)?;

        let send_body = Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "eth_getUserOperationReceipt".into(),
            params: vec![hash_value],
        };

        let response = self
            .client
            .post(bundler_url.as_str())
            .json(&send_body)
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("response_text: {:?}", response_text);
        let raw_payload = serde_json::from_str::<
            JSONRPCResponse<UserOperationReceipt>,
        >(&response_text)?;

        println!("raw_payload: {:?}", raw_payload);

        let response: Response<UserOperationReceipt> = raw_payload.into();

        let response_estimate = response?;

        Ok(response_estimate)
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl Bundler for BundlerClient {
    async fn get_user_operation_receipt(
        &self,
        hash: String,
    ) -> eyre::Result<Option<UserOperationReceipt>> {
        let bundler_url = self.config.url().clone();

        let hash_value = serde_json::to_value(&hash)?;

        let send_body = Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "eth_getUserOperationReceipt".into(),
            params: vec![hash_value],
        };

        let response = self
            .client
            .post(bundler_url.as_str())
            .json(&send_body)
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("response_text: {:?}", response_text);
        let raw_payload = serde_json::from_str::<
            JSONRPCResponse<UserOperationReceipt>,
        >(&response_text)?;

        println!("raw_payload: {:?}", raw_payload);

        let response: Response<UserOperationReceipt> = raw_payload.into();

        let response_estimate = response?;

        Ok(response_estimate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::{
        bundler::models::{estimate::Estimate, receipt::UserOperationReceipt},
        user_operation::UserOperationV08,
    };
    use alloy::primitives::{Address, Bytes, U256, address};
    use eyre::ensure;

    pub async fn setup_gas_estimation_bundler_mock()
    -> eyre::Result<BundlerClient> {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        let expected_request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "eth_estimateUserOperationGas",
        });

        let response_estimate = Estimate {
            pre_verification_gas: U256::from(100000),
            verification_gas_limit: U256::from(100000),
            call_gas_limit: U256::from(100000),
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
        };

        let response_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": response_estimate,
        });

        let response = ResponseTemplate::new(200).set_body_json(response_body);

        use wiremock::matchers::body_partial_json;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(&expected_request_body))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let bundler_client = BundlerClient::new(BundlerConfig::new(
            mock_server.uri().to_string(),
        ));

        Ok(bundler_client)
    }

    #[tokio::test]
    #[ignore = "needs local infrastructure to be running"]
    async fn test_estimate_gas() -> eyre::Result<()> {
        let sender = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");

        let entry_point_address =
            address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");

        let bundler_client = setup_gas_estimation_bundler_mock().await?;

        let user_op = {
            let nonce: U256 = U256::from(0);
            let factory: Address = Address::ZERO;
            let factory_data: Bytes = Bytes::new();
            let call_data: Bytes = Bytes::new();
            let call_gas_limit: U256 = U256::from(100000);
            let verification_gas_limit: U256 = U256::from(100000);
            let pre_verification_gas: U256 = U256::from(100000);
            let max_fee_per_gas: U256 = U256::from(100000);
            let max_priority_fee_per_gas: U256 = U256::from(100000);
            let paymaster: Option<Address> = None;
            let paymaster_data: Option<Bytes> = None;
            let signature: Bytes = Bytes::new();

            UserOperationV08 {
                sender,
                nonce,
                factory: factory.into(),
                factory_data: factory_data.into(),
                call_data,
                call_gas_limit,
                verification_gas_limit,
                paymaster_post_op_gas_limit: Some(U256::from(100000)),
                paymaster_verification_gas_limit: Some(U256::from(100000)),
                pre_verification_gas,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                paymaster,
                paymaster_data,
                authorization: None,
                signature,
            }
        };

        let estimate_result = bundler_client
            .estimate_user_operation_gas(entry_point_address, user_op)
            .await?;

        ensure!(estimate_result.call_gas_limit == U256::from(100000));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_operation_receipt() -> eyre::Result<()> {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        let expected_request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "eth_getUserOperationReceipt",
        });

        let user_operation_hash = "0x93c06f3f5909cc2b192713ed9bf93e3e1fde4b22fcd2466304fa404f9b80ff90".to_string();

        let response_payload = UserOperationReceipt::mock();

        let response_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": response_payload,
        });

        let response = ResponseTemplate::new(200).set_body_json(response_body);
        use wiremock::matchers::body_partial_json;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(&expected_request_body))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let bundler_client = BundlerClient::new(BundlerConfig::new(
            mock_server.uri().to_string(),
        ));

        let receipt = bundler_client
            .get_user_operation_receipt(user_operation_hash.clone())
            .await?;

        assert_eq!(receipt, Some(response_payload));

        Ok(())
    }
}
