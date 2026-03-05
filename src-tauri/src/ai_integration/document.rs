use std::sync::Arc;

use lancedb::arrow::arrow_schema::{DataType, Field, Fields, Schema};
use rig::Embed;
use serde::{Deserialize, Serialize};

#[derive(Embed, Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    #[embed]
    pub content: String,
}

impl Document {
    pub fn schema(dims: usize) -> Schema {
        Schema::new(Fields::from(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float64, true)),
                    dims as i32,
                ),
                false,
            ),
        ]))
    }
}
