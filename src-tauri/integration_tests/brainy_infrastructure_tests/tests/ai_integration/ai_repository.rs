use brainy_domain::ai_integration::{
    entities::{
        chat::Chat,
        message::{Message, MessageContent},
    },
    repositories::ai_repository::AiRepository,
};
use brainy_infrastructure::{
    ai_integration::sqlite_ai_repository::SqliteAiRepository, common::unit_of_work::UnitOfWorkExt,
};
use brainy_test_utils::create_test_injector;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, SqliteAiRepository);
    injector
}

#[tokio::test]
pub async fn get_all_chats_sorted_by_date_desc_multiple_chats_returned_all() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<SqliteAiRepository>().await;

    let chat1 = Chat::new(None, "First".to_string());
    repository.upsert_chat(&chat1).await.unwrap();
    let chat2 = Chat::new(None, "Second".to_string());
    repository.upsert_chat(&chat2).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = repository
        .get_all_chats_sorted_by_date_desc()
        .await
        .unwrap();

    // Assert

    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0].title(), "First");
    assert_eq!(actual[1].title(), "Second");
}

#[tokio::test]
pub async fn get_chat_messages_ordered_multiple_messages_returned_all() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<SqliteAiRepository>().await;

    let chat = Chat::new(None, "Chat".to_string());
    repository.upsert_chat(&chat).await.unwrap();

    repository
        .upsert_message(&Message::new(
            None,
            chat.id(),
            MessageContent::Human("Human".to_string()),
        ))
        .await
        .unwrap();
    repository
        .upsert_message(&Message::new(
            None,
            chat.id(),
            MessageContent::Assistant("Assistant".to_string()),
        ))
        .await
        .unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = repository
        .get_chat_messages_ordered(chat.id())
        .await
        .unwrap();

    // Assert

    assert_eq!(actual.len(), 2);
    assert_eq!(
        *actual[0].content(),
        MessageContent::Human("Human".to_string())
    );
    assert_eq!(
        *actual[1].content(),
        MessageContent::Assistant("Assistant".to_string())
    );
}

#[tokio::test]
pub async fn delete_chat_valid_input_deleted_chat() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<SqliteAiRepository>().await;

    let chat1 = Chat::new(None, "First".to_string());
    repository.upsert_chat(&chat1).await.unwrap();
    let chat2 = Chat::new(None, "Second".to_string());
    repository.upsert_chat(&chat2).await.unwrap();

    scope.save_changes().await.unwrap();

    // Act

    repository.delete_chat(chat1.id()).await.unwrap();

    // Assert

    let actual = repository
        .get_all_chats_sorted_by_date_desc()
        .await
        .unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].title(), "Second");
}
