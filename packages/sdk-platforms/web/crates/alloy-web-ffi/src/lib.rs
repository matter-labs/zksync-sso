use alloy_web_core::{erc4337 as core_erc4337, signer, transaction, utils};
use wasm_bindgen::prelude::*;

mod account;
mod erc4337;

pub use account::*;
pub use erc4337::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    // Initialize tracing to use browser console
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // No ANSI colors in browser console
        .without_time() // Disable time (not supported in WASM without extra features)
        .with_writer(tracing_web::MakeConsoleWriter); // Note: no ::default()

    tracing_subscriber::registry().with(fmt_layer).init();
}

#[wasm_bindgen]
pub struct AlloyWeb;

impl Default for AlloyWeb {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl AlloyWeb {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        AlloyWeb
    }

    #[wasm_bindgen]
    pub fn encode_rlp(&self, data: JsValue) -> Result<String, JsValue> {
        let bytes: Vec<u8> = serde_wasm_bindgen::from_value(data)?;
        utils::encode_rlp(bytes).map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn decode_rlp(&self, hex_data: &str) -> Result<JsValue, JsValue> {
        let decoded = utils::decode_rlp(hex_data).map_err(|e| JsValue::from_str(&e))?;
        Ok(serde_wasm_bindgen::to_value(&decoded)?)
    }

    #[wasm_bindgen]
    pub fn create_address(&self, address: &str) -> Result<String, JsValue> {
        utils::create_address(address).map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn parse_u256(&self, value: &str) -> Result<String, JsValue> {
        utils::parse_u256(value).map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub async fn create_transaction_request(&self, params: JsValue) -> Result<JsValue, JsValue> {
        let params: transaction::TxParams = serde_wasm_bindgen::from_value(params)?;
        let result = transaction::create_transaction_request(params)
            .await
            .map_err(|e| JsValue::from_str(&e))?;

        let js_result = serde_wasm_bindgen::to_value(&result);
        match &js_result {
            Ok(_val) => web_sys::console::log_1(&"Serialization succeeded".to_string().into()),
            Err(e) => web_sys::console::log_1(&format!("Serialization failed: {e:?}").into()),
        }

        js_result.map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn fill_transaction(&self, params: JsValue) -> Result<JsValue, JsValue> {
        let params: transaction::TxParams = serde_wasm_bindgen::from_value(params)?;
        let result = transaction::fill_transaction(params)
            .await
            .map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub async fn send_transaction(&self, params: JsValue) -> Result<JsValue, JsValue> {
        let params: transaction::TxParams = serde_wasm_bindgen::from_value(params)?;
        let result = transaction::send_transaction(params)
            .await
            .map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn create_private_key_signer(&self, private_key: &str) -> Result<String, JsValue> {
        signer::create_private_key_signer(private_key).map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn encode_function_data(
        &self,
        signature: &str,
        params: JsValue,
    ) -> Result<String, JsValue> {
        let params_vec: Vec<String> = serde_wasm_bindgen::from_value(params)?;
        utils::encode_function_data(signature, params_vec).map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn build_user_operation(&self, params: JsValue) -> Result<JsValue, JsValue> {
        let params: core_erc4337::UserOpParams = serde_wasm_bindgen::from_value(params)?;
        let result =
            core_erc4337::build_user_operation(params).map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn build_send_user_operation(&self, params: JsValue) -> Result<JsValue, JsValue> {
        let params: core_erc4337::UserOpParams = serde_wasm_bindgen::from_value(params)?;
        let result =
            core_erc4337::build_send_user_operation(params).map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub fn get_block_tag(tag: &str) -> Result<JsValue, JsValue> {
    let block_tag = utils::get_block_tag(tag).map_err(|e| JsValue::from_str(&e))?;
    Ok(serde_wasm_bindgen::to_value(&block_tag)?)
}
