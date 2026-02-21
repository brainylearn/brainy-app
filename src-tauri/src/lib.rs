mod ai_integration;
mod backend;
mod backup;
mod cells;
mod common;
mod file_system;
mod fsrs;
mod local_configurations;
mod settings;
mod sync;
#[cfg(test)]
mod test_utils;

use std::sync::Arc;

use crate::{
    ai_integration::{
        ai_service::AiService,
        ai_state::AiState,
        repositories::{
            sqlite_ai_repository::SqliteAiRepository, traits::ai_repository::AiRepository,
        },
    },
    backend::{
        brainy_backend_http_client::BrainyBackendHttpClient,
        traits::brainy_backend_client::BrainyBackendClient,
    },
    backup::{
        repositories::traits::backup_repository::BackupRepository,
        sqlite_backup_repository::SqliteBackupRepository,
    },
    cells::{
        cell_service::CellService,
        repositories::{
            sqlite_cell_repository::SqliteCellRepository,
            sqlite_review_repository::SqliteReviewRepository,
            traits::{cell_repository::CellRepository, review_repository::ReviewRepository},
        },
    },
    common::utils::create_sqlite_pool::create_sqlite_pool,
    file_system::{
        file_system_service::FileSystemService,
        repositories::{
            sqlite_file_repository::SqliteFileRepository,
            sqlite_folder_repository::SqliteFolderRepository,
            traits::{file_repository::FileRepository, folder_repository::FolderRepository},
        },
    },
    fsrs::{
        entities::repositories::{
            sqlite_fsrs_repository::SqliteFsrsRepository, traits::fsrs_repository::FsrsRepository,
        },
        fsrs_service::FsrsService,
    },
    local_configurations::repositories::{
        sqlite_local_configuration_repository::SqliteLocalConfigurationRepository,
        traits::local_configuration_repository::LocalConfigurationRepository,
    },
    settings::{Settings, get_settings_dir},
    sync::{
        repositories::{
            sqlite_sync_repository::SqliteSyncRepository, traits::sync_repository::SyncRepository,
        },
        sync_service::SyncService,
    },
};
use injector::{injector::Injector, register_scope};
use reqwest::Url;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tauri::Manager;

use ai_integration::ai_api::{
    delete_ai_chat, get_all_ai_chats_sorted_by_date_desc, get_chat_messages_ordered,
    rename_ai_chat, stop_ai_generation, stream_ai_response,
};
use backend::api::auth_api::{
    is_signed_in, resend_email_verification_code, sign_in, sign_out, sign_up, update_password,
    verify_user_email,
};
use backend::api::user_api::{delete_user, get_user_information, update_user_information};
use cells::api::cell_api::{
    create_cell, delete_cell, get_cells_for_files_with_fsrs_profile_ids,
    get_file_cells_ordered_by_index, move_cell, update_cells_contents,
};
use cells::api::repetition_api::{get_study_repetition_counts, reset_repetitions_for_cell};
use cells::api::review_api::{get_home_statistics, register_review};
use cells::api::search_api::search_cells;
use file_system::api::file_system_api::{
    create_file, create_folder, delete_file, delete_folder, get_review_tree_folder_for_root,
    move_file, move_folder, rename_file, rename_folder,
};
use fsrs::fsrs_api::{
    create_profile, delete_fsrs_profile, get_all_fsrs_profiles, get_file_fsrs_profile,
    get_folder_fsrs_profile, get_fsrs_profile_choice_for_file, get_fsrs_profile_choice_for_folder,
    get_parent_fsrs_profile_for_file, get_parent_fsrs_profile_for_folder,
    set_fsrs_profile_choice_for_file, set_fsrs_profile_choice_for_folder, update_profile,
};
use settings::settings_api::{get_settings, update_settings};

pub use sync::sync_api::sync;

pub use file_system::api::export_import_api::{export_file, export_folder, import};

use tauri_plugin_window_state::StateFlags;
use tokio::sync::Mutex;

pub type Guid = uuid::Uuid;

pub const ROOT_FOLDER_ID: Guid = uuid::uuid!("00000000-0000-0000-0000-000000000001");
pub const DEFAULT_FSRS_PROFILE_ID: Guid = uuid::uuid!("00000000-0000-0000-0000-000000000002");

pub mod generated_code {
    include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));
}

type DbTransaction = Mutex<Transaction<'static, Sqlite>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<(), String> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let settings_directory = get_settings_dir()
        .await
        .expect("Cannot get settings directory!");
    let settings = Settings::init_settings_and_get(settings_directory.clone())
        .await
        .unwrap();

    let mut tauri_builder =
        tauri::Builder::default().plugin(tauri_plugin_clipboard_manager::init());

    #[cfg(desktop)]
    {
        tauri_builder = tauri_builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

    // TODO: move to own file
    let mut injector = Injector::default();

    let pool = create_sqlite_pool(&format!("sqlite:///{}", settings.database_location))
        .await
        .expect("Error connecting to Sqlite database");
    injector.register_singleton(Arc::new(pool));

    let backend_url = Url::parse("http://localhost:5078").unwrap();
    injector.register_singleton::<dyn BrainyBackendClient>(Arc::new(
        BrainyBackendHttpClient::new(backend_url).expect("Cannot create backend client"),
    ));

    injector.register_singleton(Arc::new(Mutex::new(settings)));
    injector.register_singleton(Arc::new(AiState::default()));

    register_scope!(injector, dyn FolderRepository, SqliteFolderRepository);
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    register_scope!(injector, dyn CellRepository, SqliteCellRepository);
    register_scope!(injector, dyn ReviewRepository, SqliteReviewRepository);
    register_scope!(
        injector,
        dyn LocalConfigurationRepository,
        SqliteLocalConfigurationRepository
    );
    register_scope!(injector, dyn SyncRepository, SqliteSyncRepository);
    register_scope!(injector, dyn BackupRepository, SqliteBackupRepository);
    register_scope!(injector, dyn FsrsRepository, SqliteFsrsRepository);
    register_scope!(injector, dyn AiRepository, SqliteAiRepository);

    register_scope!(injector, FileSystemService);
    register_scope!(injector, CellService);
    register_scope!(injector, FsrsService);
    register_scope!(injector, SyncService);
    register_scope!(injector, AiService);

    injector.register_scope_factory::<DbTransaction>(|scope| {
        Box::pin(async move {
            let pool = scope.resolve::<SqlitePool>().await;
            let tx = pool.begin().await.expect("Cannot create a new transaction");
            Arc::new(Mutex::new(tx))
        })
    });

    tauri_builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            app.manage(injector);

            #[cfg(dev)]
            {
                let _ = app
                    .get_webview_window("main")
                    .expect("no main window")
                    .set_title("Brainy - development");
            }

            // TODO:
            // let backup_service = BackupService::new(
            //     repositories_context.local_configuration_repository(),
            //     repositories_context.backup_repository(),
            //     settings_directory,
            // );
            // Starting backup service.
            // tokio::spawn(async move {
            //     let mut interval =
            //         tokio::time::interval(Duration::from_mins(TIME_BETWEEN_BACKUPS_IN_MINUTES));
            //
            //     loop {
            //         interval.tick().await;
            //
            //         if let Err(err) = backup_service.ensure_backup().await {
            //             log::error!(
            //                 "An error happened when saving a backup of your files {:?}",
            //                 err
            //             );
            //         }
            //     }
            // });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Cells
            create_cell,
            delete_cell,
            get_cells_for_files_with_fsrs_profile_ids,
            get_file_cells_ordered_by_index,
            move_cell,
            update_cells_contents,
            // Search
            search_cells,
            // File System
            create_file,
            create_folder,
            delete_file,
            delete_folder,
            get_review_tree_folder_for_root,
            move_file,
            move_folder,
            rename_file,
            rename_folder,
            // Repetitions
            get_study_repetition_counts,
            reset_repetitions_for_cell,
            // Review
            get_home_statistics,
            register_review,
            // Settings
            get_settings,
            update_settings,
            // Export/Import
            export_file,
            export_folder,
            import,
            // Auth
            is_signed_in,
            resend_email_verification_code,
            sign_in,
            sign_out,
            sign_up,
            update_password,
            verify_user_email,
            // User
            delete_user,
            get_user_information,
            update_user_information,
            // Sync
            sync,
            // FSRS
            create_profile,
            delete_fsrs_profile,
            get_all_fsrs_profiles,
            get_file_fsrs_profile,
            get_folder_fsrs_profile,
            get_fsrs_profile_choice_for_file,
            get_fsrs_profile_choice_for_folder,
            get_parent_fsrs_profile_for_file,
            get_parent_fsrs_profile_for_folder,
            set_fsrs_profile_choice_for_file,
            set_fsrs_profile_choice_for_folder,
            update_profile,
            // AI
            delete_ai_chat,
            get_all_ai_chats_sorted_by_date_desc,
            get_chat_messages_ordered,
            rename_ai_chat,
            stop_ai_generation,
            stream_ai_response,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
