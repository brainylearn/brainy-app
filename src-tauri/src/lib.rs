mod api;
mod dto;
mod entity;
mod migration;
mod service;
mod util;
mod value_objects;

use std::str::FromStr;

use brainy_core::{
    common::domain_services::DomainServices,
    file_system::file_system_service::DefaultFileSystemService,
};
use brainy_infrastructure::{
    repositories_context::RepositoriesContext,
    sqlite_repositories_context::SqliteRepositoriesContext,
};
use service::settings_service;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tauri::Manager;

use api::*;
use tauri_plugin_window_state::StateFlags;
use tokio::sync::Mutex;
use util::database_util::load_database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<(), String> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    settings_service::init_settings();

    // TODO: fix path
    // TODO: error handling
    let options =
        SqliteConnectOptions::from_str("sqlite:////home/ramikw/Downloads/test.db?mode=rwc")
            .unwrap();

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .unwrap();
    let repositoriies_context = SqliteRepositoriesContext::new(pool);

    let domain_services = DomainServices {
        file_system_service: Box::new(DefaultFileSystemService {
            folder_repository: repositoriies_context.folder_repository(),
            file_repository: repositoriies_context.file_repository(),
        }),
    };

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
            let repositories_context_box: Box<dyn RepositoriesContext> =
                Box::new(repositoriies_context);
            app.manage(Mutex::new(db_conn));
            app.manage(Mutex::new(repositories_context_box));
            app.manage(domain_services);
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
