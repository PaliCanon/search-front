//! Production JSON output and request privacy, without a telemetry backend.
#![cfg(feature = "ssr")]

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn startup_failure_carries_the_deployed_tag_without_config_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_search_front"))
        .env_clear()
        .env("SEARCH_API_URL", "https://canary-private.invalid")
        .env("SEARCH_FRONT_ENV", "v9.8.7")
        .env("SEARCH_FRONT_SHA", "abcdef0123")
        .env("SEARCH_FRONT_DEPLOYMENT_ENV", "test")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stderr.len(), 0);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("canary-private"));
    let log: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(log["service.namespace"], "palicanon");
    assert_eq!(log["service.name"], "search");
    assert_eq!(log["service.version"], "v9.8.7");
    assert_eq!(log["service.commit"], "abcdef0");
    assert_eq!(log["event.name"], "service.failed");
    assert_eq!(log["stage"], "SEARCH_API_TOKEN");
}

#[tokio::test(flavor = "current_thread")]
async fn http_error_logs_use_templates_and_ignore_request_data() {
    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;
    use tracing::instrument::WithSubscriber;
    let output = Capture::default();
    let writer = output.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_writer(move || writer.clone())
        .finish();
    let app = Router::new()
        .route(
            "/word/{word}",
            get(|| async { axum::http::StatusCode::BAD_GATEWAY }),
        )
        .layer(axum::middleware::from_fn(
            search_front::telemetry::server::request,
        ));
    let request = Request::get("/word/canary-word?q=canary-query")
        .header("authorization", "Bearer canary-token")
        .header("cookie", "canary-account")
        .header("x-request-id", "canary-id")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.oneshot(request)
            .with_subscriber(subscriber)
            .await
            .unwrap()
            .status(),
        502
    );
    let bytes = output.0.lock().unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(!text.contains("canary"), "{text}");
    let log: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(log["http.route"], "/word/{word}");
    assert_eq!(log["http.response.status_code"], 502);
    assert_eq!(log["severity"], "ERROR");
    assert!(log["span"]["request.id"].as_str().is_some());
}
