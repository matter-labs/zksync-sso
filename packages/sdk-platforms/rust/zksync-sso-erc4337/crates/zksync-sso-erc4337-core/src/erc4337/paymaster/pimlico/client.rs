use crate::{
    erc4337::{
        bundler::config::BundlerConfig,
        paymaster::pimlico::models::{
            SponsorshipResponseV07, SponsorshipResultV07,
            UserOperationPreSponsorshipV07,
        },
    },
    jsonrpc::{JSONRPCResponse, Request, Response},
};
use alloy::primitives::Address;
use serde_json;

pub struct PaymasterClient {
    client: reqwest::Client,
    config: BundlerConfig,
}

impl PaymasterClient {
    pub fn new(config: BundlerConfig) -> Self {
        Self { client: reqwest::Client::new(), config }
    }

    pub async fn sponsor_user_operation_v07(
        &self,
        user_operation: &UserOperationPreSponsorshipV07,
        entry_point: &Address,
        sponsorship_policy_id: Option<String>,
    ) -> eyre::Result<SponsorshipResultV07> {
        println!("sponsor_user_operation_v07 ");

        let bundler_url = self.config.url().clone();

        let params: Vec<serde_json::Value> = {
            let user_operation_value = serde_json::to_value(user_operation)?;
            let mut vec: Vec<serde_json::Value> =
                vec![user_operation_value, entry_point.to_string().into()];
            if let Some(sponsorship_policy_id) = sponsorship_policy_id {
                vec.push(sponsorship_policy_id.into());
            }
            vec
        };

        let req_body: Request<Vec<serde_json::Value>> = Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "pm_sponsorUserOperation".into(),
            params,
        };
        println!("req_body: {:?}", serde_json::to_string(&req_body)?);

        let post = self
            .client
            .post(bundler_url.as_str())
            .json(&req_body)
            .send()
            .await?;
        println!("pm_sponsorUserOperation post: {:?}", post);
        let res = post.text().await?;
        println!("pm_sponsorUserOperation res: {:?}", res);
        let v = serde_json::from_str::<JSONRPCResponse<SponsorshipResponseV07>>(
            &res,
        )?;

        println!("pm_sponsorUserOperation json: {:?}", v);

        let response: Response<SponsorshipResponseV07> = v.into();

        let response_estimate = response?;
        let response_estimate = response_estimate.unwrap();

        let result = SponsorshipResultV07 {
            call_gas_limit: response_estimate.call_gas_limit,
            verification_gas_limit: response_estimate.verification_gas_limit,
            pre_verification_gas: response_estimate.pre_verification_gas,
            paymaster: response_estimate.paymaster,
            paymaster_verification_gas_limit: response_estimate
                .paymaster_verification_gas_limit,
            paymaster_post_op_gas_limit: response_estimate
                .paymaster_post_op_gas_limit,
            paymaster_data: response_estimate.paymaster_data,
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use eyre::ensure;

    pub async fn setup_sponsor_user_operation_v07_paymaster_mock()
    -> eyre::Result<PaymasterClient> {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        let url = mock_server.uri().to_string();

        let expected_request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "pm_sponsorUserOperation",
        });

        let sponsorship_payload = SponsorshipResponseV07::mock();

        let response_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": sponsorship_payload,
        });

        let response = ResponseTemplate::new(200).set_body_json(response_body);

        use wiremock::matchers::body_partial_json;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(&expected_request_body))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let bundler_client = PaymasterClient::new(BundlerConfig::new(url));

        Ok(bundler_client)
    }

    #[tokio::test]
    async fn test_sponsor_user_operation_v07() -> eyre::Result<()> {
        let paymaster_client =
            setup_sponsor_user_operation_v07_paymaster_mock().await?;

        let entry_point =
            "0x0000000071727De22E5E9d8BAf0edAc6f37da032".parse::<Address>()?;
        let entry_point_address = entry_point;

        let user_operation = crate::erc4337::user_operation::wrapper::PackedUserOperationWrapper::mock().0;
        let user_operation_pre =
            UserOperationPreSponsorshipV07::from(user_operation);

        let sponsorship_result = paymaster_client
            .sponsor_user_operation_v07(
                &user_operation_pre,
                &entry_point_address,
                None,
            )
            .await?;

        ensure!(sponsorship_result.call_gas_limit.to_string() == "100000");

        Ok(())
    }
}
