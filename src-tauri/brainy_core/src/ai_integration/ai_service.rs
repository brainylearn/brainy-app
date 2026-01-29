use langchain_rust::{language_models::llm::LLM, schemas::Message};
use serde::Serialize;
use tokio_stream::StreamExt;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamLlmResponseEvent {
    InProgress(String),
    Finished,
    Error(String),
}

pub struct AiService {
    llm: Box<dyn LLM>,
}

impl AiService {
    pub fn new(llm: Box<dyn LLM>) -> Self {
        Self { llm }
    }

    pub async fn stream<F>(&self, messages: &[Message], on_event: F) -> Result<(), String>
    where
        F: Fn(StreamLlmResponseEvent) -> Result<(), String>,
    {
        let mut stream = match self.llm.stream(messages).await {
            Ok(stream) => stream,
            Err(err) => return Err(err.to_string()),
        };

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => {
                    on_event(StreamLlmResponseEvent::InProgress(data.content))?;
                }
                Err(err) => {
                    on_event(StreamLlmResponseEvent::Error(err.to_string()))?;
                    break;
                }
            };
        }

        on_event(StreamLlmResponseEvent::Finished)?;

        Ok(())
    }

    pub async fn generate(&self, messages: &[Message]) -> Result<String, String> {
        match self.llm.generate(messages).await {
            Ok(response) => Ok(response.generation),
            Err(err) => Err(err.to_string()),
        }
    }
}
