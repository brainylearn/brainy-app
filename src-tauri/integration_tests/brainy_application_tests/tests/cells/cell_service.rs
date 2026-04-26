use brainy_application::cells::cell_service::CellService;
use brainy_domain::{
    Guid, ROOT_FOLDER_ID,
    cells::{
        entities::{
            cell::{Cell, CellType},
            review::Rating,
        },
        repositories::{cell_repository::CellRepository, review_repository::ReviewRepository},
        value_objects::repetition_update::RepetitionUpdate,
    },
    file_system::{
        entities::file::File, repositories::file_repository::FileRepository,
        value_objects::fsrs_profile_choice::FsrsProfileChoice,
    },
};
use brainy_infrastructure::{
    cells::{
        sqlite_cell_repository::SqliteCellRepository,
        sqlite_review_repository::SqliteReviewRepository,
    },
    common::unit_of_work::UnitOfWorkExt,
    file_system::sqlite_file_repository::SqliteFileRepository,
};
use brainy_test_utils::create_test_injector;
use chrono::Utc;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn CellRepository, SqliteCellRepository);
    register_scope!(injector, dyn ReviewRepository, SqliteReviewRepository);
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    register_scope!(injector, CellService);
    injector
}

#[tokio::test]
pub async fn create_cell_moved_all_cells_down_and_created_cell() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let service = scope.resolve::<CellService>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cells = [
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
    ];

    cell_repository.create(&cells[0]).await.unwrap();
    cell_repository.create(&cells[1]).await.unwrap();
    cell_repository.create(&cells[2]).await.unwrap();
    cell_repository.create(&cells[3]).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .create_cell(file.id(), "".to_string(), CellType::Cloze, 2)
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual_cells = cell_repository
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();
    assert_eq!(actual_cells[0].id(), cells[0].id());
    assert_eq!(actual_cells[1].id(), cells[1].id());
    assert_eq!(actual_cells[2].id(), actual);
    assert_eq!(actual_cells[3].id(), cells[2].id());
    assert_eq!(actual_cells[4].id(), cells[3].id());
}

#[tokio::test]
pub async fn delete_by_id_moved_all_cells_up_and_deleted_cell() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let service = scope.resolve::<CellService>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cells = [
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
    ];

    cell_repository.create(&cells[0]).await.unwrap();
    cell_repository.create(&cells[1]).await.unwrap();
    cell_repository.create(&cells[2]).await.unwrap();
    cell_repository.create(&cells[3]).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    service.delete_by_id(cells[1].id()).await.unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual_cells = cell_repository
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();

    assert_eq!(actual_cells[0].id(), cells[0].id());
    assert_eq!(actual_cells[0].index(), 0);

    assert_eq!(actual_cells[1].id(), cells[2].id());
    assert_eq!(actual_cells[1].index(), 1);

    assert_eq!(actual_cells[2].id(), cells[3].id());
    assert_eq!(actual_cells[2].index(), 2);
}

#[tokio::test]
pub async fn move_cell_forward_moved_cell_correctly() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let service = scope.resolve::<CellService>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cells = [
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 4),
    ];

    cell_repository.create(&cells[0]).await.unwrap();
    cell_repository.create(&cells[1]).await.unwrap();
    cell_repository.create(&cells[2]).await.unwrap();
    cell_repository.create(&cells[3]).await.unwrap();
    cell_repository.create(&cells[4]).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    service.move_cell(cells[1].id(), 3).await.unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual_cells = cell_repository
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();

    assert_eq!(actual_cells[0].id(), cells[0].id());
    assert_eq!(actual_cells[0].index(), 0);

    assert_eq!(actual_cells[1].id(), cells[2].id());
    assert_eq!(actual_cells[1].index(), 1);

    assert_eq!(actual_cells[2].id(), cells[3].id());
    assert_eq!(actual_cells[2].index(), 2);

    assert_eq!(actual_cells[3].id(), cells[1].id());
    assert_eq!(actual_cells[3].index(), 3);

    assert_eq!(actual_cells[4].id(), cells[4].id());
    assert_eq!(actual_cells[4].index(), 4);
}

#[tokio::test]
pub async fn move_cell_backward_moved_cell_correctly() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let service = scope.resolve::<CellService>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cells = [
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 4),
    ];

    cell_repository.create(&cells[0]).await.unwrap();
    cell_repository.create(&cells[1]).await.unwrap();
    cell_repository.create(&cells[2]).await.unwrap();
    cell_repository.create(&cells[3]).await.unwrap();
    cell_repository.create(&cells[4]).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    service.move_cell(cells[3].id(), 1).await.unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual_cells = cell_repository
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();

    assert_eq!(actual_cells[0].id(), cells[0].id());
    assert_eq!(actual_cells[0].index(), 0);

    assert_eq!(actual_cells[1].id(), cells[3].id());
    assert_eq!(actual_cells[1].index(), 1);

    assert_eq!(actual_cells[2].id(), cells[1].id());
    assert_eq!(actual_cells[2].index(), 2);

    assert_eq!(actual_cells[3].id(), cells[2].id());
    assert_eq!(actual_cells[3].index(), 3);

    assert_eq!(actual_cells[4].id(), cells[4].id());
    assert_eq!(actual_cells[4].index(), 4);
}

#[tokio::test]
pub async fn register_review_updated_repetition_and_created_review() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let service = scope.resolve::<CellService>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let content = r#"
            <cloze index="1">Test</cloze>
        "#
    .to_string();
    let cell = Cell::new(None, file.id(), content, CellType::Cloze, 0);

    cell_repository.create(&cell).await.unwrap();
    scope.save_changes().await.unwrap();

    let repetition_update = RepetitionUpdate {
        id: cell.repetitions()[0].id,
        cell_id: cell.id(),
        file_id: cell.file_id(),
        stability: 5.4f64,
        ..Default::default()
    };

    // Act

    service
        .register_review(repetition_update.clone(), Rating::Hard, 10)
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual = cell_repository.get_by_id(cell.id()).await.unwrap();

    assert_eq!(
        actual.repetitions()[0].stability,
        repetition_update.stability
    );

    let home_statistics = cell_repository.get_home_statistics().await.unwrap();
    assert_eq!(1, home_statistics.number_of_reviews);
}

#[tokio::test]
pub async fn enforce_cell_invariants_on_cell_two_cells_with_same_index_updated_index() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let service = scope.resolve::<CellService>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cells = [
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
        Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
    ];

    cell_repository.create(&cells[0]).await.unwrap();
    cell_repository.create(&cells[1]).await.unwrap();
    scope.save_changes().await.unwrap();

    // Act

    service
        .enforce_cell_invariants_on_cell(cells[0].id())
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual_cells = cell_repository
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();

    assert_eq!(actual_cells[0].id(), cells[0].id());
    assert_eq!(actual_cells[0].index(), 0);

    assert_eq!(actual_cells[1].id(), cells[1].id());
    assert_eq!(actual_cells[1].index(), 1);
}
