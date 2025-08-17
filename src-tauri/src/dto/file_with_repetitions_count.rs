use std::collections::{HashMap, VecDeque};

use brainy_core::{
    Guid,
    file_system::{
        entities::{file::File, folder::Folder},
        value_objects::path::Path,
    },
};
use serde::Serialize;

use crate::value_objects::file_repetitions_count::FileRepetitionCounts;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWithRepetitionsCount {
    pub id: Guid,
    pub path: Path,
    pub is_folder: bool,
    pub repetition_counts: Option<FileRepetitionCounts>,
}

impl FileWithRepetitionsCount {
    pub fn parse_file_system(
        folders: Vec<Folder>,
        files: Vec<File>,
    ) -> Vec<FileWithRepetitionsCount> {
        let mut result = Vec::new();

        let mut map = HashMap::new();
        for folder in folders.iter() {
            map.entry(folder.parent_id())
                .or_insert(Vec::new())
                .push(folder);
        }

        let mut queue = VecDeque::new();
        queue.extend(
            map.get(&None)
                .unwrap_or(&Vec::new())
                .iter()
                .map(|value| (value.id(), Path::new(&value.name().to_string()))),
        );

        while !queue.is_empty() {
            let top = queue.pop_front().unwrap();

            result.push(FileWithRepetitionsCount {
                id: top.0,
                path: top.1.clone(),
                is_folder: true,
                repetition_counts: None,
            });

            queue.extend(
                map.get(&Some(top.0))
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|value| (value.id(), top.1.navigate(&value.name().to_string()))),
            );
        }

        let folder_names = result
            .iter()
            .map(|value| (value.id, value.path.clone()))
            .collect::<HashMap<Guid, Path>>();

        for file in files {
            result.push(FileWithRepetitionsCount {
                id: file.id(),
                path: match file.parent_id() {
                    None => Path::new(&file.name().to_string()),
                    Some(parent_id) => folder_names
                        .get(&parent_id)
                        .unwrap()
                        .navigate(&file.name().to_string()),
                },
                is_folder: false,
                repetition_counts: Some(FileRepetitionCounts {
                    ..Default::default()
                }),
            });
        }

        result
    }
}
