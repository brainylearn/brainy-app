use rig::{
    completion::{self, CompletionError, CompletionModel, CompletionRequest},
    message::ToolCall,
    providers::ollama,
    streaming::{
        self, RawStreamingChoice, RawStreamingToolCall, StreamedAssistantContent,
        StreamingCompletionResponse,
    },
};
use tokio_stream::StreamExt;

use crate::ai_integration::clients::multi_completion_client::{
    MultiCompletionClient, multi_response::MultiResponse,
    multi_streaming_response::MultiStreamingResponse,
};

#[derive(Clone)]
pub enum MultiCompletionModel {
    Ollama(ollama::CompletionModel),
}

impl CompletionModel for MultiCompletionModel {
    type Response = MultiResponse;

    type StreamingResponse = MultiStreamingResponse;

    type Client = MultiCompletionClient;

    fn make(client: &Self::Client, model: impl Into<String>) -> Self {
        match client {
            MultiCompletionClient::Ollama(client) => {
                MultiCompletionModel::Ollama(ollama::CompletionModel::make(client, model))
            }
        }
    }

    async fn completion(
        &self,
        request: rig::completion::CompletionRequest,
    ) -> Result<completion::CompletionResponse<Self::Response>, CompletionError> {
        match self {
            Self::Ollama(completion_model) => {
                completion_model.completion(request).await.map(|val| {
                    completion::CompletionResponse {
                        choice: val.choice,
                        usage: val.usage,
                        raw_response: val.raw_response.into(),
                    }
                })
            }
        }
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<streaming::StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
    {
        let stream = match self {
            Self::Ollama(completion_model) => completion_model.stream(request).await?,
        };
        let mapped_stream = Box::pin(stream.map(|result| result.map(to_raw_streaming_choice)));
        Ok(StreamingCompletionResponse::stream(mapped_stream))
    }
}

fn to_raw_streaming_choice<R>(
    content: StreamedAssistantContent<R>,
) -> RawStreamingChoice<MultiStreamingResponse>
where
    R: Into<MultiStreamingResponse>,
{
    match content {
        StreamedAssistantContent::Text(text) => RawStreamingChoice::Message(text.text),
        StreamedAssistantContent::ReasoningDelta { id, reasoning } => {
            RawStreamingChoice::ReasoningDelta { id, reasoning }
        }
        StreamedAssistantContent::Reasoning(reasoning) => RawStreamingChoice::Reasoning {
            id: reasoning.id,
            reasoning: reasoning.reasoning[0].clone(),
            signature: reasoning.signature,
        },
        StreamedAssistantContent::ToolCallDelta { id, content } => {
            RawStreamingChoice::ToolCallDelta { id, content }
        }
        StreamedAssistantContent::ToolCall(tool_call) => {
            RawStreamingChoice::ToolCall(to_raw_streaming_call(tool_call))
        }
        StreamedAssistantContent::Final(response) => {
            RawStreamingChoice::FinalResponse(response.into())
        }
    }
}

fn to_raw_streaming_call(tool_call: ToolCall) -> RawStreamingToolCall {
    RawStreamingToolCall {
        id: tool_call.id,
        call_id: tool_call.call_id,
        name: tool_call.function.name,
        arguments: tool_call.function.arguments,
        signature: tool_call.signature,
        additional_params: tool_call.additional_params,
    }
}
