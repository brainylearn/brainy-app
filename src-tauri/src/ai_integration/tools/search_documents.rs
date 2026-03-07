use std::sync::Arc;

use rig::{
    completion::ToolDefinition,
    tool::Tool,
    vector_store::{
        VectorSearchRequest, VectorStoreError, VectorStoreIndex, request::SearchFilter,
    },
};
use rig_sqlite::{SqliteSearchFilter, SqliteVectorIndex};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Guid,
    ai_integration::{
        clients::multi_client::multi_embedding_model::MultiEmbeddingModel,
        document::{CHAT_ID_COLUMN_NAME, Document},
    },
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
pub enum SearchDocumentsError {
    #[error("Error fetching documents from vector store")]
    FetchingError(#[from] VectorStoreError),
    #[error("Error building search query")]
    SearchQueryError(VectorStoreError),
}

pub struct SearchDocuments {
    chat_id: Guid,
    index: Arc<SqliteVectorIndex<MultiEmbeddingModel, Document>>,
}

impl SearchDocuments {
    pub fn new(
        index: Arc<SqliteVectorIndex<MultiEmbeddingModel, Document>>,
        chat_id: Guid,
    ) -> Self {
        Self { index, chat_id }
    }
}

impl Tool for SearchDocuments {
    const NAME: &'static str = "search_documents";

    type Error = SearchDocumentsError;
    type Args = SearchDocumentsArgs;
    type Output = Vec<Document>;

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

    // TODO: unit test
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let filter = SqliteSearchFilter::eq(
            CHAT_ID_COLUMN_NAME,
            serde_json::to_value(self.chat_id.to_string()).unwrap(),
        );

        let req = match VectorSearchRequest::builder()
            .samples(args.top_k)
            .query(args.query)
            .filter(filter)
            .build()
        {
            Ok(req) => req,
            Err(err) => return Err(SearchDocumentsError::SearchQueryError(err)),
        };

        let results = self
            .index
            .top_n::<Document>(req)
            .await?
            .into_iter()
            .map(|(_, _, document)| document)
            .collect::<Vec<_>>();

        Ok(results)
    }
}
