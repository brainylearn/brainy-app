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
pub struct SearchDocumentsArgs {
    #[schemars(
        description = "The search query or question to find relevant information in the uploaded files."
    )]
    pub query: String,
    #[schemars(description = "The maximum number of top matching results to return.")]
    pub top_k: u64,
}

#[derive(Error, Debug)]
pub enum SearchDocumentsError {}

pub struct SearchDocuments {
    pub index: Arc<LanceDbVectorIndex<MultiEmbeddingModel>>,
}

impl SearchDocuments {
    pub fn new(index: Arc<LanceDbVectorIndex<MultiEmbeddingModel>>) -> Self {
        Self { index }
    }
}

impl Tool for SearchDocuments {
    const NAME: &'static str = "search_documents";

    type Error = SearchDocumentsError;
    type Args = SearchDocumentsArgs;
    type Output = Vec<Attachment>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let parameters = serde_json::to_value(schema_for!(SearchDocumentsArgs)).unwrap();

        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Performs semantic search over the text content of \
                all files uploaded by the user. It returns relevant \
                snippets (chunks) that match the query"
                .to_string(),
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        log::info!("{} called with arguments {:?}", Self::NAME, args);

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
