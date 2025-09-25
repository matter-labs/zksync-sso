use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type Response<T> = Result<Option<T>, ErrorPayload<T>>;

#[derive(Debug, Deserialize, Error)]
pub struct ErrorPayload<T> {
    pub message: String,
    pub data: Option<T>,
    pub code: Option<i64>,
}

impl<T> fmt::Display for ErrorPayload<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorPayload {{ message: {} }}", self.message)
    }
}

#[derive(Debug, Deserialize)]
pub struct JSONRPCResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<ErrorPayload<T>>,
}

impl<T> From<JSONRPCResponse<T>> for Response<T> {
    fn from(val: JSONRPCResponse<T>) -> Self {
        if let Some(result) = val.result {
            return Ok(Some(result));
        }

        if let Some(error) = val.error {
            return Err(error);
        }

        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestWithEmptyParams {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
}
