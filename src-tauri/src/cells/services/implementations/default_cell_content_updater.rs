use std::sync::Arc;

use async_trait::async_trait;
use injector_derive::ScopeInjectable;
use scraper::{Html, Selector};

use crate::{
    Guid,
    cells::{
        entities::cell::{Cell, CellType},
        repositories::cell_repository::CellRepository,
        services::cell_content_updater::{CellContentUpdater, CellContentUpdaterError},
        value_objects::incremental_reading::IncrementalReading,
    },
    extracts::{entities::extract::Extract, repositories::extract_repository::ExtractRepository},
};

#[derive(ScopeInjectable)]
pub struct DefaultCellContentUpdater {
    cell_repository: Arc<dyn CellRepository>,
    extract_repository: Arc<dyn ExtractRepository>,
}

#[async_trait]
impl CellContentUpdater for DefaultCellContentUpdater {
    async fn update_cell_content(
        &self,
        cell_id: Guid,
        content: String,
    ) -> Result<(), CellContentUpdaterError> {
        let mut cell = self.cell_repository.get_by_id(cell_id).await?;
        cell.set_content(content);
        self.cell_repository.update(&cell).await?;

        if cell.cell_type() == &CellType::IncrementalReading {
            self.sync_extracts(&cell).await?;
        }

        Ok(())
    }
}

impl DefaultCellContentUpdater {
    async fn sync_extracts(&self, cell: &Cell) -> Result<(), CellContentUpdaterError> {
        let ir: IncrementalReading =
            serde_json::from_str(cell.content()).expect("Cannot parse incremental reading JSON!");

        let found: Vec<(String, String)> = {
            let html_content = ir.content.unwrap_or_default();
            let selector = Selector::parse("highlight").expect("Invalid selector");
            let document = Html::parse_fragment(&html_content);
            document
                .select(&selector)
                .filter_map(|el| {
                    let id = el.attr("highlight-id")?.to_string();
                    let inner_html = el.inner_html();
                    Some((id, inner_html))
                })
                .collect()
        };

        log::info!("Found {} extracts.", found.len());

        let existing = self.extract_repository.get_by_cell_id(cell.id()).await?;

        for (highlight_id, inner_html) in &found {
            let highlight_guid: Guid = highlight_id.parse().unwrap_or_else(|_| Guid::new_v4());

            match existing
                .iter()
                .find(|e| e.id().to_string() == *highlight_id)
            {
                None => {
                    let extract = Extract::new(highlight_guid, cell.id(), inner_html.clone());
                    self.extract_repository.create(&extract).await?;
                }
                Some(existing_extract) if existing_extract.inner_html() != inner_html => {
                    self.extract_repository
                        .update_inner_html(highlight_guid, inner_html.clone())
                        .await?;
                }
                _ => {}
            }
        }

        for existing_extract in &existing {
            let still_present = found
                .iter()
                .any(|(id, _)| *id == existing_extract.id().to_string());
            if !still_present {
                self.extract_repository
                    .delete_by_id(existing_extract.id())
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use injector::{injector::Injector, register_scope};

    use crate::{
        ROOT_FOLDER_ID,
        cells::{
            entities::cell::{Cell, CellType},
            repositories::cell_repository::CellRepository,
            services::cell_content_updater::CellContentUpdater,
            value_objects::incremental_reading::{
                IncrementalReading, IncrementalReadingPriority, IncrementalReadingSource,
            },
        },
        extracts::repositories::extract_repository::ExtractRepository,
        file_system::{
            entities::file::File, repositories::file_repository::FileRepository,
            value_objects::fsrs_profile_choice::FsrsProfileChoice,
        },
        infrastructure::{
            extensions::unit_of_work::UnitOfWorkExt,
            repositories::sqlite::{
                sqlite_cell_repository::SqliteCellRepository,
                sqlite_extract_repository::SqliteExtractRepository,
                sqlite_file_repository::SqliteFileRepository,
            },
        },
        test_utils::create_test_injector,
    };

    use super::*;

    async fn initialize_test_injector() -> Injector {
        let mut injector = create_test_injector().await;
        register_scope!(injector, dyn CellRepository, SqliteCellRepository);
        register_scope!(injector, dyn FileRepository, SqliteFileRepository);
        register_scope!(injector, dyn ExtractRepository, SqliteExtractRepository);
        register_scope!(injector, DefaultCellContentUpdater);
        injector
    }

    fn ir_content(html: &str) -> String {
        serde_json::to_string(&IncrementalReading {
            content: Some(html.to_string()),
            title: Some("Test".to_string()),
            source: IncrementalReadingSource {
                source_type: "url".to_string(),
                url: "https://example.com".to_string(),
            },
            priority: IncrementalReadingPriority::Normal,
        })
        .unwrap()
    }

    async fn create_test_file(file_repository: &Arc<dyn FileRepository>) -> File {
        let file = File::new_unchecked(
            Guid::new_v4(),
            chrono::Utc::now(),
            chrono::Utc::now(),
            Some(ROOT_FOLDER_ID),
            "test".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        );
        file_repository.create(&file).await.unwrap();
        file
    }

    #[tokio::test]
    pub async fn update_cell_content_creates_extract_for_new_highlight() {
        // Arrange

        let injector = initialize_test_injector().await;
        let scope = injector.start_scope();
        let file_repository = scope.resolve::<dyn FileRepository>().await;
        let cell_repository = scope.resolve::<dyn CellRepository>().await;
        let extract_repository = scope.resolve::<dyn ExtractRepository>().await;
        let service = scope.resolve::<DefaultCellContentUpdater>().await;

        let file = create_test_file(&file_repository).await;
        let cell = Cell::new(
            None,
            file.id(),
            ir_content(""),
            CellType::IncrementalReading,
            0,
        );
        cell_repository.create(&cell).await.unwrap();
        scope.save_changes().await.unwrap();

        let highlight_id = "550e8400-e29b-41d4-a716-446655440000";
        let content = ir_content(&format!(
            r#"<highlight highlight-id="{highlight_id}">some text</highlight>"#
        ));

        // Act

        service
            .update_cell_content(cell.id(), content)
            .await
            .unwrap();
        scope.save_changes().await.unwrap();

        // Assert

        let extracts = extract_repository.get_by_cell_id(cell.id()).await.unwrap();
        assert_eq!(1, extracts.len());
        assert_eq!(highlight_id, extracts[0].id().to_string());
        assert_eq!("some text", extracts[0].inner_html());
    }

    #[tokio::test]
    pub async fn update_cell_content_deletes_extract_when_highlight_removed() {
        // Arrange

        let injector = initialize_test_injector().await;
        let scope = injector.start_scope();
        let file_repository = scope.resolve::<dyn FileRepository>().await;
        let cell_repository = scope.resolve::<dyn CellRepository>().await;
        let extract_repository = scope.resolve::<dyn ExtractRepository>().await;
        let service = scope.resolve::<DefaultCellContentUpdater>().await;

        let file = create_test_file(&file_repository).await;
        let highlight_id = "550e8400-e29b-41d4-a716-446655440000";
        let content_with = ir_content(&format!(
            r#"<highlight highlight-id="{highlight_id}">some text</highlight>"#
        ));
        let cell = Cell::new(
            None,
            file.id(),
            ir_content(""),
            CellType::IncrementalReading,
            0,
        );
        cell_repository.create(&cell).await.unwrap();
        scope.save_changes().await.unwrap();

        service
            .update_cell_content(cell.id(), content_with)
            .await
            .unwrap();
        scope.save_changes().await.unwrap();

        // Act

        service
            .update_cell_content(cell.id(), ir_content("no highlights here"))
            .await
            .unwrap();
        scope.save_changes().await.unwrap();

        // Assert

        let extracts = extract_repository.get_by_cell_id(cell.id()).await.unwrap();
        assert_eq!(0, extracts.len());
    }

    #[tokio::test]
    pub async fn update_cell_content_updates_inner_html_when_highlight_text_changes() {
        // Arrange

        let injector = initialize_test_injector().await;
        let scope = injector.start_scope();
        let file_repository = scope.resolve::<dyn FileRepository>().await;
        let cell_repository = scope.resolve::<dyn CellRepository>().await;
        let extract_repository = scope.resolve::<dyn ExtractRepository>().await;
        let service = scope.resolve::<DefaultCellContentUpdater>().await;

        let file = create_test_file(&file_repository).await;
        let highlight_id = "550e8400-e29b-41d4-a716-446655440000";
        let cell = Cell::new(
            None,
            file.id(),
            ir_content(""),
            CellType::IncrementalReading,
            0,
        );
        cell_repository.create(&cell).await.unwrap();
        scope.save_changes().await.unwrap();

        service
            .update_cell_content(
                cell.id(),
                ir_content(&format!(
                    r#"<highlight highlight-id="{highlight_id}">old text</highlight>"#
                )),
            )
            .await
            .unwrap();
        scope.save_changes().await.unwrap();

        // Act

        service
            .update_cell_content(
                cell.id(),
                ir_content(&format!(
                    r#"<highlight highlight-id="{highlight_id}">new text</highlight>"#
                )),
            )
            .await
            .unwrap();
        scope.save_changes().await.unwrap();

        // Assert

        let extracts = extract_repository.get_by_cell_id(cell.id()).await.unwrap();
        assert_eq!(1, extracts.len());
        assert_eq!("new text", extracts[0].inner_html());
    }

    #[tokio::test]
    pub async fn update_cell_content_does_not_create_extracts_for_note_cell() {
        // Arrange

        let injector = initialize_test_injector().await;
        let scope = injector.start_scope();
        let file_repository = scope.resolve::<dyn FileRepository>().await;
        let cell_repository = scope.resolve::<dyn CellRepository>().await;
        let extract_repository = scope.resolve::<dyn ExtractRepository>().await;
        let service = scope.resolve::<DefaultCellContentUpdater>().await;

        let file = create_test_file(&file_repository).await;
        let highlight_id = "550e8400-e29b-41d4-a716-446655440000";
        let cell = Cell::new(None, file.id(), "".to_string(), CellType::Note, 0);
        cell_repository.create(&cell).await.unwrap();
        scope.save_changes().await.unwrap();

        // Act

        service
            .update_cell_content(
                cell.id(),
                format!(r#"<highlight highlight-id="{highlight_id}">some text</highlight>"#),
            )
            .await
            .unwrap();
        scope.save_changes().await.unwrap();

        // Assert

        let extracts = extract_repository.get_by_cell_id(cell.id()).await.unwrap();
        assert_eq!(0, extracts.len());
    }
}
