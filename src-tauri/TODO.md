# TODO — Rust Architecture Improvements
<!--TODO: remove at end-->

## Critical 
- [X] Split the services into smaller where each one is responsible for one thing, all services are helper
- [X] Make a hook in the front end for calling {value, isLoading, error} the backend, remove all (Remove Request)
- [X] Refactor front-end types, move the to `api` folder
- [X] There should nothing be as request and response without DTO suffix, it is either a value object or a DTO, look at models folders in front-end
- [X] Better errors, one per use case instead of one per service
- [X] Let Claude take a round for the front-end
- [X] Update dependencies
- [ ] Better repository error
- [X] Make CLAUDE.md file

Documentation:
- Claude.md works good as documentation
- the application and presentation layer for my app is the same, called API
- It is okay not to use a dto and return an entity directly, dtos are just for special cases
- To choose if something is a value object or a dto, think if it is part of the language or just a convenince for transfering data

