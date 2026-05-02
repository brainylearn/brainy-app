# TODO — Rust Architecture Improvements
<!--TODO: remove at end-->

## Critical 
- [X] Split the services into smaller where each one is responsible for one thing, all services are helper
- [X] Make a hook in the front end for calling {value, isLoading, error} the backend, remove all (Remove Request)
- [X] Refactor front-end types, move the to `api` folder
- [X] There should nothing be as request and response without DTO suffix, it is either a value object or a DTO, look at models folders in front-end
- [X] Better errors, one per use case instead of one per service
- [ ] Make CLAUDE.md file
- [X] Let Claude take a round for the front-end
- [ ] Better repository error
- [X] Update dependencies

Documentation:
- the application and presentation layer for my app is the same, called API
- It is okay not to use a dto and return an entity directly, dtos are just for special cases
- To choose if something is a value object or a dto, think if it is part of the language or just a convenince for transfering data

## Medium Priority

- [ ] **FTS query uses `LIKE` instead of `MATCH`** — `src/infrastructure/repositories/sqlite/sqlite_cell_repository.rs:563`: Bypasses the full-text index entirely; use `WHERE cells_fts MATCH $1`
- [ ] **`defer_foreign_keys` does not disable FK checks** — `src/infrastructure/extensions/unit_of_work.rs:36`: `PRAGMA defer_foreign_keys = ON` defers but doesn't disable; if a Cell arrives before its File during sync the transaction fails at commit; review sync ordering or use `PRAGMA foreign_keys = OFF` outside the transaction
- [ ] **Unbounded recursion in FSRS profile traversal** — `src/fsrs/fsrs_api.rs:153`, `src/cells/api/cell_api.rs:147`: A cycle in `parent_id` (no DB-level cycle prevention) would exhaust the stack; add a max-depth counter
- [ ] **`FsrsProfile::new_unchecked` skips weight count validation** — `src/fsrs/entities/fsrs_profile.rs:38`: Sync and DB paths bypass the 21-weight check; FSRS silently operates on wrong-length vectors
- [ ] **`move_cell` transient duplicate index during backward move** — `src/cells/cell_service.rs:72`: Two cells briefly share the same index mid-transaction; would violate a uniqueness constraint if one is added
