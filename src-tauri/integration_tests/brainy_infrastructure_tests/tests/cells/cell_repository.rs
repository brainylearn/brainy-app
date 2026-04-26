use brainy_domain::{
    Guid, ROOT_FOLDER_ID,
    cells::{
        entities::{
            cell::{Cell, CellType},
            repetition::{Repetition, State},
            review::Review,
        },
        repositories::{cell_repository::CellRepository, review_repository::ReviewRepository},
        test_utils::create_cell,
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
use chrono::{DateTime, Duration, Utc};
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn CellRepository, SqliteCellRepository);
    register_scope!(injector, dyn ReviewRepository, SqliteReviewRepository);
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    injector
}

#[tokio::test]
pub async fn get_by_id_valid_input_returned_cell_correctly() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cell = create_cell(
        None,
        file.id(),
        r#"
                <cloze index="1">test<cloze>
                <cloze index="2">test<cloze>
            "#
        .to_string(),
        CellType::Cloze,
        0,
    );
    cell_repository.create(&cell).await.unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = cell_repository.get_by_id(cell.id()).await.unwrap();

    // Assert

    assert_eq!(cell.id(), actual.id());
    assert_eq!(2, actual.repetitions().len());
    assert!(
        actual
            .repetitions()
            .iter()
            .any(|r| r.additional_content().unwrap() == "1")
    );
    assert!(
        actual
            .repetitions()
            .iter()
            .any(|r| r.additional_content().unwrap() == "2")
    );
}

#[tokio::test]
pub async fn get_file_cells_ordered_by_index_valid_input_returned_files_ordered() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;

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
        create_cell(
            None,
            file.id(),
            r#"<cloze index="1"></cloze>"#.to_string(),
            CellType::Cloze,
            0,
        ),
        create_cell(None, file.id(), "".to_string(), CellType::Note, 1),
    ];

    cell_repository.create(&cells[1]).await.unwrap();
    cell_repository.create(&cells[0]).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = cell_repository
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();

    // Assert

    assert_eq!(cells[0].id(), actual[0].id());
    assert_eq!(1, actual[0].repetitions().len());
    assert_eq!(cells[1].id(), actual[1].id());
}

#[tokio::test]
pub async fn update_deleted_old_repetitions_and_added_new_ones() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let mut cell = create_cell(
        None,
        file.id(),
        r#"
                <cloze index="1">test<cloze>
                <cloze index="2">test<cloze>
            "#
        .to_string(),
        CellType::Cloze,
        0,
    );
    cell_repository.create(&cell).await.unwrap();
    scope.save_changes().await.unwrap();

    let old_repetitions = cell.repetitions().clone();
    cell.set_content(
        r#"
                <cloze index="1">test<cloze>
                <cloze index="3">test<cloze>
            "#
        .to_string(),
    );

    // Act

    cell_repository.update(&cell).await.unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual = cell_repository.get_by_id(cell.id()).await.unwrap();

    assert_eq!(2, cell.repetitions().len());
    assert!(
        actual
            .repetitions()
            .iter()
            .any(|r| r.additional_content().unwrap() == "1"
                && old_repetitions.iter().any(|r2| r2.id() == r.id()))
    );
    assert!(
        actual
            .repetitions()
            .iter()
            .any(|r| r.additional_content().unwrap() == "3")
    );

    let deleted_repetition_id = old_repetitions
        .iter()
        .find(|r| r.additional_content().unwrap() == "2")
        .unwrap()
        .id();
    assert!(
        !cell
            .repetitions()
            .iter()
            .any(|r| r.id() == deleted_repetition_id)
    );
}

#[tokio::test]
pub async fn search_cells_valid_input_searched_cells_correctly() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;

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
        create_cell(None, file.id(), "Test 1".to_string(), CellType::Note, 0),
        create_cell(None, file.id(), "Test 2".to_string(), CellType::Note, 1),
        create_cell(
            None,
            file.id(),
            "Not include".to_string(),
            CellType::Note,
            1,
        ),
    ];

    cell_repository.create(&cells[1]).await.unwrap();
    cell_repository.create(&cells[0]).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = cell_repository.search_cells("test").await.unwrap();

    // Assert

    assert_eq!(2, actual.len());
    assert!(actual.iter().any(|cell| cell.id() == cells[0].id()));
    assert!(actual.iter().any(|cell| cell.id() == cells[1].id()));
}

#[tokio::test]
pub async fn get_study_repetitions_valid_input_returned_count_correctly() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cell_id = Guid::new_v4();
    let create_repetition =
        |due: DateTime<Utc>, state: State, additional_content: Option<String>| {
            Repetition::new_unchecked(
                Guid::new_v4(),
                Utc::now(),
                Utc::now(),
                file.id(),
                cell_id,
                due,
                0.0,
                0.0,
                0,
                0,
                0,
                0,
                state,
                None,
                additional_content,
            )
        };

    let cell = Cell::new_unchecked(
        cell_id,
        Utc::now(),
        Utc::now(),
        file.id(),
        "".to_string(),
        CellType::Cloze,
        0,
        "".to_string(),
        vec![
            create_repetition(Utc::now().to_utc(), State::New, None),
            create_repetition(Utc::now().to_utc(), State::New, None),
            create_repetition(Utc::now().to_utc(), State::Learning, None),
            create_repetition(Utc::now().to_utc(), State::Relearning, None),
            create_repetition(Utc::now().to_utc(), State::Review, None),
            // Due later.
            create_repetition(
                Utc::now().to_utc() + Duration::days(1),
                State::New,
                Some("6".to_string()),
            ),
        ],
    );
    cell_repository.create(&cell).await.unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = cell_repository
        .get_study_repetitions(file.id())
        .await
        .unwrap();

    // Assert

    assert_eq!(2, actual.new);
    assert_eq!(1, actual.learning);
    assert_eq!(1, actual.relearning);
    assert_eq!(1, actual.review);
}

#[tokio::test]
pub async fn get_study_repetitions_for_all_files_valid_input_returned_count_correctly() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;

    let file1 = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    let file2 = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test2".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file1).await.unwrap();
    file_repository.create(&file2).await.unwrap();

    let create_repetition = |cell_id: Guid, file_id: Guid, due: DateTime<Utc>, state: State| {
        Repetition::new_unchecked(
            Guid::new_v4(),
            Utc::now(),
            Utc::now(),
            file_id,
            cell_id,
            due,
            0.0,
            0.0,
            0,
            0,
            0,
            0,
            state,
            None,
            None,
        )
    };

    let cell1_id = Guid::new_v4();
    let cell1 = Cell::new_unchecked(
        cell1_id,
        Utc::now(),
        Utc::now(),
        file1.id(),
        "".to_string(),
        CellType::Cloze,
        0,
        "".to_string(),
        vec![
            create_repetition(cell1_id, file1.id(), Utc::now().to_utc(), State::New),
            create_repetition(cell1_id, file1.id(), Utc::now().to_utc(), State::New),
            create_repetition(cell1_id, file1.id(), Utc::now().to_utc(), State::Learning),
        ],
    );

    let cell2_id = Guid::new_v4();
    let cell2 = Cell::new_unchecked(
        cell2_id,
        Utc::now(),
        Utc::now(),
        file2.id(),
        "".to_string(),
        CellType::Cloze,
        0,
        "".to_string(),
        vec![
            create_repetition(cell2_id, file2.id(), Utc::now().to_utc(), State::Relearning),
            create_repetition(cell2_id, file2.id(), Utc::now().to_utc(), State::Review),
            // Due later.
            create_repetition(
                cell2_id,
                file2.id(),
                Utc::now().to_utc() + Duration::days(1),
                State::New,
            ),
        ],
    );
    cell_repository.create(&cell1).await.unwrap();
    cell_repository.create(&cell2).await.unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = cell_repository
        .get_study_repetitions_for_all_files()
        .await
        .unwrap();

    // Assert

    assert_eq!(1, actual[&file1.id()].learning);
    assert_eq!(2, actual[&file1.id()].new);
    assert_eq!(0, actual[&file1.id()].relearning);

    assert_eq!(0, actual[&file2.id()].new);
    assert_eq!(1, actual[&file2.id()].relearning);
    assert_eq!(1, actual[&file2.id()].review);
}

#[tokio::test]
async fn get_home_statistics_with_reviews_returned_correct_statistics() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let file_repository = scope.resolve::<dyn FileRepository>().await;
    let cell_repository = scope.resolve::<dyn CellRepository>().await;
    let review_repository = scope.resolve::<dyn ReviewRepository>().await;

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );
    file_repository.create(&file).await.unwrap();

    let cell_id = Guid::new_v4();

    let create_repetition = |state: State| {
        Repetition::new_unchecked(
            Guid::new_v4(),
            Utc::now(),
            Utc::now(),
            file.id(),
            cell_id,
            Utc::now(),
            0.0,
            0.0,
            0,
            0,
            0,
            0,
            state,
            None,
            None,
        )
    };

    let cell = Cell::new_unchecked(
        cell_id,
        Utc::now(),
        Utc::now(),
        file.id(),
        "".to_string(),
        CellType::Cloze,
        0,
        "".to_string(),
        vec![
            create_repetition(State::New),
            create_repetition(State::New),
            create_repetition(State::Learning),
            create_repetition(State::Relearning),
            create_repetition(State::Review),
            // Due later.
            create_repetition(State::New),
        ],
    );
    cell_repository.create(&cell).await.unwrap();

    review_repository
        .create(&Review {
            date: Utc::now().to_utc(),
            study_time: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    review_repository
        .create(&Review {
            date: Utc::now().to_utc(),
            study_time: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    review_repository
        .create(&Review {
            date: Utc::now().to_utc() - Duration::days(1),
            study_time: 5,
            ..Default::default()
        })
        .await
        .unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = cell_repository.get_home_statistics().await.unwrap();

    // Assert

    assert_eq!(2, actual.number_of_reviews);
    assert_eq!(20, actual.total_time);
    assert_eq!(2, actual.review_counts[&Utc::now().date_naive()]);
    assert_eq!(
        1,
        actual.review_counts[&(Utc::now().to_utc() - Duration::days(1)).date_naive()]
    );
    assert_eq!(6, actual.due_counts[&Utc::now().date_naive()]);
}
