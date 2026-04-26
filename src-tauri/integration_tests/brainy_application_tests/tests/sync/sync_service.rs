// TODO:
// These tests require MockBrainyBackendClient which is generated via
// #[cfg_attr(test, automock)] in brainy_application and therefore not available to external crates.
// To re-enable: create a mock in this crate using mockall::mock! and implement
// BrainyBackendClient for it. The tests also depend on brainy_application generated protobuf types.
