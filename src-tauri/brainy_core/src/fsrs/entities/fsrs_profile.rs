use serde::{Deserialize, Serialize};

use crate::Guid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsrsProfile {
    id: Guid,
    // TODO: value object here? at least validate the name length
    name: String,
    request_retention: f64,
    maximum_interval: f64,
    weights: Vec<f64>,
}

impl FsrsProfile {
    pub fn new(
        id: Option<Guid>,
        name: String,
        request_retention: f64,
        maximum_interval: f64,
        weights: Vec<f64>,
    ) -> Self {
        Self {
            id: id.unwrap_or(Guid::new_v4()),
            name,
            request_retention,
            maximum_interval,
            weights,
        }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn request_retention(&self) -> f64 {
        self.request_retention
    }

    pub fn maximum_interval(&self) -> f64 {
        self.maximum_interval
    }

    pub fn weights(&self) -> &[f64] {
        &self.weights
    }
}
