use super::gas_price::GasPrice;
use crate::erc4337::bundler::config::BundlerConfig;
use eyre::Ok;

pub struct BundlerClient {
    client: reqwest::Client,
    config: BundlerConfig,
}

impl BundlerClient {
    pub fn new(config: BundlerConfig) -> Self {
        Self { client: reqwest::Client::new(), config }
    }

    pub async fn estimate_user_operation_gas_price(
        &self,
    ) -> eyre::Result<GasPrice> {
        println!("estimate_user_operation_gas_price");

        let bundler_url = self.config.url().clone();

        use crate::jsonrpc::{JSONRPCResponse, Request, Response};
        use serde_json;

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
