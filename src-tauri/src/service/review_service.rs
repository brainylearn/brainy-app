// TODO: delete

// #[cfg(test)]
// mod tests {
//     use chrono::Duration;
//
//     use crate::{
//         entity::repetition::State,
//         service::tests::{create_file_cell, get_db, insert_repetitions},
//     };
//
//     use super::*;
//
//     #[tokio::test]
//     async fn get_home_statistics_no_reviews_returned_zero() {
//         // Arrange
//
//         let db_conn = get_db().await;
//
//         // Act
//
//         let actual = get_home_statistics(&db_conn).await.unwrap();
//
//         // Assert
//
//         assert_eq!(0, actual.number_of_reviews);
//         assert_eq!(0, actual.total_time);
//     }
//
//     #[tokio::test]
//     async fn get_home_statistics_with_reviews_returned_correct_statistics() {
//         // Arrange
//
//         let db_conn = get_db().await;
//         let (file_id, cell_id) = create_file_cell(&db_conn, "file 1").await;
//         insert_repetitions(
//             &db_conn,
//             vec![repetition::ActiveModel {
//                 file_id: Set(file_id),
//                 cell_id: Set(cell_id),
//                 ..Default::default()
//             }],
//         )
//         .await
//         .unwrap();
//         let repetition_id = repetition::Entity::find()
//             .one(&db_conn)
//             .await
//             .unwrap()
//             .unwrap()
//             .id;
//         let repetition = repetition::Model {
//             id: repetition_id,
//             file_id,
//             cell_id,
//             due: Utc::now().to_utc(),
//             last_review: Utc::now().to_utc(),
//             ..Default::default()
//         };
//         register_review(&db_conn, repetition.clone(), Rating::Good, 15)
//             .await
//             .unwrap();
//         register_review(&db_conn, repetition.clone(), Rating::Again, 10)
//             .await
//             .unwrap();
//         let review = review::ActiveModel {
//             cell_id: Set(Some(cell_id)),
//             study_time: Set(12),
//             date: Set(Utc::now().to_utc() - Duration::days(1)),
//             rating: Set(Rating::Again),
//             ..Default::default()
//         };
//         review.insert(&db_conn).await.unwrap();
//
//         // Act
//
//         let actual = get_home_statistics(&db_conn).await.unwrap();
//
//         // Assert
//
//         assert_eq!(2, actual.number_of_reviews);
//         assert_eq!(25, actual.total_time);
//     }
//
//     #[tokio::test]
//     async fn register_review_valid_input_registered_review_and_update_repetition() {
//         // Arrange
//
//         let db_conn = get_db().await;
//         let (file_id, cell_id) = create_file_cell(&db_conn, "file 1").await;
//         insert_repetitions(
//             &db_conn,
//             vec![repetition::ActiveModel {
//                 file_id: Set(file_id),
//                 cell_id: Set(cell_id),
//                 ..Default::default()
//             }],
//         )
//         .await
//         .unwrap();
//         let repetition_id = repetition::Entity::find()
//             .one(&db_conn)
//             .await
//             .unwrap()
//             .unwrap()
//             .id;
//         let date = Utc::now().to_utc();
//         let repetition = repetition::Model {
//             id: repetition_id,
//             file_id,
//             cell_id,
//             due: date,
//             reps: 1,
//             stability: 2.1f32,
//             difficulty: 4.2f32,
//             elapsed_days: 5,
//             scheduled_days: 6,
//             lapses: 7,
//             state: State::New,
//             last_review: date,
//             additional_content: Some("".into()),
//         };
//
//         // Act
//
//         register_review(&db_conn, repetition.clone(), Rating::Again, 10)
//             .await
//             .unwrap();
//
//         // Assert
//
//         let actual_repetition = repetition::Entity::find()
//             .one(&db_conn)
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(actual_repetition.id, repetition.id);
//         assert_eq!(actual_repetition.file_id, repetition.file_id);
//         assert_eq!(actual_repetition.cell_id, repetition.cell_id);
//         assert_eq!(actual_repetition.due, repetition.due);
//         assert_eq!(actual_repetition.reps, repetition.reps);
//         assert_eq!(actual_repetition.stability, repetition.stability);
//         assert_eq!(actual_repetition.difficulty, repetition.difficulty);
//         assert_eq!(actual_repetition.elapsed_days, repetition.elapsed_days);
//         assert_eq!(actual_repetition.scheduled_days, repetition.scheduled_days);
//         assert_eq!(actual_repetition.lapses, repetition.lapses);
//         assert_eq!(actual_repetition.state, repetition.state);
//         assert_eq!(actual_repetition.last_review, repetition.last_review);
//
//         let actual_review = review::Entity::find().one(&db_conn).await.unwrap().unwrap();
//         assert_eq!(actual_review.cell_id, Some(repetition.cell_id));
//         assert!((actual_review.date - Utc::now().to_utc()).num_minutes() < 1);
//         assert_eq!(actual_review.rating, Rating::Again);
//         assert_eq!(actual_review.study_time, 10);
//     }
// }
