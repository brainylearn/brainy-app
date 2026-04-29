# TODO — Rust Architecture Improvements
<!--TODO: remove at end-->

## Critical 
- [ ] Split the services into smaller where each one is responsible for one thing
- [ ] Make a hook in the front end for calling {value, isLoading, error} the backend, remove all (Remove Request)
- [ ] Refactor front-end types, move the to `api` folder
- [ ] Better errors, one per use case instead of one per service
- [ ] Make CLAUDE.md file
- [ ] Let claude take a round for the front-end

(the application and presentation layer for my app is the same)
(It is okay not to use a dto and return an entity directly, dtos are just for special cases)

## Medium Priority

- [ ] **`rewrite_str` panics mid-import** — `src/file_system/file_system_service.rs:361`: `rewrite_str(...).unwrap()` on user-supplied HTML can panic, leaving DB in partially-imported state; propagate as `FileServiceError`
- [ ] **FTS query uses `LIKE` instead of `MATCH`** — `src/infrastructure/repositories/sqlite/sqlite_cell_repository.rs:563`: Bypasses the full-text index entirely; use `WHERE cells_fts MATCH $1`
- [ ] **N+1 queries in `update_cells_contents`** — `src/cells/api/cell_api.rs:94-99`: Issues 2N SQL round-trips for N cells; add a bulk-fetch + batch-update path
- [ ] **N+1 queries in `get_cells_for_files_with_fsrs_profile_ids`** — `src/cells/api/cell_api.rs:114-133`: O(files × folder depth) queries due to per-file parent traversal
- [ ] **`deleted_entities` table missing UNIQUE constraint** — `migrations/0001_create_tables.sql:160`: Duplicate rows accumulate on sync retry; add `UNIQUE(entity_id, entity_name)` and use upsert
- [ ] **`defer_foreign_keys` does not disable FK checks** — `src/infrastructure/extensions/unit_of_work.rs:36`: `PRAGMA defer_foreign_keys = ON` defers but doesn't disable; if a Cell arrives before its File during sync the transaction fails at commit; review sync ordering or use `PRAGMA foreign_keys = OFF` outside the transaction
- [ ] **Sync push uses `>=` instead of `>` for last_sync_date** — `src/sync/sync_service.rs:356-528`: Entities modified exactly at `last_sync_date` are re-uploaded every sync cycle; change to `>`
- [ ] **`backup_service` only deletes 1 excess backup** — `src/backup/backup_service.rs:146`: When count exceeds limit by more than 1 only one file is deleted; loop until within limit
- [ ] **`unwrap()` on `io::Result` in backup loop** — `src/backup/backup_service.rs:121`: `next_entry().await.unwrap()` panics on filesystem error, killing the background backup loop permanently; use `?`
- [ ] **`unwrap()` on weight parsing from DB** — `src/infrastructure/repositories/sqlite/sqlite_rows/fsrs_profile_row.rs:20`: Malformed `weights` column panics the async task; propagate as a repository error
- [ ] **`unwrap()` on `parent_id` in fsrs_api** — `src/fsrs/fsrs_api.rs:109,133`: Calling these commands on the root folder (no parent) panics; validate or return an error
- [ ] **`unwrap()` on RFC3339 parse from config** — `src/sync/sync_service.rs:91`, `src/backup/backup_service.rs:69`: Corrupt `LAST_SYNC_DATE` / `LAST_BACKUP_DATE` config value panics the task; propagate as an error
- [ ] **`serde_json::to_string().unwrap()` on export** — `src/file_system/api/export_import_api.rs:55`: NaN/Infinity floats cause panic; propagate with `?`
- [ ] **Unbounded recursion in FSRS profile traversal** — `src/fsrs/fsrs_api.rs:153`, `src/cells/api/cell_api.rs:147`: A cycle in `parent_id` (no DB-level cycle prevention) would exhaust the stack; add a max-depth counter
- [ ] **HTTP 201/204 treated as error** — `src/infrastructure/clients/brainy_backend_http_client.rs:363`: Only `200 OK` is accepted as success; `201 Created` and `204 No Content` are valid for POST/DELETE and should be accepted
- [ ] **`FsrsProfile::new_unchecked` skips weight count validation** — `src/fsrs/entities/fsrs_profile.rs:38`: Sync and DB paths bypass the 21-weight check; FSRS silently operates on wrong-length vectors
- [ ] **`move_cell` transient duplicate index during backward move** — `src/cells/cell_service.rs:72`: Two cells briefly share the same index mid-transaction; would violate a uniqueness constraint if one is added
- [ ] **`convert_rows_to_cells(...).remove(0)` panics on empty result** — `src/infrastructure/repositories/sqlite/sqlite_cell_repository.rs:82`: Missing ID returns empty vec and `.remove(0)` panics; return `RepositoryError::NotFound`

## Low Priority

- [ ] **Regex compiled per call** — `src/cells/entities/cell.rs:159,203`: `Regex::new(...)` called on every `update_searchable_content`; use `OnceLock<Regex>` statics
- [ ] **`std::sync::Mutex` unwrap in async context** — `src/infrastructure/clients/brainy_backend_http_client.rs:207,325`: Panics on mutex poison and causes latency jitter; handle the poison case
- [ ] **Backup interval burst behavior** — `src/lib.rs:110-122`: Default `MissedTickBehavior::Burst` piles up ticks if backup runs long; use `MissedTickBehavior::Skip`
- [ ] **`unwrap()` on keyring write** — `src/infrastructure/clients/brainy_backend_http_client.rs:333`: Panics if OS keychain is locked; log a warning and continue instead
- [ ] **Transaction factory `panic!`** — `src/common/utils/create_injector.rs:133`, `src/sync/unit_of_work.rs:53`: `begin().await.expect(...)` should propagate through `Result`
- [ ] **`get_fsrs_profile_id_for_item_recursively` panics** — `src/cells/api/cell_api.rs:148`: `parent_id.unwrap()` panics when `Inherit` is set on a root-level file; return an error instead
- [ ] **`path.to_str().unwrap()` on non-UTF-8 paths** — `src/ai_integration/ai_service.rs:340,366,494`: Panics on valid non-UTF-8 filesystem paths; use `to_string_lossy()` or propagate an error
