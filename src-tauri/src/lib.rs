mod api;
mod domain;
mod dto;
mod entity;
mod infrastructure;
mod migration;
mod service;
mod util;
mod value_objects;

use service::settings_service;
use sqlx::sqlite::SqlitePoolOptions;
use tauri::Manager;

use api::*;
use tauri_plugin_window_state::StateFlags;
use tokio::sync::Mutex;
use util::database_util::load_database;

use crate::{domain::{entities::folder::{FolderEvent, FolderEventHandler}, events::{EventBus, EventHandler}, repositories::{folder_repository::FolderRepository, repositories_context::RepositoriesContext}, value_objects::path::Path}, infrastructure::repositories::{sqlite_folder_repository::SqliteFolderRepository, sqlite_repositories_context::SqliteRepositoriesContext}};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<(), String> {
    // TODO:
    // let mut bus = EventBus::new();
    // bus.subscribe(FolderEventHandler);
    // bus.publish(FolderEvent::FolderDeleted).await;

    settings_service::init_settings();
    // TODO: fix path
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:////home/ramikw/Downloads/test.db?mode=rwc").await.unwrap();
    let context = SqliteRepositoriesContext::new(pool);

    let db_conn = load_database(&settings_service::get_settings().database_location).await;

    let mut tauri_builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        tauri_builder = tauri_builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

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
        .setup(|app| {
            let context_box: Box<dyn RepositoriesContext> = Box::new(context);
            app.manage(Mutex::new(db_conn));
            app.manage(Mutex::new(context_box));
            #[cfg(dev)]
            {
                let _ = app
                    .get_webview_window("main")
                    .expect("no main window")
                    .set_title("Brainy - development");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Cells
            create_cell,
            delete_cell,
            get_cells_for_files,
            get_file_cells_ordered_by_index,
            move_cell,
            update_cells_contents,
            // Search
            search_cells,
            // Files & Folders
            create_file,
            create_folder,
            delete_file,
            delete_folder,
            get_files,
            move_file,
            move_folder,
            rename_file,
            rename_folder,
            // Repetitions
            get_file_repetitions,
            get_repetitions_for_files,
            get_study_repetition_counts,
            reset_repetitions_for_cell,
            // Review
            get_home_statistics,
            register_review,
            // Settings
            get_settings,
            update_settings,
            // Export/Import
            export,
            import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
