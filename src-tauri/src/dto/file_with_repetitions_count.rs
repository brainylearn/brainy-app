use serde::{Deserialize, Serialize};

use crate::{domain::entities::folder::Folder, value_objects::file_repetitions_count::FileRepetitionCounts};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWithRepetitionsCount {
    pub id: i32,
    pub path: String,
    pub is_folder: bool,
    pub repetition_counts: Option<FileRepetitionCounts>,
}

impl FileWithRepetitionsCount {
    pub fn new(
        id: i32,
        path: String,
        is_folder: bool,
        repetition_counts: Option<FileRepetitionCounts>,
    ) -> Self {
        Self {
            id,
            path,
            is_folder,
            repetition_counts,
        }
    }
}

impl From<&Folder> for FileWithRepetitionsCount {
    fn from(value: &Folder) -> Self {
        // TODO: fix id
        FileWithRepetitionsCount::new(21, value.path().val(), true, None)
    }
}
