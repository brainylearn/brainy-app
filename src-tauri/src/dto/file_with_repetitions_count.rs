use std::collections::{HashMap, VecDeque};

use brainy_core::{
    cells::models::file_repetitions_count::FileRepetitionCounts, file_system::{
        entities::{file::File, folder::Folder},
        value_objects::path::Path,
    }, Guid
};
use serde::Serialize;

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
        folders: &[Folder],
        files: &[File],
        mut study_repetitions: HashMap<Guid, FileRepetitionCounts>,
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
                .map(|value| (value.id(), Path::from(value.name()))),
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
                    .map(|value| (value.id(), top.1.navigate(value.name()))),
            );
        }

        let folder_names = result
            .iter()
            .map(|value| (value.id, value.path.clone()))
            .collect::<HashMap<Guid, Path>>();

        for file in files {
            let mut repetition_count = study_repetitions.remove(&file.id());
            if repetition_count.is_none() {
                repetition_count = Some(FileRepetitionCounts::default());
            }

            result.push(FileWithRepetitionsCount {
                id: file.id(),
                path: match file.parent_id() {
                    None => Path::from(file.name()),
                    Some(parent_id) => folder_names.get(&parent_id).unwrap().navigate(file.name()),
                },
                is_folder: false,
                repetition_counts: repetition_count,
            });
        }

        result
    }
}

#[cfg(test)]
pub mod tests {
    use brainy_core::ROOT_FOLDER_ID;

    use super::*;

    #[test]
    pub fn parse_file_system_valid_input_parsed_correctly() {
        // Arrange

        let parent_folder_id = Guid::new_v4();
        let folders: Vec<Folder> = vec![
            Folder::new_unchecked(Some(ROOT_FOLDER_ID), None, "root".try_into().unwrap()),
            Folder::new_unchecked(
                Some(parent_folder_id),
                Some(ROOT_FOLDER_ID),
                "parent folder".try_into().unwrap(),
            ),
        ];

        let files: Vec<File> = vec![File::new_unchecked(
            None,
            Some(parent_folder_id),
            "file".try_into().unwrap(),
        )];

        let mut study_repetitions = HashMap::new();
        study_repetitions.insert(
            files[0].id(),
            FileRepetitionCounts {
                new: 4,
                ..Default::default()
            },
        );

        // Act

        let actual =
            FileWithRepetitionsCount::parse_file_system(&folders, &files, study_repetitions);

        // Assert

        assert!(actual.iter().any(|f| f.path.to_string() == "/root"));
        assert!(
            actual
                .iter()
                .any(|f| f.path.to_string() == "/root/parent folder")
        );
        assert!(
            actual
                .iter()
                .any(|f| f.path.to_string() == "/root/parent folder/file"
                    && f.repetition_counts.as_ref().unwrap().new == 4)
        );
    }
}
