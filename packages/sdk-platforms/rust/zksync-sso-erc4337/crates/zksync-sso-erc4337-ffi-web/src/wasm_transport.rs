//! WASM-compatible HTTP transport for Alloy using reqwasm
//!
//! This module provides a custom transport implementation that uses the browser's
//! fetch API through reqwasm instead of reqwest, making it compatible with WASM.

use alloy_json_rpc::{RequestPacket, ResponsePacket};
use alloy_transport::{TransportError, TransportErrorKind, TransportFut};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

/// A wrapper that makes a future Send for WASM
/// 
/// This is safe in WASM because JavaScript is single-threaded,
/// so the future will never actually be sent between threads.
struct SendWrapper<F>(F);

// SAFETY: WASM is single-threaded, so this is safe
unsafe impl<F> Send for SendWrapper<F> {}

impl<F: Future> Future for SendWrapper<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We're just forwarding the poll to the inner future
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
    }
}

/// WASM-compatible HTTP transport using reqwasm
#[derive(Debug, Clone)]
pub struct WasmHttpTransport {
    url: String,
}

impl WasmHttpTransport {
    /// Create a new WASM HTTP transport
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl Service<RequestPacket> for WasmHttpTransport {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // WASM transport is always ready (no connection pooling needed)
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let url = self.url.clone();

        Box::pin(SendWrapper(async move {
            // Serialize the JSON-RPC request
            let body = serde_json::to_string(&req)
                .map_err(|e| TransportErrorKind::custom_str(&e.to_string()))?;

            // Create and send request using reqwasm
            let request = reqwasm::http::Request::post(&url)
                .header("Content-Type", "application/json")
                .body(body);

            // Send the request
            let response = request.send().await.map_err(|e| {
                TransportErrorKind::custom_str(&format!(
                    "Request failed: {:?}",
                    e
                ))
            })?;

            // Check response status
            if !response.ok() {
                return Err(TransportErrorKind::custom_str(&format!(
                    "HTTP error: {}",
                    response.status()
                )));
            }

            // Get response body as text
            let text = response.text().await.map_err(|e| {
                TransportErrorKind::custom_str(&format!(
                    "Failed to read response: {:?}",
                    e
                ))
            })?;

            // Parse JSON-RPC response
            let response: ResponsePacket = serde_json::from_str(&text)
                .map_err(|e| {
                    TransportErrorKind::custom_str(&format!(
                        "Failed to parse response: {}",
                        e
                    ))
                })?;

            Ok(response)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_transport_creation() {
        let transport =
            WasmHttpTransport::new("https://sepolia.era.zksync.dev");
        assert_eq!(transport.url, "https://sepolia.era.zksync.dev");
    }
}
