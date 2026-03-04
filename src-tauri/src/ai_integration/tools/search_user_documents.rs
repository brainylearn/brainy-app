use std::sync::Arc;

use rig::{
    completion::ToolDefinition,
    tool::Tool,
    vector_store::{VectorSearchRequest, VectorStoreIndex},
};
use rig_lancedb::LanceDbVectorIndex;
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ai_integration::{
    attachment::Attachment, clients::multi_client::multi_embedding_model::MultiEmbeddingModel,
};

#[derive(Deserialize, Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SearchUserDocumentsArgs {
    #[schemars(
        description = "A concise, standalone search query optimized for semantic similarity. Should be rephrased from the user's message to be self-contained — avoid pronouns or references that depend on conversation history"
    )]
    pub query: String,
    #[schemars(description = "Number of chunks to retrieve.")]
    pub top_k: u64,
}

#[derive(Error, Debug)]
pub enum SearchUserDocumentsError {}

pub struct SearchUserDocuments {
    pub index: Arc<LanceDbVectorIndex<MultiEmbeddingModel>>,
}

impl SearchUserDocuments {
    pub fn new(index: Arc<LanceDbVectorIndex<MultiEmbeddingModel>>) -> Self {
        Self { index }
    }
}

impl Tool for SearchUserDocuments {
    const NAME: &'static str = "search_user_documents";

    type Error = SearchUserDocumentsError;
    type Args = SearchUserDocumentsArgs;
    type Output = Vec<Attachment>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let parameters = serde_json::to_value(schema_for!(SearchUserDocumentsArgs)).unwrap();

        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search the user's uploaded files for relevant information. Use when the answer likely exists in their documents."
                .to_string(),
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let request = VectorSearchRequest::builder()
            .query(args.query)
            .samples(args.top_k)
            .build()
            .unwrap();

        // TODO: error handling
        let results = self
            .index
            .top_n::<Attachment>(request)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, _, attachment)| attachment)
            .collect();

        Ok(results)
    }
}
