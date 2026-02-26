use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::ai_integration::entities::message::ToolCall;

pub mod create_flash_card;

#[async_trait]
pub trait AcceptToolCall: Send + Sync {
    type Args: for<'a> Deserialize<'a> + Send + Sync;

    async fn accept_call(&self, tool_call: &ToolCall, args: Self::Args) -> Result<(), String>;
}

#[async_trait]
pub trait AcceptToolCallFromJson: Send + Sync {
    async fn accept_call(&self, tool_call: &ToolCall, value: Value) -> Result<(), String>;
}

#[async_trait]
impl<T: AcceptToolCall + Send + Sync> AcceptToolCallFromJson for T {
    async fn accept_call(&self, tool_call: &ToolCall, value: Value) -> Result<(), String> {
        let args = serde_json::from_value(value).unwrap();
        <Self as AcceptToolCall>::accept_call(self, tool_call, args).await
    }
}
