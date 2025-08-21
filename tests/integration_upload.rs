// integration_upload.rs
// This test is a placeholder that documents how to run an integration test against the running server.
// It is ignored by default. To run it, start the server locally and run `cargo test -- --ignored`.

#[cfg(test)]
mod integration {
    use std::time::Duration;

    #[test]
    #[ignore]
    fn upload_endpoint_smoke() {
        // Start server separately (e.g. `cargo run`) then execute this test.
        // This test is intentionally ignored in CI and used as a local helper.
        std::thread::sleep(Duration::from_millis(10));
        // Use reqwest blocking to POST a multipart file to http://localhost:3000/communication/upload
        // Example left as a manual step to avoid heavy test infra.
        assert!(true);
    }
}
