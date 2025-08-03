use std::{
    collections::{HashMap, VecDeque}, sync::Arc
};

use async_trait::async_trait;
use sqlx::{sqlite::SqlitePool, Sqlite, Transaction};
use tokio::sync::Mutex;

use crate::domain::{
    entities::folder::{Folder, FolderError},
    repositories::{folder_repository::FolderRepository, repository_error::RepositoryError},
    value_objects::path::Path,
};

#[derive(sqlx::FromRow)]
struct FolderRow {
    id: uuid::fmt::Hyphenated,
    path: String,
}

impl From<FolderRow> for Folder {
    fn from(value: FolderRow) -> Self {
        Folder::new(Some(value.id.into()), Path::new(&value.path))
    }
}

impl From<&FolderRow> for Folder {
    fn from(value: &FolderRow) -> Self {
        Folder::new(Some(value.id.into()), Path::new(&value.path))
    }
}

pub struct SqliteFolderRepository {
    pub pool: Arc<SqlitePool>,
    pub tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

// TODO: use query!
#[async_trait]
impl FolderRepository for SqliteFolderRepository {
    async fn get_by_path(&self, path: &Path) -> Result<Option<Folder>, RepositoryError> {
        let rows = sqlx::query_as::<_, FolderRow>("SELECT * FROM folders WHERE path LIKE $1")
            .bind(path.val() + "%")
            .fetch_all(&*self.pool)
            .await;

        if let Err(err) = rows {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        Ok(parse_rows_into_folder(rows.unwrap(), path).unwrap())
    }

    async fn folder_exists(&self, path: &Path) -> Result<bool, RepositoryError> {
        let row =
            sqlx::query_as::<_, (bool,)>("SELECT EXISTS (SELECT * FROM folders WHERE path = $1)")
                .bind(path.val())
                .fetch_one(&*self.pool)
                .await;

        match row {
            Ok(row) => Ok(row.0),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn upsert(&mut self, folder: &Folder) -> Result<(), RepositoryError> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().unwrap();

        let mut queue = VecDeque::new();
        queue.push_back(folder);

        while !queue.is_empty() {
            let curr = queue.pop_front().unwrap();

            sqlx::query(
                "
                INSERT INTO folders (id, path)
                VALUES ($1, $2)
                ON CONFLICT(id) DO UPDATE SET
                path = $2;
                ",
            )
            .bind(curr.id())
            .bind(curr.path().val())
            .execute(tx.as_mut())
            .await
            // TODO: error handling
            .unwrap();

            queue.extend(curr.subfolders());
        }

        Ok(())
    }
}

fn parse_rows_into_folder(
    rows: Vec<FolderRow>,
    path: &Path,
) -> Result<Option<Folder>, FolderError> {
    let folder = rows.iter().find(|row| row.path == path.val());

    if let None = folder {
        return Ok(None);
    }

    let mut folder = folder.unwrap().into();
    let mut subfolders_by_path: HashMap<Path, Vec<Folder>> = HashMap::new();

    for row in rows {
        if row.path == path.val() {
            continue;
        }

        let path = Path::new(&row.path);
        subfolders_by_path
            .entry(path.parent_directory().unwrap())
            .or_insert(Vec::new())
            .push(row.into());
    }

    fn parse_folder(
        folder: &mut Folder,
        subfolders_by_path: &mut HashMap<Path, Vec<Folder>>,
    ) -> Result<(), FolderError> {
        let subfolders = subfolders_by_path.remove(folder.path()).unwrap_or_default();
        for mut subfolder in subfolders.into_iter() {
            parse_folder(&mut subfolder, subfolders_by_path)?;
            folder.add_subfolder(subfolder)?;
        }

        Ok(())
    }
    parse_folder(&mut folder, &mut subfolders_by_path)?;

    Ok(Some(folder))
}
