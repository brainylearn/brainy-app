use sea_orm::{DbConn, entity::*, query::*};

use crate::entity::file;

pub async fn create_file(db_conn: &impl ConnectionTrait, path: String) -> Result<i32, String> {
    let path = path.trim_matches('/').to_string();
    if path.trim().is_empty() {
        return Err("Name cannot be empty!".into());
    }
    if path.contains("/") {
        create_folder_recursively(db_conn, &get_folder_path(&path)).await?;
    }
    if file_exists(db_conn, path.clone()).await? {
        return Err("File already exists!".into());
    }

    let active_model = file::ActiveModel {
        path: Set(path),
        is_folder: Set(false),
        ..Default::default()
    };

    let result = active_model.insert(db_conn).await;
    match result {
        Ok(insert_result) => Ok(insert_result.id),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn create_folder(db_conn: &impl ConnectionTrait, path: String) -> Result<i32, String> {
    let path = path.trim_matches('/').to_string();
    if path.trim().is_empty() {
        return Err("Name cannot be empty!".into());
    }
    if folder_exists(db_conn, path.clone()).await? {
        return Err("Folder already exists!".into());
    }
    create_folder_recursively(db_conn, &path).await
}

async fn file_exists(db_conn: &impl ConnectionTrait, path: String) -> Result<bool, String> {
    let result = file::Entity::find()
        .filter(file::Column::Path.eq(path))
        .filter(file::Column::IsFolder.eq(false))
        .count(db_conn)
        .await;

    match result {
        Ok(result) => Ok(result > 0),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn list_folder_children_recursively(
    db_conn: &DbConn,
    id: i32,
) -> Result<Vec<file::Model>, String> {
    let folder = get_by_id(db_conn, id).await?;
    let result = file::Entity::find()
        .filter(file::Column::Path.starts_with(folder.path + "/"))
        .all(db_conn)
        .await;
    match result {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn list_folder_children(db_conn: &DbConn, id: i32) -> Result<Vec<file::Model>, String> {
    let folder = get_by_id(db_conn, id).await?;
    let folder_children = list_folder_children_recursively(db_conn, id).await?;
    let slashes_count = folder.path.chars().filter(|c| *c == '/').count();
    Ok(folder_children
        .into_iter()
        .filter(|sub_file| sub_file.path.chars().filter(|c| *c == '/').count() == slashes_count + 1)
        .collect())
}

pub async fn get_by_id(db_conn: &DbConn, id: i32) -> Result<file::Model, String> {
    let result = file::Entity::find_by_id(id).one(db_conn).await;
    match result {
        Ok(result) => Ok(result.unwrap()),
        Err(err) => Err(err.to_string()),
    }
}

async fn create_folder_recursively(
    db_conn: &impl ConnectionTrait,
    path: &str,
) -> Result<i32, String> {
    let mut current_path = String::new();
    // The id of the folder with the full path
    let mut folder_id = 0;

    for name in path.split("/") {
        if !current_path.is_empty() {
            current_path.push('/');
        }
        current_path.push_str(name);
        if current_path.is_empty() {
            continue;
        }

        if !folder_exists(db_conn, current_path.to_string()).await? {
            let active_model = file::ActiveModel {
                path: Set(current_path.clone()),
                is_folder: Set(true),
                ..Default::default()
            };

            let result = active_model.insert(db_conn).await;
            folder_id = match result {
                Ok(insert_result) => insert_result.id,
                Err(err) => return Err(err.to_string()),
            }
        }
    }

    Ok(folder_id)
}

async fn folder_exists(db_conn: &impl ConnectionTrait, path: String) -> Result<bool, String> {
    let result = file::Entity::find()
        .filter(file::Column::Path.eq(path))
        .filter(file::Column::IsFolder.eq(true))
        .count(db_conn)
        .await;

    match result {
        Ok(result) => Ok(result > 0),
        Err(err) => Err(err.to_string()),
    }
}

fn get_folder_path(path: &str) -> String {
    let index = path.rfind("/");
    match index {
        Some(index) => path.chars().take(index).collect::<String>(),
        None => "".into(),
    }
}
