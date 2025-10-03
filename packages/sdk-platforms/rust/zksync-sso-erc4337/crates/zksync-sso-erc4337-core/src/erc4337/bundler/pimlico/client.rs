use super::gas_price::GasPrice;
use crate::{
    erc4337::bundler::{config::BundlerConfig, pimlico::estimate::Estimate},
    erc4337::entry_point::PackedUserOperation,
    jsonrpc::{JSONRPCResponse, Request, Response},
};
use alloy::{
    primitives::{Address, FixedBytes},
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
};
use eyre::Ok;
use serde_json;

pub struct BundlerClient {
    client: reqwest::Client,
    pub(crate) config: BundlerConfig,
}

impl BundlerClient {
    pub fn new(config: BundlerConfig) -> Self {
        Self { client: reqwest::Client::new(), config }
    }

    pub async fn get_user_operation_receipt(
        &self,
        // user_op_hash: FixedBytes<32>,
        user_op_hash: String,
    ) -> eyre::Result<()> {
        let bundler_url = self.config.url().clone();

        // let user_op_hash_str: String = user_op_hash.to_string();

        let user_op_hash_param: serde_json::Value = user_op_hash.into();

        println!("user_op_hash_param: {}", user_op_hash_param);

        let params = vec![user_op_hash_param];

        println!("\nparams: {:#?}", params);

        let send_body = crate::jsonrpc::Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "eth_getUserOperationReceipt".into(),
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

        // let raw_payload =
        //     serde_json::from_str::<JSONRPCResponse<String>>(&response_text)?;
        // println!("raw_payload: {:?}", raw_payload);

        // {
        //   "jsonrpc": "2.0",
        //   "id": 1,
        //   "result": {
        //         "entryPoint": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        //         "userOpHash": "0xa5a579c6fd86c2d8a4d27f5bb22796614d3a31bbccaba8f3019ec001e001b95f",
        //         "sender": "0x8C6bdb488F664EB98E12cB2671fE2389Cc227D33",
        //         "nonce": "0x18554d9a95404c5e8ac591f8608a18f80000000000000000",
        //         "actualGasUsed": "0x7f550",
        //         "actualGasCost": "0x4b3b147f788710",
        //         "success": true,
        //         "logs": [
        //             // ...
        //         ],
        //         "receipt": {
        //             "transactionHash": "0x57465d20d634421008a167cfcfcde94847dba9d6b5d3652b071d4b84e5ce74ff",
        //             "transactionIndex": "0x20",
        //             "blockHash": "0xeaeec1eff4095bdcae44d86574cf1bf08b14b26be571b7c2290f32f9f250c103",
        //             "blockNumber": "0x31de70e",
        //             "from": "0x43370996A3Aff7B66B3AC7676dD973d01Ecec039",
        //             "to": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        //             "cumulativeGasUsed": "0x4724c3",
        //             "gasUsed": "0x7ff5a",
        //             "address": null,
        //             "logs": [
        //                 // ...
        //             ],
        //             "logsBloom": "0x010004000800020000000040000000000000040000000000000010000004000000080000001000000212841100000000041080000000000020000240000000000800000022001000400000080000028000040000000000200001000010000000000000000a0000000000000000800800000000004110004080800110282000000000000002000000000000000000000000000200000400000000000000240040200002000000000000400000000002000140000000000000000002200000004000000002000000000021000000000000000000000000800080108020000020000000080000000000000000000000000000000000000000000108000000102000",
        //             "status": "0x1",
        //             "effectiveGasPrice": "0x89b098f46"
        //         }
        //     }
        // }

        Ok(())
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

    pub async fn estimate_user_operation_gas_price(
        &self,
    ) -> eyre::Result<GasPrice> {
        println!("estimate_user_operation_gas_price");

        let bundler_url = self.config.url().clone();

        let req_body = Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "pimlico_getUserOperationGasPrice".into(),
            params: [] as [(); 0],
        };
        println!("req_body: {:?}", serde_json::to_string(&req_body)?);

        let post = self
            .client
            .post(bundler_url.as_str())
            .json(&req_body)
            .send()
            .await?;
        println!("pimlico_getUserOperationGasPrice post: {:?}", post);
        let res = post.text().await?;
        println!("pimlico_getUserOperationGasPrice res: {:?}", res);
        let v = serde_json::from_str::<JSONRPCResponse<GasPrice>>(&res)?;

        println!("pimlico_getUserOperationGasPrice json: {:?}", v);

        let response: Response<GasPrice> = v.into();

        let response_estimate = response?;
        let response_estimate = response_estimate.unwrap();

        Ok(response_estimate)
    }

    pub async fn estimate_user_operation_gas(
        &self,
        user_operation: &AlloyPackedUserOperation,
        entry_point: &Address,
    ) -> eyre::Result<Estimate> {
        println!("eth_estimateUserOperationGas");

        let bundler_url = self.config.url().clone();

        let params = vec![
            serde_json::to_value(user_operation)?,
            entry_point.to_string().into(),
        ];

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
        let response_estimate = response_estimate.unwrap();

        Ok(response_estimate)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::gas_price::{GasPrice, GasPriceItem},
        *,
    };
    use alloy::primitives::U256;
    use eyre::ensure;

    pub async fn setup_gas_estimation_bundler_mock()
    -> eyre::Result<BundlerClient> {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        let url = mock_server.uri().to_string();

        let expected_request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "pimlico_getUserOperationGasPrice",
            "params": [],
        });

        let response_gas_price = GasPrice {
            slow: GasPriceItem {
                max_fee_per_gas: U256::from(100000),
                max_priority_fee_per_gas: U256::from(100000),
            },
            standard: GasPriceItem {
                max_fee_per_gas: U256::from(100000),
                max_priority_fee_per_gas: U256::from(100000),
            },
            fast: GasPriceItem {
                max_fee_per_gas: U256::from(100000),
                max_priority_fee_per_gas: U256::from(100000),
            },
        };

        let response_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": response_gas_price,
        });

        let response = ResponseTemplate::new(200).set_body_json(response_body);

        use wiremock::matchers::body_partial_json;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(&expected_request_body))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let bundler_client = BundlerClient::new(BundlerConfig::new(url));

        Ok(bundler_client)
    }

    #[tokio::test]
    async fn test_estimate_user_operation_gas_price() -> eyre::Result<()> {
        let bundler_client = setup_gas_estimation_bundler_mock().await?;

        let gas_price =
            bundler_client.estimate_user_operation_gas_price().await?;

        ensure!(gas_price.fast.max_fee_per_gas.to_string() == "100000");

        Ok(())
    }
}
