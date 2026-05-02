# TODO — Rust Architecture Improvements
<!--TODO: remove at end-->

## Critical 
- [X] Split the services into smaller where each one is responsible for one thing, all services are helper
- [X] Make a hook in the front end for calling {value, isLoading, error} the backend, remove all (Remove Request)
- [X] Refactor front-end types, move the to `api` folder
- [ ] There should nothing be as request and response without DTO suffix, it is either a value object or a DTO, look at models folders in front-end
- [X] Better errors, one per use case instead of one per service
- [ ] Make CLAUDE.md file
- [ ] Let Claude take a round for the front-end
- [ ] Better repository error
- [X] Update dependencies

Documentation:
- the application and presentation layer for my app is the same, called API
- It is okay not to use a dto and return an entity directly, dtos are just for special cases
- To choose if something is a value object or a dto, think if it is part of the language or just a convenince for transfering data

## Medium Priority

- [ ] **FTS query uses `LIKE` instead of `MATCH`** — `src/infrastructure/repositories/sqlite/sqlite_cell_repository.rs:563`: Bypasses the full-text index entirely; use `WHERE cells_fts MATCH $1`
- [ ] **`defer_foreign_keys` does not disable FK checks** — `src/infrastructure/extensions/unit_of_work.rs:36`: `PRAGMA defer_foreign_keys = ON` defers but doesn't disable; if a Cell arrives before its File during sync the transaction fails at commit; review sync ordering or use `PRAGMA foreign_keys = OFF` outside the transaction
- [ ] **Sync push uses `>=` instead of `>` for last_sync_date** — `src/sync/sync_service.rs:356-528`: Entities modified exactly at `last_sync_date` are re-uploaded every sync cycle; change to `>`
- [ ] **`backup_service` only deletes 1 excess backup** — `src/backup/backup_service.rs:146`: When count exceeds limit by more than 1 only one file is deleted; loop until within limit
- [ ] **`unwrap()` on `io::Result` in backup loop** — `src/backup/backup_service.rs:121`: `next_entry().await.unwrap()` panics on filesystem error, killing the background backup loop permanently; use `?`
- [ ] **`unwrap()` on weight parsing from DB** — `src/infrastructure/repositories/sqlite/sqlite_rows/fsrs_profile_row.rs:20`: Malformed `weights` column panics the async task; propagate as a repository error
- [ ] **`unwrap()` on `parent_id` in fsrs_api** — `src/fsrs/fsrs_api.rs:109,133`: Calling these commands on the root folder (no parent) panics; validate or return an error
- [ ] **`unwrap()` on RFC3339 parse from config** — `src/sync/sync_service.rs:91`, `src/backup/backup_service.rs:69`: Corrupt `LAST_SYNC_DATE` / `LAST_BACKUP_DATE` config value panics the task; propagate as an error
- [ ] **`serde_json::to_string().unwrap()` on export** — `src/file_system/api/export_import_api.rs:55`: NaN/Infinity floats cause panic; propagate with `?`
- [ ] **Unbounded recursion in FSRS profile traversal** — `src/fsrs/fsrs_api.rs:153`, `src/cells/api/cell_api.rs:147`: A cycle in `parent_id` (no DB-level cycle prevention) would exhaust the stack; add a max-depth counter
- [ ] **`FsrsProfile::new_unchecked` skips weight count validation** — `src/fsrs/entities/fsrs_profile.rs:38`: Sync and DB paths bypass the 21-weight check; FSRS silently operates on wrong-length vectors
- [ ] **`move_cell` transient duplicate index during backward move** — `src/cells/cell_service.rs:72`: Two cells briefly share the same index mid-transaction; would violate a uniqueness constraint if one is added
- [ ] **`convert_rows_to_cells(...).remove(0)` panics on empty result** — `src/infrastructure/repositories/sqlite/sqlite_cell_repository.rs:82`: Missing ID returns empty vec and `.remove(0)` panics; return `RepositoryError::NotFound`
