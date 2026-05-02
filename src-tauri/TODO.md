# TODO — Rust Architecture Improvements
<!--TODO: remove at end-->

## Critical 
- [X] Split the services into smaller where each one is responsible for one thing, all services are helper
- [X] Make a hook in the front end for calling {value, isLoading, error} the backend, remove all (Remove Request)
- [X] Refactor front-end types, move the to `api` folder
- [ ] There should nothing be as request and response without DTO suffix, it is either a value object or a DTO, look at models folders in front-end
- [X] Better errors, one per use case instead of one per service
- [ ] Make CLAUDE.md file
- [X] Let Claude take a round for the front-end
- [ ] Better repository error
- [X] Update dependencies

## Front-end Critical

- [ ] **Event listener uses hardcoded string instead of constant on removal** — `src/features/Editor/components/Editor.tsx:69`: `removeEventListener("toolCallAccepted")` should use `TOOL_CALL_ACCEPTED_EVENT` constant; if the constant changes the listener is never removed, causing a memory leak
- [ ] **Non-null assertion on nullable query parameter** — `src/features/Editor/components/Editor.tsx:36`: `searchParams.get(FILE_ID_QUERY_PARAMETER)!` crashes if user navigates to `/editor` without `?fileId=`; add a guard redirect
- [ ] **Unsafe non-null assertion on `find()` result** — `src/features/EditableCells/hooks/useAutoSave.ts:74`: `newCells.find(c => c.id === id)!.content = content` crashes if the cell is not found; validate before mutating
- [ ] **`alert()` used for error display** — `src/stores/sync/syncActions.ts:28`: Browser `alert()` blocks the UI thread and is inaccessible; replace with Redux error state consumed by a toast/banner component
- [ ] **`createCell` spreads entire `Cell` entity to backend** — `src/api/cells/api/cellApi.ts:17-19`: `invoke("create_cell", { ...cell })` sends all internal fields; define an explicit DTO with only the required fields

## Front-end Medium

- [ ] **Pointless `await Promise.resolve()` in event callbacks** — `src/features/EditableCells/components/EditableCells.tsx:135,153` and `src/features/SideBar/components/SyncRow.tsx:35,58`: callbacks are marked `async` and await a no-op; remove `async`/`await` or add actual async work
- [ ] **`getReviewTreeFolderForRoot` does nothing** — `src/stores/fileSystem/fileSystemActions.ts:21-22`: action body is `() => Promise.resolve()` so it always refetches unconditionally; implement real logic or remove the callback indirection
- [ ] **Reviewer bounds check missing** — `src/features/Reviewer/components/Reviewer.tsx:100-106`: `dueToday[currentCellIndex]` is accessed without a bounds check; if `currentCellIndex` is stale after cards are rated, this is `undefined`
- [ ] **Non-null assertions on Redux state in Settings** — e.g. `src/features/Settings/components/Settings.tsx:96-97`: `state.userInformation!` will throw if the user slice hasn't loaded yet; use optional chaining or a loading guard
- [ ] **No error boundaries** — a single component throw crashes the entire app; add error boundaries at the route level and around major feature areas (Editor, Reviewer, Sync)
- [ ] **Selectors not memoized** — most selectors in `src/stores/` are plain arrow functions; only `selectFileById` uses `createSelector`; add memoization to selectors that derive non-primitive values to avoid unnecessary re-renders
- [ ] **`void (async () => await fn())()` IIFE pattern** — `src/features/Home/components/Home.tsx:37-39` and elsewhere: replace with `void fn()` or a `useEffect`; the IIFE wrapper adds noise without benefit

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
