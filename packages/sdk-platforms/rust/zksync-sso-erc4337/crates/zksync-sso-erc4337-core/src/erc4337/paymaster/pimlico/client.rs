use crate::{
    erc4337::{
        bundler::config::BundlerConfig,
        paymaster::pimlico::models::{
            SponsorshipResponse, SponsorshipResult, UserOperationPreSponsorship,
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

    pub async fn sponsor_user_operation(
        &self,
        user_operation: &UserOperationPreSponsorship,
        entry_point: &Address,
        sponsorship_policy_id: Option<String>,
    ) -> eyre::Result<SponsorshipResult> {
        println!("sponsor_user_operation_v08");

        let bundler_url = self.config.url();
        let params = if let Some(policy_id) = sponsorship_policy_id {
            vec![
                serde_json::to_value(user_operation)?,
                serde_json::to_value(entry_point)?,
                serde_json::json!({ "sponsorshipPolicyId": policy_id }),
            ]
        } else {
            vec![
                serde_json::to_value(user_operation)?,
                serde_json::to_value(entry_point)?,
            ]
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
        let v =
            serde_json::from_str::<JSONRPCResponse<SponsorshipResponse>>(&res)?;

        println!("pm_sponsorUserOperation json: {:?}", v);

        let response: Response<SponsorshipResponse> = v.into();

        let response_estimate = response?;
        let response_estimate = response_estimate.unwrap();

        let result = SponsorshipResult {
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
