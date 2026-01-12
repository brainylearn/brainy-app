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
        id: Guid,
        name: String,
        request_retention: f64,
        maximum_interval: f64,
        weights: Vec<f64>,
    ) -> Self {
        Self {
            id,
            name,
            request_retention,
            maximum_interval,
            weights,
        }
    }
}
