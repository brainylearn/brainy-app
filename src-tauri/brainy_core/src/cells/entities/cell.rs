use std::fmt::Display;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    Guid,
    cells::value_objects::{flash_card::FlashCard, true_false::TrueFalse},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    FlashCard,
    Note,
    Cloze,
    TrueFalse,
}

impl Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).expect("Cannot serialize CellType")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    id: Guid,
    file_id: Guid,
    content: String,
    cell_type: CellType,
    searchable_content: String,
    index: u32,
}

impl Cell {
    // TODO: repetitions, maybe an aggregate
    pub(in crate::cells) fn new(
        id: Option<Guid>,
        file_id: Guid,
        content: String,
        cell_type: CellType,
        index: u32,
    ) -> Self {
        let mut output = Self {
            id: id.unwrap_or(Guid::new_v4()),
            file_id,
            content,
            cell_type,
            index,
            searchable_content: "".to_string(),
        };

        output.update_searcahble_content();
        output
    }

    /// Used for unit testing, or repositories when reconsturcting a cell.
    pub(in crate::cells) fn new_unchecked(
        id: Option<Guid>,
        file_id: Guid,
        content: String,
        cell_type: CellType,
        index: u32,
        searchable_content: String,
    ) -> Self {
        Self {
            id: id.unwrap_or(Guid::new_v4()),
            file_id,
            content,
            cell_type,
            index,
            searchable_content,
        }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn file_id(&self) -> Guid {
        self.file_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cell_type(&self) -> &CellType {
        &self.cell_type
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub(in crate::cells) fn searchable_content(&self) -> &str {
        &self.searchable_content
    }

    pub(in crate::cells) fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    // TODO: repetitions, and unit test
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.update_searcahble_content();
    }

    fn update_searcahble_content(&mut self) {
        let remove_html_regex = Regex::new("<[^>]*>").expect("Invalid regex");

        let searchable_content = match self.cell_type {
            CellType::Cloze => remove_html_regex.replace_all(&self.content, "").to_string(),
            CellType::Note => remove_html_regex.replace_all(&self.content, "").to_string(),
            CellType::FlashCard => {
                let flash_card: FlashCard =
                    serde_json::from_str(&self.content).expect("Cannot parse flash card JSON!");
                remove_html_regex
                    .replace_all(
                        &format!("{} {}", flash_card.question, flash_card.answer),
                        "",
                    )
                    .to_string()
            }
            CellType::TrueFalse => {
                let true_false: TrueFalse =
                    serde_json::from_str(&self.content).expect("Cannot parse true false JSON!");
                remove_html_regex
                    .replace_all(&true_false.question, "")
                    .to_string()
            }
        };

        self.searchable_content = searchable_content.to_lowercase().to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn new_flash_card_updated_search_content_correctly() {
        // Arrange

        let content = serde_json::to_string(&FlashCard {
            question: "question".into(),
            answer: "<bold>Answer</bold>".into(),
        })
        .unwrap();

        // Act

        let actual = Cell::new(None, Guid::new_v4(), content, CellType::FlashCard, 0);

        // Assert

        assert_eq!(actual.searchable_content(), "question answer".to_string());
    }

    #[test]
    pub fn set_content_on_true_false_updated_search_content_correctly() {
        // Arrange

        let content = serde_json::to_string(&TrueFalse {
            question: "<bold>Question</bold>".into(),
            is_true: true,
        })
        .unwrap();

        // Act

        let actual = Cell::new(None, Guid::new_v4(), content, CellType::TrueFalse, 0);

        // Assert

        assert_eq!(actual.searchable_content(), "question".to_string());
    }

    #[test]
    pub fn set_content_on_note_updated_search_content_correctly() {
        // Act

        let actual = Cell::new(
            None,
            Guid::new_v4(),
            "<bold>Note</bold>".to_string(),
            CellType::Note,
            0,
        );

        // Assert

        assert_eq!(actual.searchable_content(), "note".to_string());
    }
}
