The file `integration_test.rs.disabled` (originally `integration_test.rs`)
has been temporarily disabled. Running `cargo test` on integration tests
that link against the `memtrack-rs` library (when compiled with
`tokio = { features = ["full"] }`) causes a fatal Tokio runtime error:
`fatal runtime error: assertion failed: thread_info.stack_guard.get().is_none() && thread_info.thread.get().is_none()`.
This error occurs even if the test functions are empty.

To test the library's core functionality, particularly memory tracking and
output generation (JSON/SVG), a separate test program approach will be
used in `tests/lifecycle_validation_test.rs`. This program will be
compiled and run as a standalone executable.
